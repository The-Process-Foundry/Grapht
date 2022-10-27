//! Context sensitive statistics about a data set
//!
//! This is used both as the result of a query and of a static data set

use std::ops::{Add, AddAssign};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stats {
  pub nodes: Mutations,
  pub labels: Mutations,
  pub indices: Mutations,
  pub edges: Mutations,
  pub paths: Mutations,
}

impl Stats {
  pub fn new() -> Stats {
    Stats::default()
  }
}

impl Add for Stats {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    Self {
      nodes: self.nodes + rhs.nodes,
      labels: self.labels + rhs.labels,
      indices: self.indices + rhs.indices,
      edges: self.edges + rhs.edges,
      paths: self.paths + rhs.paths,
    }
  }
}

impl AddAssign for Stats {
  fn add_assign(&mut self, rhs: Self) {
    *self = self.clone().add(rhs);
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
