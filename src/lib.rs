//! Personal portfolio application library.
//!
//! The browser binary remains deliberately thin. Public domain and
//! configuration modules provide stable boundaries for the external test suite
//! in `tests/`.

pub mod app;
pub mod components;
pub mod content;
pub mod cv;
pub mod cv_presentation;
#[cfg(not(target_arch = "wasm32"))]
pub mod cv_sync;
pub mod generated_cv;
pub mod generated_projects;
pub mod pages;
#[cfg(not(target_arch = "wasm32"))]
pub mod project_sync;
pub mod projects;
pub mod routes;
