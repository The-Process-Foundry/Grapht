//! The definition of a relationship between two nodes

use super::*;
use crate::err;

use std::{
  collections::{
    hash_map::{DefaultHasher, Entry},
    HashMap,
  },
  hash::{Hash, Hasher},
  sync::{Arc, RwLock},
};

use uuid::Uuid;

/// Definition of a relationship between to nodes, from target to source
#[derive(Clone, Debug)]
pub struct Edge<G>
where
  G: Graph,
{
  guid: Uuid,

  /// A pointer to the left side node
  source: Node<G>,

  target: Node<G>,

  properties: Arc<G::Edge>,

  /// An optional weight that can be used for sorting
  options: EdgeOpts,
}

impl<G> Edge<G>
where
  G: Graph,
{
  pub fn new(source: &Node<G>, target: &Node<G>, properties: G::Edge) -> Edge<G> {
    Edge {
      guid: Self::make_guid(source.get_guid(), target.get_guid(), properties.get_key()),
      source: source.clone(),
      target: target.clone(),
      properties: Arc::new(properties),
      options: EdgeOpts::new(),
    }
  }

  pub fn get_guid(&self) -> Uuid {
    self.guid.clone()
  }
}
impl<G> Edge<G>
where
  G: Graph,
{
  fn make_guid(source: Uuid, target: Uuid, key: Uuid) -> Uuid {
    let mut hasher = DefaultHasher::new();
    hasher.write(source.as_bytes());
    hasher.write(target.as_bytes());
    hasher.write(key.as_bytes());
    Uuid::new_v5(&Uuid::NAMESPACE_OID, &hasher.finish().to_ne_bytes())
  }

  pub fn get_key(&self) -> Uuid {
    self.properties.get_key()
  }

  pub fn get_source(&self) -> Node<G> {
    self.source.clone()
  }

  pub fn get_target(&self) -> Node<G> {
    self.target.clone()
  }

  pub fn get_properties(&self) -> Arc<G::Edge> {
    self.properties.clone()
  }

  pub fn get_type_label(&self) -> String {
    self.properties.get_type_label()
  }
}

impl<G> std::cmp::PartialEq for Edge<G>
where
  G: Graph,
{
  fn eq(&self, other: &Self) -> bool {
    // If the keys are equal, their internals must be as well
    self.get_key() == other.get_key()
  }
}

impl<G> Hash for Edge<G>
where
  G: Graph,
{
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.get_properties().hash(state)
  }
}

/// Generic options that can be placed on each edge
#[derive(Debug, Clone)]
pub struct EdgeOpts {
  /// An optional weight that can be used order similar edges
  weight: Option<f32>,

  /// Whether there is a reciprocal edge going from target to source
  direction: EdgeDirection,
}

impl EdgeOpts {
  pub fn new() -> EdgeOpts {
    EdgeOpts {
      weight: None,
      direction: EdgeDirection::OneWay,
    }
  }
}

/// Whether the edge is from the source to the target
///
/// Any bi-directional edges should have paired entries, one for each direction. Undirected means
/// we can use the edge to go from target to source, if needed.
#[derive(Clone, Debug)]
pub enum EdgeDirection {
  /// This edge goes only one way
  OneWay,

  /// An edge that can also start from the target and go to the source.
  ///
  /// We don't necessarily want this to be automatic, as the weight might be different for each
  /// direction. This is more of a tag to let the user/graph know that there is another edge going
  /// the opposite direction.
  TwoWay,
}

/// A searchable index for related nodes
///
#[derive(Debug, Clone)]
pub struct EdgeMap<G>
where
  G: Graph,
{
  // Look up all the targets that have this label
  by_label: HashMap<String, HashMap<Uuid, Edge<G>>>,
  // by_type: HashMap<E::Edge, Arc<RwLock<Edge<G::Edge>>>>,
}

impl<G> EdgeMap<G>
where
  G: Graph,
{
  pub fn new() -> EdgeMap<G> {
    EdgeMap {
      by_label: HashMap::new(),
    }
  }

  pub fn find_all(&self, matcher: &str) -> Vec<Edge<G>> {
    let mut result = Vec::new();
    match self.by_label.get(matcher) {
      Some(edges) => {
        info!("Looking up the values by label {}", matcher);
        for v in edges.values() {
          result.push(v.clone());
        }
      }
      None => (),
    }
    result
  }

  pub fn insert(&mut self, edge: Edge<G>) -> GraphtResult<()> {
    match self.by_label.entry(edge.get_type_label()) {
      Entry::Vacant(entry) => {
        let mut targets = HashMap::new();
        targets.insert(edge.get_target().get_guid(), edge);
        // info!("Inserting new targets: {:?}", targets);
        entry.insert(targets);
        Ok(())
      }
      Entry::Occupied(mut targets) => match targets.get_mut().entry(edge.target.get_guid()) {
        Entry::Vacant(entry) => {
          // info!("Adding existing targets {:?}: {:?}", entry, edge);
          entry.insert(edge);
          Ok(())
        }
        Entry::Occupied(_) => Err(err!(
          DuplicateKey,
          "Duplicate return value {}. Please drop the existing value before trying to add it",
          edge.get_type_label()
        )),
      },
    }
  }
}
