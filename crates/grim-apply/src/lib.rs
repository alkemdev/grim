//! grim's file-templating and placement engine.

pub mod engine;
pub mod error;
pub mod name;
pub mod render;

pub use engine::{Body, Change, FileAction, Plan, execute, plan};
pub use error::ApplyError;
pub use name::{ModeAttr, NamePart, parse_component};
pub use render::RenderContext;
