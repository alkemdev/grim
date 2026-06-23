//! Loading a grimoire from disk into the typed model.
//!
//! A grimoire is the configuration repository `grim` reads. For now a grimoire is a single
//! `grimoire.toml` holding identity metadata and platform definitions; package and file sections,
//! and splitting across multiple files, are additive and land in later phases. Parsing
//! ([`Grimoire::from_toml`]) is pure and tested; [`Grimoire::load`] adds the filesystem read.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

use crate::error::{LoadError, ResolveError};
use crate::facts::Facts;
use crate::platform::{Platform, PlatformDef};
use crate::resolve::{ActiveStack, resolve_stack};

/// The on-disk shape of a `grimoire.toml`. Top-level unknown sections are tolerated so later phases
/// (e.g. `[package.*]`) can be added without breaking older engines.
#[derive(Debug, Default, Deserialize)]
struct GrimoireFile {
    #[serde(default)]
    grimoire: Meta,
    #[serde(default)]
    platform: BTreeMap<String, PlatformDef>,
}

/// Identity and composition metadata for a grimoire.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Meta {
    /// A human-readable name for the grimoire.
    pub name: String,
    /// Other grimoires this one extends (composition). Parsed now; applied in a later phase.
    pub extends: Vec<String>,
}

/// A loaded grimoire: its metadata and the platforms it declares.
#[derive(Debug)]
pub struct Grimoire {
    /// Identity and composition metadata.
    pub meta: Meta,
    /// The declared platforms, in no particular order (precedence comes from resolution).
    pub platforms: Vec<Platform>,
}

impl Grimoire {
    /// Parse a grimoire from TOML text. Pure; the unit of testing.
    pub fn from_toml(text: &str) -> Result<Self, toml::de::Error> {
        let file: GrimoireFile = toml::from_str(text)?;
        let platforms = file
            .platform
            .into_iter()
            .map(|(name, def)| Platform::from_def(name, def))
            .collect();
        Ok(Grimoire {
            meta: file.grimoire,
            platforms,
        })
    }

    /// Load a grimoire from a directory containing a `grimoire.toml`.
    pub fn load_dir(dir: &Path) -> Result<Self, LoadError> {
        Self::load(&dir.join("grimoire.toml"))
    }

    /// Load a grimoire from a `grimoire.toml` file on disk.
    pub fn load(path: &Path) -> Result<Self, LoadError> {
        let text = std::fs::read_to_string(path).map_err(|source| LoadError::Io {
            path: path.display().to_string(),
            source,
        })?;
        Self::from_toml(&text).map_err(|source| LoadError::Parse {
            path: path.display().to_string(),
            source,
        })
    }

    /// Resolve the active platform stack for a machine against this grimoire's platforms.
    pub fn resolve(&self, facts: &Facts) -> Result<ActiveStack<'_>, ResolveError> {
        resolve_stack(&self.platforms, facts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::{Arch, Container, ContainerKind, Gpu, GpuVendor, Os};

    const EXAMPLE: &str = r#"
[grimoire]
name = "example"
extends = ["vendor/dotfiles"]

[platform.linux]
match = { os = "linux" }

[platform.cuda]
match = { gpu = { vendor = "nvidia" } }

[platform.devcontainer]
match = { container = {} }

[platform.workstation]
match = { hostname = "x1" }
"#;

    #[test]
    fn parses_metadata_and_platforms() {
        let g = Grimoire::from_toml(EXAMPLE).unwrap();
        assert_eq!(g.meta.name, "example");
        assert_eq!(g.meta.extends, vec!["vendor/dotfiles".to_string()]);
        assert_eq!(g.platforms.len(), 4);
    }

    #[test]
    fn resolves_stack_against_a_machine() {
        let g = Grimoire::from_toml(EXAMPLE).unwrap();
        let facts = Facts::new(Os::Linux, Arch::X86_64)
            .with_hostname("x1")
            .with_gpu(Gpu {
                vendor: GpuVendor::Nvidia,
                model: None,
            });
        let stack = g.resolve(&facts).unwrap();
        // host > hardware > os; no container present so devcontainer drops out
        assert_eq!(stack.names(), vec!["workstation", "cuda", "linux"]);
    }

    #[test]
    fn container_machine_tops_with_devcontainer() {
        let g = Grimoire::from_toml(EXAMPLE).unwrap();
        let facts = Facts::new(Os::Linux, Arch::X86_64)
            .with_hostname("x1")
            .with_container(Container {
                kind: ContainerKind::Devcontainer,
            });
        let stack = g.resolve(&facts).unwrap();
        assert_eq!(stack.names().first(), Some(&"devcontainer"));
    }

    #[test]
    fn rejects_unknown_meta_key() {
        let err = Grimoire::from_toml("[grimoire]\nnam = \"typo\"\n");
        assert!(err.is_err());
    }
}
