//! The package model: one canonical id, many providers, resolved against the platform stack.
//!
//! A package names a piece of software once and lists ordered *provider* rows — each a
//! `(manager, id)` pair, optionally gated by a platform. Resolution walks the rows in order and
//! keeps those whose platform gate is satisfied by the [active stack](crate::resolve::ActiveStack);
//! the first is preferred. Whether a manager is actually *installed* is a separate, runtime concern
//! handled by `grim-pkg` — this layer is pure.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::de::{self, Deserializer};

use crate::resolve::ActiveStack;

/// A supported package manager. Closed on purpose: an unknown manager in a grimoire is an error, not
/// something to silently skip.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Manager {
    /// Homebrew (`brew`).
    Brew,
    /// Rust crates via `cargo` (binstall-able).
    Cargo,
    /// `uv tool` Python CLIs.
    Uv,
    /// Node globals via `npm`.
    Npm,
    /// Go modules via `go install`.
    Go,
    /// Python via `pip`.
    Pip,
    /// Debian/Ubuntu `apt`.
    Apt,
    /// Fedora/RHEL `dnf`.
    Dnf,
    /// Arch `pacman`.
    Pacman,
}

impl Manager {
    /// Parse a manager from its grimoire key, e.g. `"brew"`.
    pub fn from_key(key: &str) -> Option<Self> {
        Some(match key {
            "brew" => Manager::Brew,
            "cargo" => Manager::Cargo,
            "uv" => Manager::Uv,
            "npm" => Manager::Npm,
            "go" => Manager::Go,
            "pip" => Manager::Pip,
            "apt" => Manager::Apt,
            "dnf" => Manager::Dnf,
            "pacman" => Manager::Pacman,
            _ => return None,
        })
    }

    /// The manager's canonical key.
    pub fn as_str(self) -> &'static str {
        match self {
            Manager::Brew => "brew",
            Manager::Cargo => "cargo",
            Manager::Uv => "uv",
            Manager::Npm => "npm",
            Manager::Go => "go",
            Manager::Pip => "pip",
            Manager::Apt => "apt",
            Manager::Dnf => "dnf",
            Manager::Pacman => "pacman",
        }
    }
}

impl std::fmt::Display for Manager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One way to install a package: a manager, the manager-specific id, and an optional platform gate.
///
/// Written in a grimoire as `{ <manager> = "<id>" }`, optionally with `platform = "<name>"`:
///
/// ```toml
/// providers = [
///     { brew = "docker", platform = "macos" },
///     { apt  = "docker.io", platform = "linux" },
///     { cargo = "ripgrep" },
/// ]
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Provider {
    /// The package manager.
    pub manager: Manager,
    /// The manager-specific package id.
    pub id: String,
    /// A platform gate: this row only applies when the named platform is in the active stack.
    pub platform: Option<String>,
}

impl<'de> Deserialize<'de> for Provider {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let map = BTreeMap::<String, String>::deserialize(deserializer)?;
        let mut platform = None;
        let mut chosen: Option<(Manager, String)> = None;
        for (key, value) in map {
            if key == "platform" {
                platform = Some(value);
                continue;
            }
            let manager = Manager::from_key(&key)
                .ok_or_else(|| de::Error::custom(format!("unknown package manager `{key}`")))?;
            if chosen.is_some() {
                return Err(de::Error::custom(
                    "a provider row may name only one manager",
                ));
            }
            chosen = Some((manager, value));
        }
        let (manager, id) =
            chosen.ok_or_else(|| de::Error::custom("provider row names no manager"))?;
        Ok(Provider {
            manager,
            id,
            platform,
        })
    }
}

/// A package definition as written under `[package.<name>]`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageDef {
    /// The ordered provider rows; earlier rows are preferred.
    pub providers: Vec<Provider>,
}

/// A package: a canonical name and its ordered providers.
#[derive(Clone, Debug)]
pub struct Package {
    /// The canonical package name (the table key).
    pub name: String,
    /// The ordered provider rows; earlier rows are preferred.
    pub providers: Vec<Provider>,
}

impl Package {
    /// Build a package from its name and definition.
    pub fn from_def(name: impl Into<String>, def: PackageDef) -> Self {
        Package {
            name: name.into(),
            providers: def.providers,
        }
    }
}

/// One provider's standing for the current machine: eligible (its platform gate is satisfied) or
/// skipped (with a reason).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderChoice<'a> {
    /// The provider being judged.
    pub provider: &'a Provider,
    /// Whether it applies to this machine.
    pub eligible: bool,
    /// Why it was skipped, if it was.
    pub reason: Option<String>,
}

/// Judge every provider of a package against the active stack, in declared order. The first
/// `eligible` entry is the preferred provider; the rest of the eligibles are fallbacks (used by
/// `grim-pkg` when a preferred manager isn't installed). Useful for `--explain`.
pub fn resolve_package<'a>(package: &'a Package, stack: &ActiveStack) -> Vec<ProviderChoice<'a>> {
    package
        .providers
        .iter()
        .map(|provider| match &provider.platform {
            None => ProviderChoice {
                provider,
                eligible: true,
                reason: None,
            },
            Some(name) if stack.contains(name) => ProviderChoice {
                provider,
                eligible: true,
                reason: None,
            },
            Some(name) => ProviderChoice {
                provider,
                eligible: false,
                reason: Some(format!("platform `{name}` is not active")),
            },
        })
        .collect()
}

/// The preferred (first eligible) provider for a package on this machine, if any.
pub fn preferred_provider<'a>(package: &'a Package, stack: &ActiveStack) -> Option<&'a Provider> {
    resolve_package(package, stack)
        .into_iter()
        .find(|choice| choice.eligible)
        .map(|choice| choice.provider)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::{Arch, Facts, Os};
    use crate::platform::{Match, Platform, PlatformDef};
    use crate::resolve::resolve_stack;

    fn pkg(toml_body: &str) -> Package {
        let def: PackageDef = toml::from_str(toml_body).unwrap();
        Package::from_def("thing", def)
    }

    fn linux_stack() -> Vec<Platform> {
        vec![Platform::from_def(
            "linux",
            PlatformDef {
                predicate: Match {
                    os: Some(Os::Linux),
                    ..Default::default()
                },
                band: None,
                weight: 0,
                refines: vec![],
            },
        )]
    }

    #[test]
    fn deserializes_shorthand_rows() {
        let p = pkg(r#"providers = [{ brew = "ripgrep" }, { cargo = "ripgrep" }]"#);
        assert_eq!(p.providers[0].manager, Manager::Brew);
        assert_eq!(p.providers[0].id, "ripgrep");
        assert_eq!(p.providers[1].manager, Manager::Cargo);
        assert!(p.providers[0].platform.is_none());
    }

    #[test]
    fn platform_gate_parses() {
        let p = pkg(r#"providers = [{ apt = "docker.io", platform = "linux" }]"#);
        assert_eq!(p.providers[0].platform.as_deref(), Some("linux"));
    }

    #[test]
    fn rejects_two_managers_in_one_row() {
        let def = toml::from_str::<PackageDef>(r#"providers = [{ brew = "x", cargo = "y" }]"#);
        assert!(def.is_err());
    }

    #[test]
    fn rejects_unknown_manager() {
        let def = toml::from_str::<PackageDef>(r#"providers = [{ snap = "x" }]"#);
        assert!(def.is_err());
    }

    #[test]
    fn resolves_preferred_by_platform_gate() {
        let platforms = linux_stack();
        let stack = resolve_stack(&platforms, &Facts::new(Os::Linux, Arch::X86_64)).unwrap();
        let p = pkg(
            r#"providers = [{ brew = "docker", platform = "macos" }, { apt = "docker.io", platform = "linux" }]"#,
        );
        let chosen = preferred_provider(&p, &stack).unwrap();
        assert_eq!(chosen.manager, Manager::Apt);
        assert_eq!(chosen.id, "docker.io");

        // explain: the macos row is skipped with a reason
        let choices = resolve_package(&p, &stack);
        assert!(!choices[0].eligible);
        assert!(choices[0].reason.as_ref().unwrap().contains("macos"));
        assert!(choices[1].eligible);
    }

    #[test]
    fn ungated_providers_are_always_eligible() {
        let platforms = linux_stack();
        let stack = resolve_stack(&platforms, &Facts::new(Os::Linux, Arch::X86_64)).unwrap();
        let p = pkg(r#"providers = [{ cargo = "ripgrep" }]"#);
        assert_eq!(
            preferred_provider(&p, &stack).unwrap().manager,
            Manager::Cargo
        );
    }
}
