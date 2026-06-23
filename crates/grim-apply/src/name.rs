//! The source→target naming convention.
//!
//! A grimoire's `files/` tree mirrors the target tree, with a small, explicit set of attributes
//! encoded in path-component names. Parsing is pure and applies per path component:
//!
//! | source component        | effect                                            |
//! | ----------------------- | ------------------------------------------------- |
//! | `dot_foo`               | target name `.foo` (works for files and dirs)     |
//! | `executable_foo`        | file mode `0755`                                  |
//! | `private_foo`           | file mode `0600` (dir mode `0700`)                |
//! | `foo.tmpl`              | rendered as a template; target name `foo`         |
//!
//! Attributes combine in the order `[executable_|private_] dot_ name [.tmpl]`, e.g.
//! `private_dot_ssh` → `.ssh` (mode `0700`), `executable_dot_foo.tmpl` → `.foo` (mode `0755`,
//! templated).

/// The mode attribute encoded in a source name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeAttr {
    /// No explicit mode.
    None,
    /// `executable_` — file mode `0755`.
    Executable,
    /// `private_` — file mode `0600`, directory mode `0700`.
    Private,
}

impl ModeAttr {
    /// The file mode this attribute implies, if any.
    pub fn file_mode(self) -> Option<u32> {
        match self {
            ModeAttr::Executable => Some(0o755),
            ModeAttr::Private => Some(0o600),
            ModeAttr::None => None,
        }
    }

    /// The directory mode this attribute implies, if any.
    pub fn dir_mode(self) -> Option<u32> {
        match self {
            ModeAttr::Private => Some(0o700),
            _ => None,
        }
    }
}

/// The result of parsing one source path component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamePart {
    /// The target component name (with attribute prefixes stripped and `dot_` applied).
    pub target: String,
    /// The mode attribute, if any.
    pub attr: ModeAttr,
    /// Whether this component is a template (`.tmpl` suffix), to be rendered.
    pub is_template: bool,
}

/// Parse a single source path component into its target name and attributes.
pub fn parse_component(name: &str) -> NamePart {
    let mut attr = ModeAttr::None;
    let mut rest = name;

    if let Some(s) = rest.strip_prefix("executable_") {
        attr = ModeAttr::Executable;
        rest = s;
    } else if let Some(s) = rest.strip_prefix("private_") {
        attr = ModeAttr::Private;
        rest = s;
    }

    let mut prefix = "";
    if let Some(s) = rest.strip_prefix("dot_") {
        prefix = ".";
        rest = s;
    }

    let mut is_template = false;
    if let Some(s) = rest.strip_suffix(".tmpl") {
        is_template = true;
        rest = s;
    }

    NamePart {
        target: format!("{prefix}{rest}"),
        attr,
        is_template,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn part(name: &str) -> (String, ModeAttr, bool) {
        let p = parse_component(name);
        (p.target, p.attr, p.is_template)
    }

    #[test]
    fn plain_name() {
        assert_eq!(
            part("config.toml"),
            ("config.toml".into(), ModeAttr::None, false)
        );
    }

    #[test]
    fn dot_prefix() {
        assert_eq!(part("dot_zshrc"), (".zshrc".into(), ModeAttr::None, false));
        assert_eq!(
            part("dot_config"),
            (".config".into(), ModeAttr::None, false)
        );
    }

    #[test]
    fn template_suffix() {
        assert_eq!(
            part("gitconfig.tmpl"),
            ("gitconfig".into(), ModeAttr::None, true)
        );
        assert_eq!(
            part("dot_gitconfig.tmpl"),
            (".gitconfig".into(), ModeAttr::None, true)
        );
    }

    #[test]
    fn mode_attributes_combine() {
        assert_eq!(
            part("executable_dot_foo"),
            (".foo".into(), ModeAttr::Executable, false)
        );
        assert_eq!(
            part("private_dot_ssh"),
            (".ssh".into(), ModeAttr::Private, false)
        );
        assert_eq!(
            part("executable_dot_script.tmpl"),
            (".script".into(), ModeAttr::Executable, true)
        );
    }

    #[test]
    fn modes_map_correctly() {
        assert_eq!(ModeAttr::Executable.file_mode(), Some(0o755));
        assert_eq!(ModeAttr::Private.file_mode(), Some(0o600));
        assert_eq!(ModeAttr::Private.dir_mode(), Some(0o700));
        assert_eq!(ModeAttr::None.file_mode(), None);
    }
}
