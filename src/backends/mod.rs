//! Drivers for communicating with various storage media
//!
//! Part of Grapht is to act as a data clearing-house. It aggregates data from multiple sources and
//! turns it into a queryable graph, so it must know how to speak multiple languages.

use super::prelude::*;
use crate::{local::*, prelude::*};

pub mod backend;
// pub use Backend;

pub trait Backend {
  type RawResponse;

  /// Get the name the backend knows itself as
  fn name(&self) -> String;

  // /// Get a parser for the backend's grammar
  // fn grammar() -> nomnomicon::Grammar {
  //   unimplemented!("Backend Grammar")
  // }

  /// Sends raw messages to the backend and returns the minimally processed result
  fn send(&mut self, msg: &str) -> GraphtResult<Self::RawResponse> {
    info!("Sending message to {}:\n{}", self.name(), msg);
    unimplemented!("{}::Backend Send", self.name())
  }

  /// Send a query to the backend
  fn query(&mut self, gquery: GQuery) -> GraphtResult<QuerySet> {
    unimplemented!("{}::Backend query not implemented", self.name());
    // Convert GQuery to string
    // Send the raw value to the server
    // Parse the result
  }

  /// Receive process a RawResponse and add it to the query set
  fn parse(&mut self, query_set: &mut QuerySet, value: Self::RawResponse) -> GraphtResult<()>;

  /// Convert a GQuery into the raw message using a grammar
  fn translate(query: GQuery) -> String {
    unimplemented!("Backend grammar not set");
  }

  /// Get statistics (nodes, edges, paths, indices, etc.)
  fn stats(&mut self) -> GraphtResult<()> {
    unimplemented!("Backend Query")
  }
}

/// A driver for communicating with a specific type of data source
// pub trait Backend {
// Convert to and from a GQuery
// type QueryGrammar;

// Convert responses into log and data elements
// type NodeGrammar;
// }

pub mod redis_graph;
