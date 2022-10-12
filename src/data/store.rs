//! The raw set of nodes and edges

use crate::local::*;

use std::collections::HashMap;

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Store<G>
where
  G: Graph,
{
  nodes: HashMap<Uuid, Node<G>>,
}

impl<G> Store<G>
where
  G: Graph,
{
  pub fn new() -> Store<G> {
    Store {
      nodes: HashMap::new(),
    }
  }
}
