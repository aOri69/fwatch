//! Provides an configuration structure and application structure.
//!
//! - [Config]
//! - [App]
//!
//! [Config]: crate::config::Config
//! [App]: crate::app::App

// Lints
#![deny(missing_docs, rustdoc::missing_crate_level_docs, rustdoc::broken_intra_doc_links)]
#![warn(clippy::missing_docs_in_private_items)]

mod app;
mod config;

pub use app::*;
pub use config::*;
