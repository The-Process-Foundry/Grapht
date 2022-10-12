//! A generic adaptor for sending and receiving messages
//!
//! THINK: Is this Wrapi (an generic API communication wrapper). Likely another separate module
//! FUTURE:
//!   - Connection Pooling (Parallell send/receive)

use std::collections::HashMap;
use uuid::Uuid;

pub use crate::local::*;

mod api_endpoint;
pub use api_endpoint::ApiEndpoint;
mod socket_endpoint;
pub use socket_endpoint::SocketEndpoint;
mod listener_endpoint;
pub use listener_endpoint::ListenerEndpoint;

/// All the connections known and managed by the system
pub struct Pool {
  /// All the registered connections
  connections: HashMap<Uuid, Connection>,
}

impl Pool {
  pub fn new() -> Pool {
    Pool {
      connections: HashMap::new(),
    }
  }

  pub fn add_connection(&mut self, connection: Connection) -> GraphtResult<()> {
    todo!("Pool::add_connection");
  }
}

#[derive(Debug, Clone)]
pub struct Connection {
  guid: Uuid,

  name: String,

  state: ConnectionState,

  /// All the active endpoints associated with this connection
  endpoint: Endpoint,

  /// Whether this connection can be pooled
  concurrency: Concurrency,

  /// Whether the connection can be reused after the current transaction is complete
  reusable: bool,
}

#[derive(Debug, Clone)]
pub enum Concurrency {
  Singleton,

  /// Create more than one concurrent connection can be created
  Multiple(u32, u32),
}

#[derive(Debug, Clone)]
pub enum ConnectionState {
  /// Connection is configured but has not been initialized
  Closed,

  /// The connection is open and waiting to process messages
  Ready,

  /// The connection is actively processing messages
  Busy,

  /// The connection is waiting for an external
  Listening,

  /// A recoverable error occurred
  ///
  /// This state is for items like network connectivity issues, where repair can be automated
  Error,

  /// The connection crashed in an indeterminate state and require manual intervetion to clean up
  Poisoned,
}

/// And individual data portal, handling a specific format of information
#[derive(Debug, Clone)]
pub struct Endpoint {
  /// Unique Identifier generated from then endpoint typo
  guid: Uuid,

  name: String,

  /// How to encode/decode messages travelling through this endpoint
  ///
  /// TODO: This will be Nomnomicon based grammar
  grammar: (),
}

#[derive(Debug, Clone)]
pub enum EndpointType {
  /// A target where messages have to be routed to the correct address before sending
  ///
  /// Generally, this would be like targeting a RESTful interface
  API(ApiEndpoint),

  /// A single data stream (file or url)
  Socket(SocketEndpoint),

  /// An open endpoint to receive unsolicited messages on (such as subscription events)
  Listener(ListenerEndpoint),
}
