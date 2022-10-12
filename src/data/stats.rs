//! Statistics about a data set
//!
//! This is used both as the result of a query and of a static data set

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stats {
  pub nodes: Mutations,
  pub labels: Mutations,
  pub edges: Mutations,
  pub paths: Mutations,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Mutations {
  // Total number of items created
  pub created: u128,
  // Total number of items deleted
  pub deleted: u128,
}
