//! Intraprocess Messaging
//!
//! These are the messages that are sent via channel between DataSets and the central Grapht store

use crate::local::*;
use sync::mpsc::{channel, Sender};

use uuid::Uuid;

pub enum GraphtMessage {
  /// A new dataset
  DataSet,
  /// Asking for fresh data
  Query,
}

pub enum DataSet {
  Register(Uuid, Sender<GraphtMessage>),
}
