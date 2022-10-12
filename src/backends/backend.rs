//! Definition of the backend interface

pub struct Source {
  /// If owned,
  ownership: Owner,

  backend: Backend,
}

/// Who controls the backend
pub enum Owner {
  /// All requests will come through this instance.
  Solo,

  /// Multiple users can connect to the
  Shared,

  ReadOnly,
}

// pub trait Backend {
// }
/*
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
 */
