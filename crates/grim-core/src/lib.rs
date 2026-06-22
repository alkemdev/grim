//! Core types and resolution logic for `grim`.
//!
//! This crate is the pure, side-effect-free heart of the engine. It will hold:
//!
//! - **Facts** — the current machine, probed once into a typed struct.
//! - **Platforms** — named, precedence-ordered layers matched against facts.
//! - **Resolution** — selecting/merging layered values along the active platform stack.
//! - **Manifests** — the typed model of a grimoire (packages, files, secrets, hosts).
//!
//! Everything here is designed to be unit-testable without touching the filesystem,
//! the network, or any package manager. I/O lives in the sibling crates.

/// The name of the engine.
pub const NAME: &str = "grim";
