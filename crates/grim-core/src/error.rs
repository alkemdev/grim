//! Error types for `grim-core`.

use thiserror::Error;

/// Errors raised while resolving the active platform stack for a machine.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ResolveError {
    /// A platform's `refines` list names a platform that doesn't exist.
    #[error("platform `{platform}` refines unknown platform `{target}`")]
    UnknownRefinement { platform: String, target: String },

    /// The `refines` relation contains a cycle through this platform.
    #[error("platform `{platform}` participates in a `refines` cycle")]
    PlatformCycle { platform: String },

    /// Two platforms match the machine at the same precedence and nothing
    /// (`weight` or `refines`) breaks the tie. Resolution refuses to guess.
    #[error(
        "platforms `{a}` and `{b}` both match at precedence {score}; \
         add a `weight` or a `refines` relation to disambiguate"
    )]
    AmbiguousPrecedence { a: String, b: String, score: i64 },
}
