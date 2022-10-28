//! A graph query language processor
//!
//! This allows translations between various syntaxes (GraphQL, OpenCypher), a builder
//! for generating queries programattically, and an Abstract Syntax Tree for building querying one's
//! own graphs.

// Cypher Errors
pub mod errors;

// Getting the difference between to objects
pub mod utils;

// Graph objects and their components
pub mod model;

// Statistics about the state of data
pub mod stats;

// In-memory cache of queryable data
pub mod store;

// Transport layer for communicating with remote servers
// pub mod connection;

// Drivers for remote sources
// pub mod backends;

// The module responsible for synthesizing and synchronizing non-local data
// pub mod grapht;

// The query language syntax tree
// pub mod gquery;

// Subscription messaging
// pub mod messages;

// ASTs for some known query languages
// pub mod grammars;

pub mod prelude {
  pub use crate::{
    err,
    err_into,
    errors::{GraphtError, Kind, Result as GraphtResult},
    // grapht::Grapht,
    model::*,
    stats::*,
    store::{dataset::*, index::*, value::*},
    // backends,
    // connection::Pool,
    utils::*,
  };
}

/// Toggles std and alternate implementations that are used throughout the crate
mod local {
  // pub use crate::prelude::*;
  pub use crate::{err as grapht_err, err_into};

  // Prefer using core vs std
  pub use core::{
    fmt::{self, Debug, Display},
    str,
  };

  // Std vs Tokio sync
  pub use std::sync;

  pub use tracing::{debug, error, info, trace, warn};
}
