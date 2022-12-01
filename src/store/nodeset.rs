//! A group of nodes contained in a graph

use crate::{local::*, prelude::*};

use std::{
  collections::{
    hash_map::{self, Entry},
    HashMap,
  },
  ops::{Add, AddAssign},
}; // , VecDeque};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A group of nodes and their associated indices
#[derive(Clone, Debug, Default)]
pub struct NodeSet<G>
where
  G: Graph,
{
  /// A unique ID for this group of nodes
  guid: Uuid,

  /// Look up for all nodes in the set. The values of which can be used as an iterator.
  nodes: HashMap<Uuid, Node<G>>,

  /// Group the nodes by their type
  typed: HashMap<String, HashMap<Uuid, Node<G>>>,

  /// Group the nodes by each individual label
  labels: HashMap<String, HashMap<Uuid, Node<G>>>,

  /// Live statistics about the nodes
  stats: NodeStats,
}

impl<G> NodeSet<G>
where
  G: Graph,
{
  pub fn new() -> NodeSet<G> {
    NodeSet {
      guid: Uuid::new_v4(),
      nodes: HashMap::new(),
      typed: HashMap::new(),
      labels: HashMap::new(),
      stats: NodeStats::new(),
    }
  }

  /// Check if a node with the given guid is registered with the graph
  pub fn contains(&self, guid: &Uuid) -> bool {
    self.nodes.contains_key(guid)
  }

  /// Queries and retrieves nodes matching the query
  pub fn get(&self, guid: &Uuid) -> Option<&Node<G>> {
    self.nodes.get(guid)
  }

  /// Queries and retrieves nodes matching the query
  pub fn query(&self, _query: &str) -> GraphtResult<Vec<Node<G>>> {
    todo!()
  }

  /// Insert a new node into the graph and fail if it already exists
  pub fn insert(&mut self, node: &Node<G>) -> GraphtResult<CrudResultStats<NodeStats>> {
    let mut stats = NodeStats::new();

    // Make a copy of the node that cannot be directly accessed by calling code
    let mut new_node: Node<G>;
    match self.nodes.entry(node.get_guid()) {
      Entry::Vacant(entry) => {
        debug!("Vacant node");
        new_node = node.deep_clone()?;
        new_node.set_bound(true);
        entry.insert(new_node.clone());
      }
      Entry::Occupied(_) => {
        return Err(err!(
          DuplicateKey,
          "Node with Uuid {} already exists in the NodeSet {}",
          node.get_guid(),
          self.guid
        ));
      }
    };

    // Update the stats for Node
    stats.total.increase(1);
    self.stats.total.increase(1);

    // Clone the node for use with closures for indexing the node
    let node = new_node.clone();
    self
      .typed
      .entry(node.type_label())
      .and_modify(|inner| {
        let result = inner.insert(node.get_guid(), node.clone());
        if result.is_some() {
          panic!("Inserting a node into self.typed should never fail if it wasn't in self.nodes")
        }
      })
      .or_insert_with(|| [(node.get_guid(), node.clone())].into());

    stats.typed.increase((node.type_label(), 1));
    self.stats.typed.increase((node.type_label(), 1));

    // Insert labels
    for label in node.get_labels() {
      let node = new_node.clone();

      self
        .labels
        .entry(label.clone())
        .and_modify(|inner| {
          let _ = inner.insert(node.get_guid(), node.clone());
        })
        .or_insert_with(|| [(node.get_guid(), node)].into());

      stats.labels.increase((label.clone(), 1));
      self.stats.labels.increase((label.clone(), 1));
    }
    let mut data_set_stats = CrudResultStats::new();
    data_set_stats.add_created(stats);
    Ok(data_set_stats)
  }

  /// Updates the original node with the new node, updating the internal indices as it goes
  pub fn update(&mut self, _node: &Node<G>) -> GraphtResult<NodeStats> {
    todo!()
  }

  /// Removes the node and all its indices from the set
  pub fn delete(&mut self, _query: ()) -> GraphtResult<NodeStats> {
    todo!()
  }

  pub fn stats(&self) -> NodeStats {
    let mut total = StatCount::new();
    total.increase(self.nodes.len() as i128);

    let mut typed = StatMap::new();
    for (key, values) in &self.typed {
      typed.increase((key.clone(), values.len() as i128));
    }

    let mut labels = StatMap::new();
    for (key, values) in &self.labels {
      labels.increase((key.clone(), values.len() as i128));
    }

    let properties = StatMap::new();

    NodeStats {
      total,
      typed,
      labels,
      properties,
    }
  }
}

impl<'a, G> IntoIterator for &'a NodeSet<G>
where
  G: Graph,
{
  type Item = &'a Node<G>;
  type IntoIter = hash_map::Values<'a, Uuid, Node<G>>;

  fn into_iter(self) -> Self::IntoIter {
    self.nodes.values()
  }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeStats {
  /// A count of the nodes
  #[serde(default)]
  pub total: StatCount,

  /// Counts of nodes sorted by type
  #[serde(default)]
  pub typed: StatMap<String, StatCount, <StatCount as Stats>::Item>,

  /// Counts of nodes tagged by a given label
  #[serde(default)]
  pub labels: StatMap<String, StatCount, <StatCount as Stats>::Item>,

  /// Counts of distinct property names
  ///
  /// These are a holdover from RedisGraph. As Grapht uses explicit types rather than a
  /// document model, properties don't really change.
  #[serde(default)]
  pub properties: StatMap<String, StatCount, <StatCount as Stats>::Item>,
}

impl NodeStats {
  pub fn new() -> NodeStats {
    NodeStats {
      total: StatCount::new(),
      typed: StatMap::new(),
      labels: StatMap::new(),
      properties: StatMap::new(),
    }
  }
}

impl Diff for NodeStats {
  fn diff(&self, rhs: &Self, name: Option<&str>) -> Difference {
    let mut diffs = Difference::new();
    diffs += self.total.diff(&rhs.total, Some("total"));
    diffs += self.typed.diff(&rhs.typed, Some("typed"));
    diffs += self.labels.diff(&rhs.labels, Some("labels"));
    diffs += self.properties.diff(&rhs.properties, Some("properties"));
    diffs.opt_tag(name)
  }
}

impl Add for NodeStats {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    NodeStats {
      total: self.total + rhs.total,
      typed: self.typed + rhs.typed,
      labels: self.labels + rhs.labels,
      properties: self.properties + rhs.properties,
    }
  }
}

impl AddAssign for NodeStats {
  fn add_assign(&mut self, rhs: Self) {
    *self = self.clone().add(rhs);
  }
}

impl Stats for NodeStats {
  type Item = String;

  /// Increase the statistic using the a YAML string
  fn increase(&mut self, _value: Self::Item) {
    unimplemented!("'Stats::increase needs to be implemented for NodeStats")
  }

  fn clear(&mut self) {
    self.total.clear();
    self.typed.clear();
    self.labels.clear();
    self.properties.clear();
  }
}
