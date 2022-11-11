//! A group of edges contained in a graph

use crate::{local::*, prelude::*};

use std::{
  collections::{
    hash_map::{self, Entry},
    HashMap,
  },
  ops::{Add, AddAssign},
}; // , VecDeque};

use uuid::Uuid;

/// A group of edges and their associated indices
#[derive(Clone, Debug, Default)]
pub struct EdgeSet<G>
where
  G: Graph,
{
  /// A unique ID for this group of edges
  guid: Uuid,

  /// Look up for all edges in the set. The values of which can be used as an iterator.
  edges: HashMap<Uuid, Edge<G>>,

  /// Group the edges by their type
  typed: HashMap<String, HashMap<Uuid, Edge<G>>>,

  /// Live statistics about the edges
  stats: EdgeStats,
}

impl<G> EdgeSet<G>
where
  G: Graph,
{
  pub fn new() -> EdgeSet<G> {
    EdgeSet {
      guid: Uuid::new_v4(),
      edges: HashMap::new(),
      typed: HashMap::new(),
      stats: EdgeStats::new(),
    }
  }

  /// Check if an edge with the given guid is registered with the graph
  pub fn contains(&self, guid: &Uuid) -> bool {
    self.edges.contains_key(guid)
  }

  /// Queries and retrieves edges matching the query
  pub fn get(&self, _query: &str) -> GraphtResult<Vec<Edge<G>>> {
    todo!()
  }

  // Insert the edge into the graph and fail if it already exists
  pub fn insert(&mut self, edge: &Edge<G>) -> GraphtResult<EdgeStats> {
    // Make a copy of the edge that cannot be directly accessed by calling code
    let edge = edge.clone();
    match self.edges.entry(edge.get_guid()) {
      Entry::Vacant(entry) => {
        debug!("Vacant edge");
        entry.insert(edge.clone());
      }
      Entry::Occupied(_) => {
        return Err(err!(
          DuplicateKey,
          "Edge with Uuid {} already exists in the EdgeSet {}",
          edge.get_guid(),
          self.guid
        ));
      }
    };

    // Update the stats
    let mut stats = EdgeStats::new();
    stats.total = 1;
    self.stats.total += 1;

    // Clone the edge for use with closures
    self
      .typed
      .entry(edge.get_type_label())
      .and_modify(|inner| {
        let result = inner.insert(edge.get_guid(), edge.clone());
        if result.is_some() {
          panic!("Inserting a edge into self.typed should never fail if it wasn't in self.edges")
        }
      })
      .or_insert_with(|| [(edge.get_guid(), edge)].into());

    Ok(stats)
  }

  /// Updates the original edge with the new edge, updating the internal indices as it goes
  pub fn update(&mut self, edge: &Edge<G>) -> GraphtResult<EdgeStats> {
    todo!()
  }

  /// Removes the edge and all its indices from the set
  pub fn delete(&mut self, _query: ()) -> GraphtResult<EdgeStats> {
    todo!()
  }

  pub fn stats(&self) -> EdgeStats {
    let mut typed = HashMap::new();
    for (key, values) in &self.typed {
      let _ = typed.insert(key.clone(), values.len() as u128);
    }

    let properties = HashMap::new();

    EdgeStats {
      total: self.edges.len() as u128,
      typed,
      properties,
    }
  }
}

impl<'a, G> IntoIterator for &'a EdgeSet<G>
where
  G: Graph,
{
  type Item = &'a Edge<G>;
  type IntoIter = hash_map::Values<'a, Uuid, Edge<G>>;

  fn into_iter(self) -> Self::IntoIter {
    self.edges.values()
  }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EdgeStats {
  /// A count of the edges
  total: u128,
  typed: HashMap<String, u128>,
  properties: HashMap<String, u128>,
}

impl EdgeStats {
  pub fn new() -> EdgeStats {
    EdgeStats {
      total: 0,
      typed: HashMap::new(),
      properties: HashMap::new(),
    }
  }
}

impl Diff for EdgeStats {
  fn diff(&self, rhs: &Self, name: Option<&str>) -> Difference {
    todo!()
  }
}

impl Add for EdgeStats {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    let total = self.total + rhs.total;

    let mut typed = self.typed;
    for (key, value) in rhs.typed {
      typed
        .entry(key)
        .and_modify(|val| *val = *val + value)
        .or_insert(value);
    }

    let mut properties = self.properties;
    for (key, value) in rhs.properties {
      properties
        .entry(key)
        .and_modify(|val| *val = *val + value)
        .or_insert(value);
    }

    EdgeStats {
      total,
      typed,
      properties,
    }
  }
}

impl AddAssign for EdgeStats {
  fn add_assign(&mut self, rhs: Self) {
    *self = self.clone().add(rhs);
  }
}

impl Stats for EdgeStats {}
