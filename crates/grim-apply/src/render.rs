//! Template rendering context.
//!
//! Templates are MiniJinja (Jinja2-compatible). The render context exposes the machine and the
//! resolved platform stack, so a template can branch on facts and on which platforms are active:
//!
//! ```jinja
//! editor = {% if "workstation" in platforms %}nvim{% else %}vim{% endif %}
//! # host: {{ facts.hostname }} ({{ facts.os }}/{{ facts.arch }})
//! ```

use std::path::Path;

use grim_core::Facts;
use minijinja::{Environment, context};

use crate::error::ApplyError;

/// The data a template can see: the detected [`Facts`], the active platform names (highest
/// precedence first), and the isolation id.
#[derive(Debug, Clone)]
pub struct RenderContext {
    facts: Facts,
    platforms: Vec<String>,
}

impl RenderContext {
    /// Build a render context from the machine facts and the resolved active platform names.
    pub fn new(facts: Facts, platforms: Vec<String>) -> Self {
        RenderContext { facts, platforms }
    }

    /// Render a template string. `source_path` is used only for error context.
    pub fn render(&self, source_path: &Path, template: &str) -> Result<String, ApplyError> {
        let mut env = Environment::new();
        // Config files should end with a newline; don't let Jinja's default strip it.
        env.set_keep_trailing_newline(true);
        env.render_str(
            template,
            context! {
                facts => &self.facts,
                platforms => &self.platforms,
                isolation_id => self.facts.isolation_id(),
            },
        )
        .map_err(|source| ApplyError::Render {
            path: source_path.to_path_buf(),
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use grim_core::{Arch, Os};

    fn ctx() -> RenderContext {
        let facts = Facts::new(Os::Linux, Arch::X86_64).with_hostname("x1");
        RenderContext::new(
            facts,
            vec!["workstation".into(), "cuda".into(), "linux".into()],
        )
    }

    #[test]
    fn renders_facts_and_platform_membership() {
        let out = ctx()
            .render(
                Path::new("t"),
                "os={{ facts.os }} host={{ facts.hostname }} cuda={{ \"cuda\" in platforms }}",
            )
            .unwrap();
        assert_eq!(out, "os=linux host=x1 cuda=true");
    }

    #[test]
    fn conditional_on_platform() {
        let tmpl = "{% if \"workstation\" in platforms %}nvim{% else %}vim{% endif %}";
        assert_eq!(ctx().render(Path::new("t"), tmpl).unwrap(), "nvim");
    }

    #[test]
    fn render_error_is_reported_with_path() {
        let err = ctx().render(Path::new("bad"), "{{ unclosed ").unwrap_err();
        assert!(matches!(err, ApplyError::Render { .. }));
    }
}
