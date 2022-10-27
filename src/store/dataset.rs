//! A mutable subset used for working with items in the data store
//!
//! TODO:
//! - Ordered subsets
//! - get_mut/update: Create a trait for getting a node from the set with a commit() function that
//!   automatically pushes a patch into the set

use crate::{local::*, prelude::*};

use std::collections::{hash_map::Entry, HashMap}; // , VecDeque};

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

  /// All the vertices known to this set mapped by its guid
  nodes: HashMap<Uuid, Node<G>>,

  /// Pointers for ordering and subsets
  /// FIXME: This needs to be more flexible
  indices: Indices<G>,

  // -----    Thinking/YAGNI items
  /// Statistics about the current DataSet
  ///
  /// Eventually, This should be items like memory usage, cache hits/misses
  /// THINK: Is this needed or is it fast enough to be run dynamically?
  _stats: (),

  /// A consolidated patch query to to bring Grapht up to date with
  ///
  /// A dynamic query containing mutations since the last sync to Grapht
  _diff: (),
}

impl<G> PartialEq for DataSet<G>
where
  G: Graph,
{
  fn eq(&self, other: &Self) -> bool {
    self.guid == other.guid
  }
}

impl<G> DataSet<G>
where
  G: Graph,
{
  pub fn new() -> DataSet<G> {
    DataSet {
      guid: Uuid::new_v4(),
      nodes: HashMap::new(),
      indices: Indices::new(),
      _stats: (),
      _diff: (),
    }
  }

  /// Basic database query, returning results in iterable lists
  pub fn query(&self, _query: &str) -> GraphtResult<HashMap<String, Vec<Value<G>>>> {
    todo!()
  }

  /// Create a new DataSet that matches the values matching the query
  ///
  /// Items such as ordering are created in the indices. All annotations are dropped and aggregates
  /// are re-calculated
  pub fn subset(&self, _query: &str) -> GraphtResult<DataSet<G>> {
    todo!()
  }

  /// Return a list of all the nodes included in the DataSet
  pub fn nodes(&self, _query: &str) -> GraphtResult<Vec<Node<G>>> {
    let mut matches = Vec::new();
    for node in self.nodes.values() {
      matches.push(node.clone())
    }
    Ok(matches)
  }

  pub fn get_guid(&self) -> Uuid {
    self.guid.clone()
  }

  /// Add a value and all its related values (properties, edges, etc) to a graph
  ///
  /// The primary purpose of this insert is to make sure the value is indexed properly within the
  /// dataset. Errors are caused when finding mutated properties.
  /// THINK:
  /// - When should each node be hashed/diffed on insert so we can determine if there has been a
  ///   change? (use/update Patchwork library)
  /// - How are updated nodes handled?
  /// - If an edge is added,
  pub fn insert(&mut self, value: Value<G>) -> GraphtResult<Stats> {
    let mut stats = Stats::new();

    // To protect against stack overflows during recursion, we use a while loop containing all the
    // new nodes
    let mut unprocessed = vec![value];

    while !unprocessed.is_empty() {
      let value = unprocessed.pop().unwrap();
      debug!(
        "Inserting value {} with {} remaining unprocessed",
        value.get_guid(),
        unprocessed.len()
      );

      // We disassemble the value and process one node
      match value {
        // Only need to add the one node
        Value::Node(node) => {
          match self.insert_node(node.clone()) {
            Err(err) => match err.is(Kind::DuplicateKey) {
              true => continue,
              false => return Err(err),
            },
            Ok(value_stats) => stats += value_stats,
          };

          // Add the target and each edge
          for edge in node.edges("") {
            unprocessed.push(edge.get_target().into());
            unprocessed.push(edge.into())
          }
        }

        Value::Edge(edge) => {
          let index = Index::Edge(edge.get_type_label());
          // Skip if the edge already exists if the edge already exists.
          if self.indices.contains(&index, &edge.get_guid()) {
            debug!(
              "Skipping repeat edge {} {}",
              edge.get_type_label(),
              edge.get_guid()
            );
            continue;
          };

          // First, we ensure that both source and target are in the data set before starting
          // let mut missing_nodes = Vec::new();
          // let source = edge.get_source();
          // if !self.nodes.contains_key(&source.get_guid()) {
          //   missing_nodes.push(source.into());
          // }

          // let target = edge.get_target();
          // if !self.nodes.contains_key(&target.get_guid()) {
          //   missing_nodes.push(target.into());
          // }

          // If either source or target node is missing, we requeue the edge to try again later

          // Insert the source node. If it already in the data set, only add the edge to it
          // match self.nodes.get(&source.get_guid()) {
          //   Some(node) => {
          //     let mut node = node.clone();
          //     node.add_edge(edge.clone());
          //     self.update_node_unchecked(node.clone())?;
          //   }
          //   None => {
          //     match self.insert_node(source) {
          //       Err(err) => match err.is(Kind::DuplicateKey) {
          //         true => unreachable!(""),
          //         false => return Err(err),
          //       },
          //       Ok(value_stats) => stats += value_stats,
          //     };
          //   }
          // };

          // index the

          // And make sure the target node exists as well
        }

        Value::Path(_) => todo!("Cannot insert paths yet"),
      };
    }
    Ok(stats)
  }

  /// Insert a single node into the graph
  pub fn insert_node(&mut self, node: Node<G>) -> GraphtResult<Stats> {
    let mut stats = Stats::new();
    match self.nodes.entry(node.get_guid()) {
      Entry::Vacant(entry) => {
        entry.insert(node.clone());
        stats.nodes.created = 1;
      }
      Entry::Occupied(_) => {
        return Err(err!(
          DuplicateKey,
          "Node with Uuid {} already exists in the graph {}",
          node.get_guid(),
          self.guid
        ))
      }
    };

    stats += self.indices.index_value(Value::Node(node))?;
    Ok(stats)
  }

  /// Update the properties, edges, and indices of a node and return the stats of the changes
  pub fn update_node(&mut self, _node: Node<G>) -> GraphtResult<Stats> {
    todo!()
  }

  /// Replace an existing node with the current one without internal validation
  pub(crate) fn _update_node_unchecked(&mut self, node: Node<G>) -> GraphtResult<()> {
    // let mut stats = Stats::new();
    match self.nodes.entry(node.get_guid()) {
      Entry::Vacant(_) => {
        return Err(err!(
          NotFound,
          "Could not get Node with Uuid {} in the graph {} for updating",
          node.get_guid(),
          self.guid
        ))
      }
      Entry::Occupied(mut entry) => {
        entry.insert(node.clone());
      }
    };
    Ok(())
  }

  pub fn stats(&self) -> Stats {
    let mut stats = Stats::default();
    stats.nodes.created = self.nodes.len() as u128;

    stats
  }
}

impl<G> fmt::Display for DataSet<G>
where
  G: Graph,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("DataSet")
      .field("guid", &self.guid)
      .field("nodes", &self.nodes.len())
      .field("indices", &format!("{}", self.indices))
      .finish()
  }
}

/*
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
 */
