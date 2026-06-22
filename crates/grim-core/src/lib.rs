//! Core types and resolution logic for `grim`.
//!
//! This crate is the pure, side-effect-free heart of the engine:
//!
//! - [`Facts`] — the current machine, probed once into a typed struct.
//! - [`Platform`] / [`Match`] — named, precedence-ordered layers matched against facts.
//! - [`resolve_stack`] — selecting and ordering the platforms that apply, then folding layered
//!   values along that order.
//!
//! Detection ([`Facts::detect`]) is the only machine-dependent code; everything else is a pure
//! function of [`Facts`] plus the grimoire, and is tested without touching the filesystem, the
//! network, or any package manager.

pub mod error;
pub mod facts;
pub mod platform;
pub mod resolve;

pub use error::ResolveError;
pub use facts::{Arch, Container, ContainerKind, Distro, Facts, Gpu, GpuVendor, Libc, Os};
pub use platform::{
    Band, ContainerMatch, DistroMatch, FeatureMatch, GpuMatch, Match, Platform, PlatformDef,
};
pub use resolve::{ActiveStack, resolve_stack};

/// The name of the engine.
pub const NAME: &str = "grim";
