//! Personal portfolio application library.
//!
//! The browser binary remains deliberately thin. Public domain and
//! configuration modules provide stable boundaries for the external test suite
//! in `tests/`.

pub mod app;
mod components;
pub mod content;
#[cfg(not(target_arch = "wasm32"))]
pub mod cv_sync;
mod pages;
pub mod routes;
