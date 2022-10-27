//! A Grapht data store

use crate::{local::*, prelude::*};
use sync::mpsc::channel;

pub struct Grapht<G>
where
  G: Graph,
{
  data: DataSet<G>,
  // /// Connections to the remote services
  // remotes: Pool,

  // /// A channel for all the data
  // messenger: Messenger<G>,
  // Messengers for datasets wanting messages pushed
  // subscribers: (),
}

impl<G> Grapht<G>
where
  G: Graph,
{
  pub fn new() -> Grapht<G> {
    Grapht {
      data: DataSet::new(),
      // remotes: Pool::new(),

      // Active queries that need to be pushed updates to data
      // subscriptions: HashSet<Subset>
      // messenger: Messenger::new(),
    }
  }

  /// Creates a new immutable data set based on the query
  ///
  pub fn query(&mut self, _query: &str) -> GraphtResult<DataSet<G>> {
    todo!(" No grapht query yet")
  }

  /// Creates a new mutable data set using the query as a
  pub fn query_mut(&mut self, _query: &str) -> GraphtResult<DataSet<G>> {
    todo!(" No grapht query_mut yet")
  }

  /// Creates a immutable data set using the query and receives any updates that match
  pub fn subscribe(&mut self, _query: &str) -> GraphtResult<DataSet<G>> {
    todo!(" No grapht subscribe yet")
  }

  /// Creates a new mutable data set using the query as a
  pub fn subscribe_mut(&mut self, _query: &str) -> GraphtResult<DataSet<G>> {
    todo!(" No grapht subscribe_mut yet")
  }
}

/*
/// Channel
pub struct Messenger<G>
where
  G: Graph, {


  }

impl<G> Messenger<G>
where
  G: Graph,
{
  pub fn new() -> Messenger<G> {
    let (sender, receiver) = channel();
    Messenger {}
  }
}

pub enum Message {
  Insert,
  Delete,
  Update,
}
 */
