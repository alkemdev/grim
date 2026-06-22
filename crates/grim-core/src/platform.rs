//! Platforms: named, precedence-ordered layers matched against [`Facts`].
//!
//! A platform pairs a [`Match`] predicate over facts with a precedence. Precedence is expressed by a
//! [`Band`] (inferred from which facts the predicate constrains, or set explicitly) plus an optional
//! `weight` nudge; a `refines` relation breaks ties between same-precedence platforms. See
//! `docs/concepts/platforms.md`.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::facts::{Arch, Container, Distro, Facts, Gpu, GpuVendor, Libc, Os};

/// A precedence band. Each band has a base score; the active stack is ordered by score (descending).
/// Bands encode the natural layering — a host-specific platform outranks a hardware-specific one,
/// which outranks an OS-wide one — so most platforms never need an explicit number. A dev container
/// sits above everything.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Band {
    /// Matches every machine (empty predicate). Lowest precedence.
    Base,
    /// Constrained by OS only.
    Os,
    /// Constrained by CPU architecture.
    Arch,
    /// Constrained by Linux distribution.
    Distro,
    /// Constrained by hardware/runtime facts (CPU features, GPU, libc).
    Hardware,
    /// Constrained by a specific hostname.
    Host,
    /// Inside a container. Highest precedence — a dev container wins.
    Container,
}

impl Band {
    /// The base precedence score for this band.
    pub fn base(self) -> i64 {
        match self {
            Band::Base => 0,
            Band::Os => 100,
            Band::Arch => 200,
            Band::Distro => 300,
            Band::Hardware => 400,
            Band::Host => 900,
            Band::Container => 1000,
        }
    }

    /// The higher-precedence of two bands.
    fn max(self, other: Band) -> Band {
        if other.base() > self.base() {
            other
        } else {
            self
        }
    }
}

/// A predicate over [`Facts`]. Every present field is a constraint; a machine matches when *all* of
/// them hold (logical AND). `deny_unknown_fields` turns a misspelled key into a hard error rather
/// than a silently-ignored constraint.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Match {
    /// Require this OS.
    pub os: Option<Os>,
    /// Require this architecture.
    pub arch: Option<Arch>,
    /// Require this exact hostname.
    pub hostname: Option<String>,
    /// Require this C library.
    pub libc: Option<Libc>,
    /// Require a matching distribution.
    pub distro: Option<DistroMatch>,
    /// Require a matching GPU.
    pub gpu: Option<GpuMatch>,
    /// Require a (matching) container environment.
    pub container: Option<ContainerMatch>,
    /// Require matching CPU features.
    pub cpu_features: Option<FeatureMatch>,
}

impl Match {
    /// Whether this predicate holds for the given facts.
    pub fn matches(&self, f: &Facts) -> bool {
        if let Some(os) = &self.os
            && &f.os != os
        {
            return false;
        }
        if let Some(arch) = &self.arch
            && &f.arch != arch
        {
            return false;
        }
        if let Some(hostname) = &self.hostname
            && &f.hostname != hostname
        {
            return false;
        }
        if let Some(libc) = &self.libc
            && f.libc.as_ref() != Some(libc)
        {
            return false;
        }
        if let Some(distro) = &self.distro
            && !distro.matches(f.distro.as_ref())
        {
            return false;
        }
        if let Some(gpu) = &self.gpu
            && !gpu.matches(&f.gpus)
        {
            return false;
        }
        if let Some(container) = &self.container
            && !container.matches(f.container.as_ref())
        {
            return false;
        }
        if let Some(features) = &self.cpu_features
            && !features.matches(&f.cpu_features)
        {
            return false;
        }
        true
    }

    /// The band implied by which facts this predicate constrains: the highest-precedence band among
    /// the constrained axes. An empty predicate yields [`Band::Base`].
    pub fn inferred_band(&self) -> Band {
        let mut band = Band::Base;
        if self.os.is_some() {
            band = band.max(Band::Os);
        }
        if self.arch.is_some() {
            band = band.max(Band::Arch);
        }
        if self.distro.is_some() {
            band = band.max(Band::Distro);
        }
        if self.libc.is_some() || self.cpu_features.is_some() || self.gpu.is_some() {
            band = band.max(Band::Hardware);
        }
        if self.container.is_some() {
            band = band.max(Band::Container);
        }
        if self.hostname.is_some() {
            band = band.max(Band::Host);
        }
        band
    }
}

/// Match a distribution by `id` (also satisfied by `ID_LIKE`) and optionally `version`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DistroMatch {
    /// The distribution id, e.g. `ubuntu`. Also matches if listed in the machine's `ID_LIKE`.
    pub id: Option<String>,
    /// An exact `VERSION_ID` to require.
    pub version: Option<String>,
}

impl DistroMatch {
    fn matches(&self, distro: Option<&Distro>) -> bool {
        let Some(distro) = distro else {
            return false;
        };
        if let Some(id) = &self.id
            && &distro.id != id
            && !distro.like.iter().any(|l| l == id)
        {
            return false;
        }
        if let Some(version) = &self.version
            && distro.version.as_ref() != Some(version)
        {
            return false;
        }
        true
    }
}

/// Match a GPU by `vendor` and/or a case-insensitive `model` substring. Satisfied if *any* GPU on
/// the machine matches.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GpuMatch {
    /// Required vendor.
    pub vendor: Option<GpuVendor>,
    /// A case-insensitive substring the model must contain.
    pub model: Option<String>,
}

impl GpuMatch {
    fn matches(&self, gpus: &[Gpu]) -> bool {
        gpus.iter().any(|g| {
            self.vendor.as_ref().is_none_or(|v| &g.vendor == v)
                && self.model.as_ref().is_none_or(|m| {
                    g.model
                        .as_ref()
                        .is_some_and(|gm| gm.to_lowercase().contains(&m.to_lowercase()))
                })
        })
    }
}

/// Match a container environment. An empty table matches any container; `kind` narrows to one.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ContainerMatch {
    /// Require this specific container runtime.
    pub kind: Option<crate::facts::ContainerKind>,
}

impl ContainerMatch {
    fn matches(&self, container: Option<&Container>) -> bool {
        let Some(container) = container else {
            return false;
        };
        self.kind.as_ref().is_none_or(|k| &container.kind == k)
    }
}

/// Match against the machine's CPU feature set. `all` must all be present, `any` requires at least
/// one (when non-empty), and `none` must all be absent.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FeatureMatch {
    /// Every listed feature must be present.
    pub all: Vec<String>,
    /// At least one listed feature must be present (ignored when empty).
    pub any: Vec<String>,
    /// None of the listed features may be present.
    pub none: Vec<String>,
}

impl FeatureMatch {
    fn matches(&self, features: &BTreeSet<String>) -> bool {
        if !self.all.iter().all(|f| features.contains(f)) {
            return false;
        }
        if !self.any.is_empty() && !self.any.iter().any(|f| features.contains(f)) {
            return false;
        }
        if self.none.iter().any(|f| features.contains(f)) {
            return false;
        }
        true
    }
}

/// A platform definition as written in a grimoire — the table body under `[platform.<name>]`. The
/// name is the table key and is supplied separately by the loader.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformDef {
    /// The match predicate.
    #[serde(rename = "match", default)]
    pub predicate: Match,
    /// An explicit band, overriding the one inferred from the predicate.
    #[serde(default)]
    pub band: Option<Band>,
    /// A signed nudge to the precedence score within the band.
    #[serde(default)]
    pub weight: i64,
    /// Platforms this one refines (is more specific than). Used to break precedence ties.
    #[serde(default)]
    pub refines: Vec<String>,
}

/// A resolved platform: a [`PlatformDef`] with its name and effective band filled in.
#[derive(Clone, Debug)]
pub struct Platform {
    /// The platform's name (the table key in the grimoire).
    pub name: String,
    /// The match predicate.
    pub predicate: Match,
    /// The effective band (explicit, or inferred from the predicate).
    pub band: Band,
    /// The within-band precedence nudge.
    pub weight: i64,
    /// Platforms this one refines.
    pub refines: Vec<String>,
}

impl Platform {
    /// Resolve a definition into a named platform, inferring the band if none was given.
    pub fn from_def(name: impl Into<String>, def: PlatformDef) -> Self {
        let band = def.band.unwrap_or_else(|| def.predicate.inferred_band());
        Platform {
            name: name.into(),
            predicate: def.predicate,
            band,
            weight: def.weight,
            refines: def.refines,
        }
    }

    /// The precedence score: band base plus weight. Higher wins.
    pub fn score(&self) -> i64 {
        self.band.base() + self.weight
    }

    /// Whether this platform matches the given machine.
    pub fn matches(&self, facts: &Facts) -> bool {
        self.predicate.matches(facts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::{ContainerKind, GpuVendor};

    fn linux_facts() -> Facts {
        Facts::new(Os::Linux, Arch::X86_64)
            .with_hostname("x1")
            .with_features(["avx2", "fma"])
    }

    #[test]
    fn os_and_arch_constraints() {
        let m = Match {
            os: Some(Os::Linux),
            ..Default::default()
        };
        assert!(m.matches(&linux_facts()));
        let m = Match {
            os: Some(Os::Macos),
            ..Default::default()
        };
        assert!(!m.matches(&linux_facts()));
    }

    #[test]
    fn feature_constraints() {
        let f = linux_facts();
        assert!(
            FeatureMatch {
                all: vec!["avx2".into()],
                ..Default::default()
            }
            .matches(&f.cpu_features)
        );
        assert!(
            !FeatureMatch {
                all: vec!["avx512f".into()],
                ..Default::default()
            }
            .matches(&f.cpu_features)
        );
        assert!(
            !FeatureMatch {
                none: vec!["avx2".into()],
                ..Default::default()
            }
            .matches(&f.cpu_features)
        );
        assert!(
            FeatureMatch {
                any: vec!["sse2".into(), "fma".into()],
                ..Default::default()
            }
            .matches(&f.cpu_features)
        );
    }

    #[test]
    fn gpu_and_container_constraints() {
        let f = Facts::new(Os::Linux, Arch::X86_64)
            .with_gpu(Gpu {
                vendor: GpuVendor::Nvidia,
                model: Some("RTX 4090".into()),
            })
            .with_container(Container {
                kind: ContainerKind::Docker,
            });
        assert!(
            Match {
                gpu: Some(GpuMatch {
                    vendor: Some(GpuVendor::Nvidia),
                    ..Default::default()
                }),
                ..Default::default()
            }
            .matches(&f)
        );
        assert!(
            Match {
                gpu: Some(GpuMatch {
                    vendor: Some(GpuVendor::Nvidia),
                    model: Some("4090".into())
                }),
                ..Default::default()
            }
            .matches(&f)
        );
        assert!(
            Match {
                container: Some(ContainerMatch::default()),
                ..Default::default()
            }
            .matches(&f)
        );
    }

    #[test]
    fn distro_id_like() {
        let f = Facts::new(Os::Linux, Arch::X86_64).with_distro(Distro {
            id: "ubuntu".into(),
            version: Some("24.04".into()),
            like: vec!["debian".into()],
        });
        assert!(
            Match {
                distro: Some(DistroMatch {
                    id: Some("ubuntu".into()),
                    ..Default::default()
                }),
                ..Default::default()
            }
            .matches(&f)
        );
        // matches via ID_LIKE
        assert!(
            Match {
                distro: Some(DistroMatch {
                    id: Some("debian".into()),
                    ..Default::default()
                }),
                ..Default::default()
            }
            .matches(&f)
        );
        assert!(
            !Match {
                distro: Some(DistroMatch {
                    id: Some("fedora".into()),
                    ..Default::default()
                }),
                ..Default::default()
            }
            .matches(&f)
        );
    }

    #[test]
    fn band_inference() {
        let band = |m: Match| m.inferred_band();
        assert_eq!(band(Match::default()), Band::Base);
        assert_eq!(
            band(Match {
                os: Some(Os::Linux),
                ..Default::default()
            }),
            Band::Os
        );
        assert_eq!(
            band(Match {
                hostname: Some("x1".into()),
                ..Default::default()
            }),
            Band::Host
        );
        assert_eq!(
            band(Match {
                gpu: Some(GpuMatch {
                    vendor: Some(GpuVendor::Nvidia),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            Band::Hardware
        );
        // most-specific axis wins: os + hostname -> host
        assert_eq!(
            band(Match {
                os: Some(Os::Linux),
                hostname: Some("x1".into()),
                ..Default::default()
            }),
            Band::Host
        );
        assert_eq!(
            band(Match {
                container: Some(ContainerMatch::default()),
                ..Default::default()
            }),
            Band::Container
        );
    }

    #[test]
    fn deserialize_platform_def_infers_band() {
        let def: PlatformDef =
            toml::from_str(r#"match = { gpu = { vendor = "nvidia" } }"#).unwrap();
        let p = Platform::from_def("cuda", def);
        assert_eq!(p.band, Band::Hardware);
        assert_eq!(p.score(), 400);
    }

    #[test]
    fn deserialize_rejects_unknown_match_key() {
        // a misspelled predicate key must be a hard error, not a silently-ignored constraint
        let err = toml::from_str::<PlatformDef>(r#"match = { osx = "macos" }"#);
        assert!(err.is_err());
    }
}
