//! A holistic view of a full graph

use crate::local::*;

/// Define all the possible nodes and edges available in a graph
pub trait Graph: Debug + Clone {
  type Node: GraphtEntity;
  type Edge: GraphtEntity;
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
