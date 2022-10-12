//! A graph query language processor
//!
//! This allows translations between various syntaxes (GraphQL, OpenCypher), a builder
//! for generating queries programattically, and an Abstract Syntax Tree for building querying one's
//! own graphs.

/// Cypher Errors
pub mod errors;

// pub mod backends;

// pub mod connection;

pub mod model;

pub mod data;

pub mod grapht;

// AST for building queries
pub mod gquery;

// Subscription messaging
// pub mod messages;

// ASTs for some known query languages
pub mod grammars;

pub mod prelude {
  pub use crate::{
    // backends,
    // connection::Pool,
    data::{dataset::*, stats::*},
    err,
    err_into,
    errors::{GraphtError, Kind, Result as GraphtResult},
    grapht::Grapht,
    model::*,
  };
}

/// Toggles std and alternate implementations that are used throughout the crate
mod local {
  pub use crate::prelude::*;
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
