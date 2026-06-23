//! Resolving the active platform stack for a machine, and resolving layered values against it.
//!
//! Given the platforms declared in a grimoire and the machine's [`Facts`], [`resolve_stack`] selects
//! the platforms that match and orders them by precedence (highest first). The result is a path down
//! the platform lattice; resolving any layered value is then a fold along that path.

use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use crate::error::ResolveError;
use crate::facts::Facts;
use crate::platform::Platform;

/// The platforms that apply to a machine, ordered highest-precedence first.
#[derive(Debug)]
pub struct ActiveStack<'p> {
    platforms: Vec<&'p Platform>,
}

impl<'p> ActiveStack<'p> {
    /// The matching platforms, highest precedence first.
    pub fn platforms(&self) -> &[&'p Platform] {
        &self.platforms
    }

    /// The names of the matching platforms, highest precedence first. Handy for `--explain` output.
    pub fn names(&self) -> Vec<&'p str> {
        self.platforms.iter().map(|p| p.name.as_str()).collect()
    }

    /// Whether a platform of the given name is in the active stack.
    pub fn contains(&self, name: &str) -> bool {
        self.platforms.iter().any(|p| p.name == name)
    }

    /// Resolve a scalar value keyed by platform name: the contribution from the highest-precedence
    /// platform present in `map` wins; the rest are shadowed.
    pub fn resolve_override<'a, T>(&self, map: &'a BTreeMap<String, T>) -> Option<&'a T> {
        self.platforms.iter().find_map(|p| map.get(&p.name))
    }

    /// Resolve a collection value keyed by platform name: contributions from every matching platform
    /// are merged, applied low-precedence first so higher-precedence platforms override on
    /// collisions. The merge step is supplied by the caller.
    pub fn resolve_merged<'a, T, A>(
        &self,
        map: &'a BTreeMap<String, T>,
        mut acc: A,
        mut merge: impl FnMut(&mut A, &'a T),
    ) -> A {
        for p in self.platforms.iter().rev() {
            if let Some(value) = map.get(&p.name) {
                merge(&mut acc, value);
            }
        }
        acc
    }
}

/// Resolve the active platform stack for `facts` from the full set of declared `platforms`.
///
/// Platforms are ordered by precedence score (descending). Ties are broken by the `refines`
/// relation; two platforms that match at the same score with no `refines` relation between them are
/// an [`ResolveError::AmbiguousPrecedence`] error rather than a silent guess.
pub fn resolve_stack<'p>(
    platforms: &'p [Platform],
    facts: &Facts,
) -> Result<ActiveStack<'p>, ResolveError> {
    let closure = refines_closure(platforms)?;

    let mut matching: Vec<&Platform> = platforms.iter().filter(|p| p.matches(facts)).collect();
    // Stable sort by score descending; equal-score groups are disambiguated next.
    matching.sort_by_key(|p| Reverse(p.score()));

    let ordered = order_within_ties(matching, &closure)?;
    Ok(ActiveStack { platforms: ordered })
}

/// Build the transitive `refines` closure: each platform name mapped to the set of names it
/// transitively refines. Validates that every `refines` target exists and that there are no cycles.
fn refines_closure(
    platforms: &[Platform],
) -> Result<HashMap<String, HashSet<String>>, ResolveError> {
    let names: HashSet<&str> = platforms.iter().map(|p| p.name.as_str()).collect();
    for p in platforms {
        for target in &p.refines {
            if !names.contains(target.as_str()) {
                return Err(ResolveError::UnknownRefinement {
                    platform: p.name.clone(),
                    target: target.clone(),
                });
            }
        }
    }

    let direct: HashMap<&str, &[String]> = platforms
        .iter()
        .map(|p| (p.name.as_str(), p.refines.as_slice()))
        .collect();

    let mut closure = HashMap::with_capacity(platforms.len());
    for p in platforms {
        let mut reachable: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<&str> = p.refines.iter().map(String::as_str).collect();
        while let Some(node) = queue.pop_front() {
            if !reachable.insert(node.to_string()) {
                continue;
            }
            if let Some(next) = direct.get(node) {
                queue.extend(next.iter().map(String::as_str));
            }
        }
        if reachable.contains(&p.name) {
            return Err(ResolveError::PlatformCycle {
                platform: p.name.clone(),
            });
        }
        closure.insert(p.name.clone(), reachable);
    }
    Ok(closure)
}

/// Given platforms already sorted by score descending, order each equal-score group by `refines`
/// (a platform that refines another is more specific, so it comes first). A group whose members are
/// not all pairwise comparable under `refines` is ambiguous.
fn order_within_ties<'p>(
    sorted: Vec<&'p Platform>,
    closure: &HashMap<String, HashSet<String>>,
) -> Result<Vec<&'p Platform>, ResolveError> {
    let refines = |a: &Platform, b: &Platform| closure[&a.name].contains(&b.name);

    let mut result = Vec::with_capacity(sorted.len());
    let mut i = 0;
    while i < sorted.len() {
        let score = sorted[i].score();
        let mut j = i;
        while j < sorted.len() && sorted[j].score() == score {
            j += 1;
        }
        let mut group: Vec<&Platform> = sorted[i..j].to_vec();
        if group.len() > 1 {
            // Every pair must be comparable under `refines`, or the precedence is ambiguous.
            for a in 0..group.len() {
                for b in (a + 1)..group.len() {
                    if !refines(group[a], group[b]) && !refines(group[b], group[a]) {
                        return Err(ResolveError::AmbiguousPrecedence {
                            a: group[a].name.clone(),
                            b: group[b].name.clone(),
                            score,
                        });
                    }
                }
            }
            group.sort_by(|a, b| {
                if refines(a, b) {
                    Ordering::Less
                } else if refines(b, a) {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            });
        }
        result.extend(group);
        i = j;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::{Arch, Container, ContainerKind, Gpu, GpuVendor, Os};
    use crate::platform::{ContainerMatch, GpuMatch, Match, PlatformDef};

    fn plat(name: &str, predicate: Match) -> Platform {
        Platform::from_def(
            name,
            PlatformDef {
                predicate,
                band: None,
                weight: 0,
                refines: vec![],
            },
        )
    }

    fn plat_refines(name: &str, predicate: Match, refines: &[&str]) -> Platform {
        Platform::from_def(
            name,
            PlatformDef {
                predicate,
                band: None,
                weight: 0,
                refines: refines.iter().map(|s| s.to_string()).collect(),
            },
        )
    }

    fn workstation() -> Facts {
        Facts::new(Os::Linux, Arch::X86_64)
            .with_hostname("x1")
            .with_gpu(Gpu {
                vendor: GpuVendor::Nvidia,
                model: None,
            })
    }

    #[test]
    fn orders_by_band_precedence() {
        let platforms = vec![
            plat(
                "linux",
                Match {
                    os: Some(Os::Linux),
                    ..Default::default()
                },
            ),
            plat(
                "cuda",
                Match {
                    gpu: Some(GpuMatch {
                        vendor: Some(GpuVendor::Nvidia),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ),
            plat(
                "x1",
                Match {
                    hostname: Some("x1".into()),
                    ..Default::default()
                },
            ),
        ];
        let stack = resolve_stack(&platforms, &workstation()).unwrap();
        assert_eq!(stack.names(), vec!["x1", "cuda", "linux"]);
    }

    #[test]
    fn container_tops_the_stack() {
        let facts = workstation().with_container(Container {
            kind: ContainerKind::Devcontainer,
        });
        let platforms = vec![
            plat(
                "linux",
                Match {
                    os: Some(Os::Linux),
                    ..Default::default()
                },
            ),
            plat(
                "x1",
                Match {
                    hostname: Some("x1".into()),
                    ..Default::default()
                },
            ),
            plat(
                "dev",
                Match {
                    container: Some(ContainerMatch::default()),
                    ..Default::default()
                },
            ),
        ];
        let stack = resolve_stack(&platforms, &facts).unwrap();
        assert_eq!(stack.names().first(), Some(&"dev"));
    }

    #[test]
    fn override_resolution_picks_highest() {
        let platforms = vec![
            plat(
                "linux",
                Match {
                    os: Some(Os::Linux),
                    ..Default::default()
                },
            ),
            plat(
                "x1",
                Match {
                    hostname: Some("x1".into()),
                    ..Default::default()
                },
            ),
        ];
        let stack = resolve_stack(&platforms, &workstation()).unwrap();
        let mut editors = BTreeMap::new();
        editors.insert("linux".to_string(), "vim");
        editors.insert("x1".to_string(), "nvim");
        assert_eq!(stack.resolve_override(&editors), Some(&"nvim"));
    }

    #[test]
    fn merge_resolution_unions_low_to_high() {
        let platforms = vec![
            plat(
                "linux",
                Match {
                    os: Some(Os::Linux),
                    ..Default::default()
                },
            ),
            plat(
                "x1",
                Match {
                    hostname: Some("x1".into()),
                    ..Default::default()
                },
            ),
        ];
        let stack = resolve_stack(&platforms, &workstation()).unwrap();
        let mut pkgs: BTreeMap<String, Vec<&str>> = BTreeMap::new();
        pkgs.insert("linux".into(), vec!["ripgrep"]);
        pkgs.insert("x1".into(), vec!["cuda-toolkit"]);
        let merged =
            stack.resolve_merged(&pkgs, Vec::new(), |acc, v| acc.extend(v.iter().copied()));
        // low precedence first, then high
        assert_eq!(merged, vec!["ripgrep", "cuda-toolkit"]);
    }

    #[test]
    fn equal_precedence_without_refines_is_ambiguous() {
        let platforms = vec![
            plat(
                "cuda",
                Match {
                    gpu: Some(GpuMatch {
                        vendor: Some(GpuVendor::Nvidia),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ),
            plat(
                "avx",
                Match {
                    cpu_features: Some(crate::platform::FeatureMatch {
                        all: vec!["avx2".into()],
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ),
        ];
        let facts = workstation().with_features(["avx2"]);
        let err = resolve_stack(&platforms, &facts).unwrap_err();
        assert!(matches!(
            err,
            ResolveError::AmbiguousPrecedence { score: 400, .. }
        ));
    }

    #[test]
    fn refines_breaks_the_tie() {
        let platforms = vec![
            plat(
                "avx",
                Match {
                    cpu_features: Some(crate::platform::FeatureMatch {
                        all: vec!["avx2".into()],
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ),
            plat_refines(
                "cuda",
                Match {
                    gpu: Some(GpuMatch {
                        vendor: Some(GpuVendor::Nvidia),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                &["avx"],
            ),
        ];
        let facts = workstation().with_features(["avx2"]);
        let stack = resolve_stack(&platforms, &facts).unwrap();
        // cuda refines avx, so cuda is more specific and wins the tie
        assert_eq!(stack.names(), vec!["cuda", "avx"]);
    }

    #[test]
    fn unknown_refinement_errors() {
        let platforms = vec![plat_refines(
            "a",
            Match {
                os: Some(Os::Linux),
                ..Default::default()
            },
            &["ghost"],
        )];
        let err = resolve_stack(&platforms, &workstation()).unwrap_err();
        assert_eq!(
            err,
            ResolveError::UnknownRefinement {
                platform: "a".into(),
                target: "ghost".into()
            }
        );
    }

    #[test]
    fn refines_cycle_errors() {
        let platforms = vec![
            plat_refines(
                "a",
                Match {
                    os: Some(Os::Linux),
                    ..Default::default()
                },
                &["b"],
            ),
            plat_refines(
                "b",
                Match {
                    os: Some(Os::Linux),
                    ..Default::default()
                },
                &["a"],
            ),
        ];
        let err = resolve_stack(&platforms, &workstation()).unwrap_err();
        assert!(matches!(err, ResolveError::PlatformCycle { .. }));
    }
}
