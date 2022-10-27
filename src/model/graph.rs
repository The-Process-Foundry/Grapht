//! A holistic view of a full graph

use crate::{local::*, prelude::*};

/// Define all the possible nodes and edges available in a graph
pub trait Graph: Debug + Clone + PartialEq {
  type Node: GraphtEntity;
  type Edge: GraphtEntity;

  /// Load the Graph with initial data, if needed
  ///
  /// This is a place where we can add seed data to the graph, if desired. Some useful cases:
  /// - Testing data
  /// - Example code build on top of a graph
  /// - Normalizing system constants
  fn bootstrap(_data_set: &mut DataSet<Self>) -> GraphtResult<Stats> {
    warn!(
      "There is no bootstrap required for Graph {:?}",
      std::any::type_name::<Self>()
    );
    Ok(Stats::new())
  }
}

/// The basic building blocks of data in a graph
pub enum GraphItem<G>
where
  G: Graph,
{
  /// Used as either a placeholder or an empty set
  Null,

  Node(Node<G>),

  Edge(Edge<G>),

  /// An ordered list of edges
  ///
  /// This is either Null, a single node, or a list of edges. All other items are invalid
  Path(Vec<GraphItem<G>>),
}
