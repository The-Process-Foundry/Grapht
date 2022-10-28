//! Context sensitive statistics about a data set
//!
//! This is used both as the result of a query and of a static data set

use crate::{local::*, utils::*};

use std::{
  collections::HashMap,
  ops::{Add, AddAssign},
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stats {
  /// Total mutations on the nodes
  pub nodes: Mutations,
  /// Total mutations on the edges
  pub edges: Mutations,
  /// Mutations on the various indices
  pub indices: IndexStats,
}

impl Stats {
  pub fn new() -> Stats {
    Stats::default()
  }

  /// Diff two sets of stats and throw an error if they are not the same
  ///
  /// This is primarily for testing, but handy to have integrated into the live code.
  pub fn assert_eq(&self, rhs: &Self) {
    let diff = self.diff(&rhs, None);
    if let Difference::Node(_, _) = diff {
      panic!(
        "The two sets of stats had the following differences: {:?}",
        diff
      )
    }
  }
}

impl Add for Stats {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    Self {
      nodes: self.nodes + rhs.nodes,
      edges: self.edges + rhs.edges,
      indices: self.indices + rhs.indices,
    }
  }
}

impl AddAssign for Stats {
  fn add_assign(&mut self, rhs: Self) {
    *self = self.clone().add(rhs);
  }
}

impl Diff for Stats {
  fn diff(&self, rhs: &Self, name: Option<&str>) -> Difference {
    let mut diff = Difference::new();
    diff += self.nodes.diff(&rhs.nodes, Some("nodes"));
    diff += self.edges.diff(&rhs.edges, Some("edges"));
    diff += self.indices.diff(&rhs.indices, Some("indices"));

    match name {
      Some(name) => {
        let mut result = Difference::Empty;
        result.merge(diff, Some(vec![name.to_string()]));
        result
      }
      None => diff,
    }
  }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Mutations {
  // The total number of items that were touched but left alone
  pub unchanged: u128,

  // Total number of items created
  pub created: u128,

  // Total number of items created
  pub updated: u128,

  // Total number of items deleted
  pub deleted: u128,
}

impl Mutations {
  pub fn new() -> Mutations {
    Mutations::default()
  }
}

impl Add for Mutations {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    Self {
      unchanged: self.unchanged + rhs.unchanged,
      created: self.created + rhs.created,
      updated: self.updated + rhs.updated,
      deleted: self.deleted + rhs.deleted,
    }
  }
}

impl AddAssign for Mutations {
  fn add_assign(&mut self, rhs: Self) {
    *self = self.clone().add(rhs);
  }
}

impl Diff for Mutations {
  fn diff(&self, rhs: &Mutations, name: Option<&str>) -> Difference {
    let mut diff = Difference::new();
    diff += self.unchanged.diff(&rhs.unchanged, Some("unchanged"));
    diff += self.created.diff(&rhs.created, Some("created"));
    diff += self.updated.diff(&rhs.updated, Some("updated"));
    diff += self.deleted.diff(&rhs.deleted, Some("deleted"));

    match name {
      Some(name) => {
        let mut result = Difference::Empty;
        result.merge(diff, Some(vec![name.to_string()]));
        result
      }
      None => diff,
    }
  }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IndexStats {
  pub nodes: NodeStats,
  pub edges: EdgeStats,
  pub labels: LabelStats,
}

impl Add for IndexStats {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    Self {
      nodes: self.nodes + rhs.nodes,
      edges: self.edges + rhs.edges,
      labels: self.labels + rhs.labels,
    }
  }
}

impl Diff for IndexStats {
  fn diff(&self, rhs: &Self, name: Option<&str>) -> Difference {
    let mut diff = Difference::new();
    diff += self.nodes.diff(&rhs.nodes, Some("nodes"));

    match name {
      Some(name) => {
        let mut result = Difference::Empty;
        result.merge(diff, Some(vec![name.to_string()]));
        result
      }
      None => diff,
    }
  }
}

/// Information about the nodes that are being watched
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NodeStats {
  /// A breakdown of counts by type
  types: HashMap<String, Mutations>,
}

impl Add for NodeStats {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    let mut types = self.types.clone();
    for (key, value) in rhs.types.iter() {
      if let Some(lhs) = types.get_mut(key) {
        *lhs += value.clone();
      } else {
        let _ = types.insert(key.clone(), value.clone());
      }
    }
    Self { types }
  }
}

impl Diff for NodeStats {
  fn diff(&self, rhs: &Self, name: Option<&str>) -> Difference {
    let mut diff = Difference::new();
    diff += self.types.diff(&rhs.types, None);

    match name {
      Some(name) => {
        let mut result = Difference::Empty;
        result.merge(diff, Some(vec![name.to_string()]));
        result
      }
      None => diff,
    }
  }
}

/// Information about the edges that are connecting the nodes
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EdgeStats {
  /// A breakdown of counts by type
  types: HashMap<String, Mutations>,
}

impl Add for EdgeStats {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    let mut types = self.types.clone();
    for (key, value) in rhs.types.iter() {
      if let Some(lhs) = types.get_mut(key) {
        *lhs += value.clone();
      } else {
        let _ = types.insert(key.clone(), value.clone());
      }
    }
    Self { types }
  }
}

impl Diff for EdgeStats {
  fn diff(&self, rhs: &Self, name: Option<&str>) -> Difference {
    let diff = self.types.diff(&rhs.types, None);
    match name {
      Some(name) => {
        let mut result = Difference::Empty;
        result.merge(diff, Some(vec![name.to_string()]));
        result
      }
      None => diff,
    }
  }
}

/// Counts of tags and labels
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LabelStats {
  pub total: Mutations,
  pub labels: HashMap<String, Mutations>,
}

impl Add for LabelStats {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    let mut labels = self.labels.clone();
    for (key, value) in rhs.labels.iter() {
      if let Some(lhs) = labels.get_mut(key) {
        *lhs += value.clone();
      } else {
        let _ = labels.insert(key.clone(), value.clone());
      }
    }
    Self {
      total: self.total + rhs.total,
      labels,
    }
  }
}

impl Diff for LabelStats {
  fn diff(&self, rhs: &Self, name: Option<&str>) -> Difference {
    let mut diff = Difference::new();
    diff += self.total.diff(&rhs.total, Some("total"));
    diff += self.labels.diff(&rhs.labels, Some("labels"));

    match name {
      Some(name) => {
        let mut result = Difference::Empty;
        result.merge(diff, Some(vec![name.to_string()]));
        result
      }
      None => diff,
    }
  }
}
