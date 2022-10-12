//! A mutable subset used for working with items in the data store
//!
//! TODO:
//! - Ordered subsets -

use crate::{err, local::*};

use std::collections::{hash_map::Entry, HashMap};

// use sync::{
//   mpsc::{channel, Sender},
//   Arc,
// };

use uuid::Uuid;

/// A group of related nodes and edges
#[derive(Clone, Debug)]
pub struct DataSet<G>
where
  G: Graph,
{
  guid: Uuid,

  nodes: HashMap<Uuid, Node<G>>,

  /// Lookups for edges included in this group
  edges: EdgeMap<G>,

  // Indices pointing to groups of nodes
  subsets: HashMap<String, DataSet<G>>,

  /// Pointers for ordering and subsets
  /// FIXME: This needs to be more flexible
  indices: HashMap<String, Vec<Uuid>>,

  /// A consolidated patch query to to bring Grapht up to date with
  ///
  /// A dynamic query containing mutations since the last sync to Grapht
  diff: (),
}

impl<G> DataSet<G>
where
  G: Graph,
{
  pub fn new() -> DataSet<G> {
    let mut indices = HashMap::new();

    DataSet {
      guid: Uuid::new_v4(),
      nodes: HashMap::new(),
      edges: EdgeMap::new(),
      subsets: HashMap::new(),
      indices,
      diff: (),
    }
  }

  pub fn get(&self, query: &str) -> GraphtResult<DataSet<G>> {
    todo!()
  }

  pub fn get_guid(&self) -> Uuid {
    self.guid.clone()
  }

  pub fn insert(&mut self, node: Node<G>) -> GraphtResult<Stats> {
    match self.nodes.entry(node.get_guid()) {
      Entry::Vacant(entry) => {
        entry.insert(node);
        let mut stats = Stats::default();
        stats.nodes.created = 1;
        Ok(stats)
      }
      Entry::Occupied(_) => Err(err!(
        DuplicateKey,
        "Node with Uuid {} already exists in the graph {}",
        node.get_guid(),
        self.guid
      )),
    }
  }

  pub fn stats(&self) -> Stats {
    let mut stats = Stats::default();
    stats.nodes.created = self.nodes.len() as u128;

    stats
  }
}

#[derive(Clone, Debug)]
pub struct Activity {
  timestamp: (),
  item: ActivityItem,
  synced: bool,
}

#[derive(Clone, Debug)]
pub enum ActivityItem {
  // Makes sure the change is not included in the sync
  Local(Box<ActivityItem>),

  /// Select specific data from the Grapht instance
  Query,
  Insert,
  Update,
  Mutate,
  /// Excludes an item from the set, but doesn't persist any changes to
  Remove,
  Delete,
}
