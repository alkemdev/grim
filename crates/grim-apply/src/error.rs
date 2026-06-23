//! Error type for the apply engine.

use std::path::PathBuf;

use thiserror::Error;

/// Errors raised while planning or executing an apply.
#[derive(Debug, Error)]
pub enum ApplyError {
    /// A source directory could not be walked.
    #[error("walking source `{path}`: {source}")]
    Walk {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// A source file could not be read.
    #[error("reading source `{path}`: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// A `.tmpl` source was not valid UTF-8 and so cannot be rendered.
    #[error("source `{path}` is a template but is not valid UTF-8")]
    NonUtf8Template { path: PathBuf },

    /// A template failed to render.
    #[error("rendering template `{path}`: {source}")]
    Render {
        path: PathBuf,
        #[source]
        source: minijinja::Error,
    },

    /// A target path could not be written.
    #[error("writing `{path}`: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
