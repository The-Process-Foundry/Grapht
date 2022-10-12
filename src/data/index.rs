//! Pre-compiled lookups for getting node, edge, and paths

use crate::local::*;

use std::{
  collections::{HashMap, HashSet},
  hash::{Hash, Hasher},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Value<G>
where
  G: Graph,
{
  Node(Node<G>),
  Edge(Edge<G>),
  Path(Path<G>),
}

impl<G> Value<G>
where
  G: Graph,
{
  pub fn get_guid(&self) -> Uuid {
    match self {
      Value::Node(node) => node.get_guid(),
      Value::Edge(edge) => edge.get_guid(),
      Value::Path(path) => path.get_guid(),
    }
  }
}

impl<G> Hash for Value<G>
where
  G: Graph,
{
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.get_guid().hash(state);
  }
}

#[derive(Debug, Clone)]
pub struct Index<G>
where
  G: Graph,
{
  name: String,
  lookup: HashMap<Uuid, HashSet<Value<G>>>,
}
