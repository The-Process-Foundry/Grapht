//! A wrapper for nodes, edges, and paths so they can be used interchangeably

use crate::{local::*, prelude::*};

use core::hash::{Hash, Hasher};
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

impl<G> From<Node<G>> for Value<G>
where
  G: Graph,
{
  fn from(value: Node<G>) -> Self {
    Value::Node(value)
  }
}

impl<G> From<Edge<G>> for Value<G>
where
  G: Graph,
{
  fn from(value: Edge<G>) -> Self {
    Value::Edge(value)
  }
}

impl<G> From<&Value<G>> for Edge<G>
where
  G: Graph,
{
  fn from(value: &Value<G>) -> Self {
    match value {
      Value::Edge(edge) => edge.clone(),
      Value::Node(_) => panic!("Cannot convert a node to an edge"),
      Value::Path(_) => panic!("Cannot convert a path to an edge"),
    }
  }
}

impl<G> From<Path<G>> for Value<G>
where
  G: Graph,
{
  fn from(value: Path<G>) -> Self {
    Value::Path(value)
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

impl<G> PartialEq for Value<G>
where
  G: Graph,
{
  fn eq(&self, other: &Self) -> bool {
    self.get_guid() == other.get_guid()
  }
}

impl<G> Eq for Value<G> where G: Graph {}
