//! Context sensitive statistics about a data set
//!
//! This is used both as the result of a query and of a static data set

use crate::{local::*, utils::*};

use std::ops::{Add, AddAssign};

pub trait Stats: Debug + Clone + PartialEq + Eq + Diff + Add + AddAssign {}

/// CRUD operation statistics created by wrapping existing statistics by mutation type
#[derive(Debug, Clone, Default)]
pub struct CrudResultStats<T>
where
  T: Stats + Add<Output = T> + Default,
{
  created: Option<T>,
  updated: Option<T>,
  deleted: Option<T>,
  errors: Vec<crate::errors::GraphtError>,
}

impl<T> CrudResultStats<T>
where
  T: Stats + Add<Output = T> + Default,
{
  pub fn new() -> CrudResultStats<T> {
    CrudResultStats {
      created: None,
      updated: None,
      deleted: None,
      errors: Vec::new(),
    }
  }

  pub fn created(&self) -> Option<T> {
    self.created.clone()
  }

  pub fn add_created(&mut self, stats: T) {
    if let Some(s) = &mut self.created {
      *s += stats;
    }
  }

  pub fn updated(&self) -> Option<T> {
    self.updated.clone()
  }

  pub fn add_updated(&mut self, stats: T) {
    if let Some(s) = &mut self.updated {
      *s += stats;
    }
  }

  pub fn deleted(&self) -> Option<T> {
    self.deleted.clone()
  }

  pub fn add_deleted(&mut self, stats: T) {
    if let Some(s) = &mut self.deleted {
      *s += stats;
    }
  }

  /// Diff two sets of stats and throw an error if they are not the same
  ///
  /// This is primarily for testing, but handy to have integrated into the live code.
  pub fn assert_eq(&self, rhs: &Self) {
    let diff = self.diff(&rhs, None);
    if let Difference::Node(_, _) = diff {
      error!(
        "The two sets of stats had the following differences:\n{}",
        diff
      );
      panic!()
    }
  }
}

impl<T> Add for CrudResultStats<T>
where
  T: Stats + Add<Output = T> + Default,
{
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    let created = match (self.created, rhs.created) {
      (None, None) => None,
      (Some(x), None) => Some(x),
      (None, Some(y)) => Some(y),
      (Some(x), Some(y)) => Some(x + y),
    };

    let updated = match (self.updated, rhs.updated) {
      (None, None) => None,
      (Some(x), None) => Some(x),
      (None, Some(y)) => Some(y),
      (Some(x), Some(y)) => Some(x + y),
    };

    let deleted = match (self.deleted, rhs.deleted) {
      (None, None) => None,
      (Some(x), None) => Some(x),
      (None, Some(y)) => Some(y),
      (Some(x), Some(y)) => Some(x + y),
    };

    let mut errors = self.errors.clone();
    errors.extend(rhs.errors);

    Self {
      created,
      updated,
      deleted,
      errors,
    }
  }
}

impl<T> AddAssign for CrudResultStats<T>
where
  T: Stats + Add<Output = T> + Default,
{
  fn add_assign(&mut self, rhs: Self) {
    *self = self.clone().add(rhs);
  }
}

impl<T> Diff for CrudResultStats<T>
where
  T: Stats + Add<Output = T> + Diff + Default,
{
  fn diff(&self, rhs: &Self, name: Option<&str>) -> Difference {
    let mut diff = Difference::new();
    diff += self.created.diff(&rhs.created, Some("created"));
    diff += self.updated.diff(&rhs.updated, Some("updated"));
    diff += self.deleted.diff(&rhs.deleted, Some("deleted"));
    diff += self.errors.diff(&rhs.errors, Some("failed"));

    diff
  }
}

/*

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

    if diff.is_empty() {
      return Difference::Empty;
    }

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

    // If there is no change, it shouldn't need to do anything else
    if let Difference::Empty = diff {
      return diff;
    }

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

impl IndexStats {
  pub fn new() -> IndexStats {
    IndexStats::default()
  }
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
  total: Mutations,
  types: HashMap<String, Mutations>,
  // Properties
  // Labels
}

impl Add for NodeStats {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    let mut total = self.total.clone();
    total += rhs.total;

    let mut types = self.types.clone();
    for (key, value) in rhs.types.iter() {
      if let Some(lhs) = types.get_mut(key) {
        *lhs += value.clone();
      } else {
        let _ = types.insert(key.clone(), value.clone());
      }
    }
    Self { total, types }
  }
}

impl Diff for NodeStats {
  fn diff(&self, rhs: &Self, name: Option<&str>) -> Difference {
    let mut diff = Difference::new();
    diff += self.total.diff(&rhs.total, Some("total"));
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
 */
