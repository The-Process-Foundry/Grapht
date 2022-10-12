//! A sequence of nodes connected to nodes via edges

use crate::err;
use crate::local::*;

use std::{
  collections::{hash_map::DefaultHasher, HashSet},
  hash::Hasher,
};

use uuid::Uuid;

lazy_static::lazy_static! {
  /// This is a seed for all path guids.
  static ref NIL_PATH_GUID: Uuid = {
    let mut hasher = DefaultHasher::new();
    hasher.write("PathGuid:".as_bytes());
    Uuid::new_v5(&Uuid::NAMESPACE_OID, &hasher.finish().to_ne_bytes())
  };
}

/// A immutable set of steps showing a unique path through a graph
#[derive(Debug, Clone)]
pub struct Path<G>
where
  G: Graph,
{
  config: Option<PathConfig>,
  guid: Uuid,
  steps: Vec<PathStep<G>>,
}

impl<G> Path<G>
where
  G: Graph,
{
  pub fn new(start: Option<Node<G>>, steps: Vec<Edge<G>>) -> Path<G> {
    // Create an empty path
    let mut path = Path {
      config: None,
      guid: NIL_PATH_GUID.clone(),
      steps: vec![PathStep::Nil],
    };

    // Add the node as the first element in the path, if it was included
    if let Some(node) = start {
      path
        .add_step(PathStep::Node(node))
        .expect("Error making new path");
    };

    // Add all the additional edges
    for edge in steps {
      path
        .add_step(PathStep::Edge(edge))
        .expect("Could not create path - error adding step");
    }
    path
  }

  /// Get the first node in the path, if it exists
  pub fn start(&self) -> Option<Node<G>> {
    if let Some(first) = self.steps.first() {
      match first {
        PathStep::Nil => None,
        PathStep::Node(node) => Some(node.clone()),
        PathStep::Edge(edge) => Some(edge.get_source()),
      }
    } else {
      None
    }
  }

  /// Get the final node in the path, if it exists
  pub fn end(&self) -> Option<Node<G>> {
    if let Some(last) = self.steps.last() {
      match last {
        PathStep::Nil => None,
        PathStep::Node(node) => Some(node.clone()),
        PathStep::Edge(edge) => Some(edge.get_source()),
      }
    } else {
      None
    }
  }

  /// An internal method for building a path via mutation.
  fn add_step(&mut self, step: PathStep<G>) -> GraphtResult<()> {
    let mut hasher = DefaultHasher::new();

    // Get the current guid:
    match &step {
      // Nothing to be done for the identity
      PathStep::Nil => return Ok(()),

      // A node can only be a single entity path, with no
      PathStep::Node(node) => match self.steps.first() {
        Some(PathStep::Nil) | None => {
          assert!(
            self.steps.len() > 1,
            "The path must only contain a path step of Nil to add a node"
          );
          self.steps = vec![PathStep::Node(node.clone())];
          hasher.write(node.get_guid().as_bytes());
        }
        _ => {
          // This case doesn't make sense, as there is no edge
          return Err(err!(
            InvalidItem,
            "Tried to add node {:?} to existing path {:?}",
            step,
            self.steps
          ));
        }
      },
      PathStep::Edge(edge) => {
        // Make sure the next edge creates a continuous path
        match self.end() {
          None => (),
          Some(end) => {
            if end.get_guid() == edge.get_source().get_guid() {
              return Err(err!(InvalidItem, "Attempting to add a non-continuous edge to a path:\n\tCurrent End: {:?}\n\tNew Edge: {:?}", end, edge));
            }
          }
        }

        // Clear out nil/node if they are the only step
        match self.steps.get(0) {
          Some(PathStep::Nil) | Some(PathStep::Node(_)) => {
            assert!(
              self.steps.len() > 1,
              "Path length is not 1 yet has a first node of nil or node"
            );
            self.steps = Vec::new();
            hasher.write(NIL_PATH_GUID.as_bytes());
          }
          _ => hasher.write(self.guid.as_bytes()),
        }

        // And add the new edge
        self.steps.push(step);
      }
    }

    // Calculate the new guid
    // path.guid = Uuid::new_v5(&Uuid::NAMESPACE_OID, &hasher.finish().to_ne_bytes());
    self.guid = Uuid::new_v5(&Uuid::NAMESPACE_OID, &hasher.finish().to_ne_bytes());
    Ok(())
  }

  /// Concatenate steps with the current path, creating a longer one
  pub fn add_steps(&self, steps: Vec<Edge<G>>) -> GraphtResult<Path<G>> {
    let mut path = self.clone();
    for step in steps {
      path.add_step(PathStep::Edge(step))?
    }

    Ok(path)
  }

  pub fn get_guid(&self) -> Uuid {
    self.guid.clone()
  }
}

#[derive(Debug, Clone)]
pub enum PathStep<G>
where
  G: Graph,
{
  /// An empty path
  Nil,

  /// A path of only one node or the end of a node
  Node(Node<G>),

  /// A step from one node to the next along the path
  Edge(Edge<G>),
}

impl<G> PathStep<G> where G: Graph {}

#[derive(Debug, Clone)]
pub struct PathConfig {
  /// A query that sets bounds of the path
  /// TODO: This should be a gQuery
  query: String,
}
