//! Headless core of the mongo-raabit MongoDB client.
//!
//! No UI dependencies: the egui views and the CLI drive everything through this crate.
//! All network operations are `async` on Tokio; synchronous
//! hosts spawn them on their own Tokio runtime.

pub mod connection;
pub mod error;
pub mod profile;
pub mod query;
pub mod secrets;
pub mod shell;
pub mod tunnel;

pub use connection::Connection;
pub use error::{RaabitError, Result};
pub use mongodb::{self, bson};
