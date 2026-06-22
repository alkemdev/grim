//! The machine, probed once into a typed struct.
//!
//! [`Facts`] is the single machine-dependent input to the engine; everything downstream is a pure
//! function of it (plus the grimoire). Detection is isolated in [`Facts::detect`]; the parsing it
//! relies on lives in small pure helpers ([`parse_cpuinfo_flags`], [`parse_os_release`],
//! [`classify_container`]) so it can be unit-tested without touching the real machine.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// Defines a string-backed enum with a catch-all `Other(String)` variant, plus `as_str`, the
/// `From`/`Display` conversions, and string-based serde. Keeps the typed ergonomics of an enum
/// while never failing on an unknown value from a new OS, CPU, or GPU.
macro_rules! string_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident => $s:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub enum $name {
            $($variant,)+
            /// An unrecognized value, preserved verbatim.
            Other(String),
        }

        impl $name {
            /// The canonical lowercase string for this value.
            pub fn as_str(&self) -> &str {
                match self {
                    $(Self::$variant => $s,)+
                    Self::Other(s) => s.as_str(),
                }
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                match s {
                    $($s => Self::$variant,)+
                    other => Self::Other(other.to_string()),
                }
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self { Self::from(s.as_str()) }
        }

        impl From<$name> for String {
            fn from(v: $name) -> Self { v.as_str().to_string() }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
                ser.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
                Ok(Self::from(String::deserialize(de)?))
            }
        }
    };
}

string_enum! {
    /// Operating system family (matches Rust's `std::env::consts::OS` values).
    Os { Macos => "macos", Linux => "linux", Windows => "windows" }
}

string_enum! {
    /// CPU architecture (matches Rust's `std::env::consts::ARCH` values).
    Arch { Aarch64 => "aarch64", X86_64 => "x86_64" }
}

string_enum! {
    /// C standard library implementation (Linux).
    Libc { Glibc => "glibc", Musl => "musl" }
}

string_enum! {
    /// GPU vendor.
    GpuVendor { Nvidia => "nvidia", Amd => "amd", Apple => "apple", Intel => "intel" }
}

string_enum! {
    /// Container runtime the process is running inside.
    ContainerKind { Docker => "docker", Podman => "podman", Devcontainer => "devcontainer" }
}

/// A Linux distribution, as reported by `/etc/os-release`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Distro {
    /// The `ID` field, e.g. `ubuntu`, `fedora`, `arch`.
    pub id: String,
    /// The `VERSION_ID` field, e.g. `24.04`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub version: Option<String>,
    /// The `ID_LIKE` field, e.g. `["debian"]` — distributions this one is compatible with.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub like: Vec<String>,
}

/// A single GPU present on the machine.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gpu {
    /// The GPU vendor.
    pub vendor: GpuVendor,
    /// A model string if known, e.g. `RTX 4090`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub model: Option<String>,
}

/// The container environment, if the process is running inside one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Container {
    /// Which runtime.
    pub kind: ContainerKind,
}

/// The detected machine. The single machine-dependent input to resolution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Facts {
    /// Operating system family.
    pub os: Os,
    /// CPU architecture.
    pub arch: Arch,
    /// The machine's hostname.
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub hostname: String,
    /// C library implementation (Linux only).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub libc: Option<Libc>,
    /// CPU feature flags (e.g. `avx2`, `neon`).
    #[serde(skip_serializing_if = "BTreeSet::is_empty", default)]
    pub cpu_features: BTreeSet<String>,
    /// Linux distribution, if applicable.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub distro: Option<Distro>,
    /// GPUs present on the machine.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub gpus: Vec<Gpu>,
    /// Container environment, if any.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub container: Option<Container>,
}

impl Facts {
    /// An otherwise-empty fact set for a given OS and architecture. Builder-style `with_*` methods
    /// fill in the rest; primarily for constructing test inputs.
    pub fn new(os: Os, arch: Arch) -> Self {
        Facts {
            os,
            arch,
            hostname: String::new(),
            libc: None,
            cpu_features: BTreeSet::new(),
            distro: None,
            gpus: Vec::new(),
            container: None,
        }
    }

    /// Set the hostname.
    pub fn with_hostname(mut self, hostname: impl Into<String>) -> Self {
        self.hostname = hostname.into();
        self
    }

    /// Set the CPU feature flags.
    pub fn with_features<I, S>(mut self, features: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.cpu_features = features.into_iter().map(Into::into).collect();
        self
    }

    /// Set the distribution.
    pub fn with_distro(mut self, distro: Distro) -> Self {
        self.distro = Some(distro);
        self
    }

    /// Set the C library.
    pub fn with_libc(mut self, libc: Libc) -> Self {
        self.libc = Some(libc);
        self
    }

    /// Add a GPU.
    pub fn with_gpu(mut self, gpu: Gpu) -> Self {
        self.gpus.push(gpu);
        self
    }

    /// Set the container environment.
    pub fn with_container(mut self, container: Container) -> Self {
        self.container = Some(container);
        self
    }

    /// The default isolation id: the namespace under which per-platform install trees live, so one
    /// shared `$HOME` can serve heterogeneous machines. Currently `{os}-{arch}`; a microarch
    /// component will be added once feature-set → microarch labelling lands.
    pub fn isolation_id(&self) -> String {
        format!("{}-{}", self.os, self.arch)
    }

    /// Probe the current machine. The only machine-dependent entry point.
    pub fn detect() -> Self {
        let os = Os::from(std::env::consts::OS);
        let arch = Arch::from(std::env::consts::ARCH);
        let hostname = gethostname::gethostname().to_string_lossy().into_owned();
        Facts {
            os,
            arch,
            hostname,
            libc: detect_libc(),
            cpu_features: detect_cpu_features(),
            distro: detect_distro(),
            gpus: Vec::new(), // GPU detection lands in a later phase.
            container: detect_container(),
        }
    }
}

/// Parse the CPU feature flags out of `/proc/cpuinfo` text. x86 uses a `flags` line; ARM uses a
/// `Features` line. Tokens from every such line are unioned.
pub fn parse_cpuinfo_flags(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in text.lines() {
        let Some((key, val)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        if key == "flags" || key == "features" {
            out.extend(val.split_whitespace().map(str::to_string));
        }
    }
    out
}

/// Parse `/etc/os-release` text into a [`Distro`]. Returns `None` if no `ID` is present.
pub fn parse_os_release(text: &str) -> Option<Distro> {
    let mut id = None;
    let mut version = None;
    let mut like = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, val)) = line.split_once('=') else {
            continue;
        };
        let val = unquote(val.trim());
        match key.trim() {
            "ID" => id = Some(val.to_string()),
            "VERSION_ID" => version = Some(val.to_string()),
            "ID_LIKE" => like = val.split_whitespace().map(str::to_string).collect(),
            _ => {}
        }
    }
    id.map(|id| Distro { id, version, like })
}

fn unquote(s: &str) -> &str {
    s.trim_matches(['"', '\''])
}

/// Decide the container environment from a few independent signals. Pure so it can be tested
/// without a real container; [`detect_container`] wires it to the filesystem and environment.
pub fn classify_container(
    dockerenv: bool,
    containerenv: bool,
    env_container: Option<&str>,
    devcontainer: bool,
) -> Option<Container> {
    if devcontainer {
        return Some(Container {
            kind: ContainerKind::Devcontainer,
        });
    }
    if containerenv || env_container == Some("podman") {
        return Some(Container {
            kind: ContainerKind::Podman,
        });
    }
    if dockerenv || env_container == Some("docker") {
        return Some(Container {
            kind: ContainerKind::Docker,
        });
    }
    env_container.map(|k| Container {
        kind: ContainerKind::from(k),
    })
}

fn detect_cpu_features() -> BTreeSet<String> {
    if cfg!(target_os = "linux") {
        if let Ok(text) = std::fs::read_to_string("/proc/cpuinfo") {
            return parse_cpuinfo_flags(&text);
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(set) = macos_cpu_features() {
            return set;
        }
    }
    BTreeSet::new()
}

#[cfg(target_os = "macos")]
fn macos_cpu_features() -> Option<BTreeSet<String>> {
    let out = std::process::Command::new("sysctl")
        .args(["-n", "machdep.cpu.features", "machdep.cpu.leaf7_features"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let set: BTreeSet<String> = text
        .split_whitespace()
        .map(|t| t.to_ascii_lowercase())
        .collect();
    (!set.is_empty()).then_some(set)
}

fn detect_distro() -> Option<Distro> {
    let text = std::fs::read_to_string("/etc/os-release").ok()?;
    parse_os_release(&text)
}

fn detect_libc() -> Option<Libc> {
    if !cfg!(target_os = "linux") {
        return None;
    }
    for loader in ["/lib/ld-musl-x86_64.so.1", "/lib/ld-musl-aarch64.so.1"] {
        if std::path::Path::new(loader).exists() {
            return Some(Libc::Musl);
        }
    }
    Some(Libc::Glibc)
}

fn detect_container() -> Option<Container> {
    let env_container = std::env::var("container").ok();
    let devcontainer = std::env::var_os("REMOTE_CONTAINERS").is_some()
        || std::env::var_os("DEVCONTAINER").is_some()
        || std::env::var_os("CODESPACES").is_some();
    classify_container(
        std::path::Path::new("/.dockerenv").exists(),
        std::path::Path::new("/run/.containerenv").exists(),
        env_container.as_deref(),
        devcontainer,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpuinfo_x86_flags() {
        let text = "processor\t: 0\nflags\t\t: fpu vme de avx2 fma bmi2\nbogomips\t: 7000\n";
        let flags = parse_cpuinfo_flags(text);
        assert!(flags.contains("avx2"));
        assert!(flags.contains("fma"));
        assert!(!flags.contains("bogomips"));
    }

    #[test]
    fn cpuinfo_arm_features() {
        let text = "processor\t: 0\nFeatures\t: fp asimd evtstrm aes sha2\nCPU part\t: 0xd0c\n";
        let flags = parse_cpuinfo_flags(text);
        assert!(flags.contains("asimd"));
        assert!(flags.contains("aes"));
    }

    #[test]
    fn os_release_ubuntu() {
        let text = "NAME=\"Ubuntu\"\nID=ubuntu\nVERSION_ID=\"24.04\"\nID_LIKE=debian\n";
        let distro = parse_os_release(text).expect("distro");
        assert_eq!(distro.id, "ubuntu");
        assert_eq!(distro.version.as_deref(), Some("24.04"));
        assert_eq!(distro.like, vec!["debian".to_string()]);
    }

    #[test]
    fn os_release_no_id_is_none() {
        assert!(parse_os_release("NAME=\"Mystery\"\n").is_none());
    }

    #[test]
    fn container_classification() {
        assert_eq!(
            classify_container(true, false, None, false).map(|c| c.kind),
            Some(ContainerKind::Docker)
        );
        assert_eq!(
            classify_container(false, true, None, false).map(|c| c.kind),
            Some(ContainerKind::Podman)
        );
        // devcontainer signal wins over everything else.
        assert_eq!(
            classify_container(true, true, Some("podman"), true).map(|c| c.kind),
            Some(ContainerKind::Devcontainer)
        );
        assert_eq!(classify_container(false, false, None, false), None);
    }

    #[test]
    fn string_enum_roundtrip_and_unknown() {
        assert_eq!(Os::from("linux"), Os::Linux);
        assert_eq!(Os::Linux.as_str(), "linux");
        assert_eq!(Arch::from("riscv64"), Arch::Other("riscv64".to_string()));
        assert_eq!(Arch::Other("riscv64".to_string()).to_string(), "riscv64");
    }
}
