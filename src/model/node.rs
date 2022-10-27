//! Implementation of a node in a graph

use crate::{local::*, prelude::*};
use sync::{Arc, RwLock};

use std::collections::{
  hash_map::Entry,
  {HashMap, HashSet},
};

use uuid::Uuid;

// pub trait GraphtNode: Into<Node<Self>> + Clone {
//   /// This is the full set of edges available to the node
//   type Edges: GraphtEdge;

//   fn get_guid(&self) -> Uuid;

//   fn get_type_label(&self) -> String;

//   /// Convert the properties into a JSON type string
//   fn to_gql(&self) -> String;

//   /// Take the result of a query and convert it back into a node
//   fn from_gql(_value: &str) -> Self {
//     todo!("from_gql isn't written yet")
//   }
// }

/// A record in a graph
#[derive(Debug, Clone)]
pub struct Node<G>
where
  G: Graph,
{
  guid: Uuid,

  inner: Arc<RwLock<InnerNode<G>>>,
}

impl<G> PartialEq for Node<G>
where
  G: Graph,
{
  fn eq(&self, other: &Self) -> bool {
    self.guid == other.guid
  }
}

impl<G> Node<G>
where
  G: Graph,
{
  pub fn new(props: G::Node) -> Node<G> {
    Node {
      guid: props.get_key(),
      inner: Arc::new(RwLock::new(InnerNode::new(props))),
    }
  }

  pub fn get_guid(&self) -> Uuid {
    self.guid.clone()
  }

  pub fn has_label(&self, label: &str) -> bool {
    self.inner.read().unwrap().labels.contains(label)
  }

  pub fn add_label(&mut self, label: &str) -> bool {
    self.inner.write().unwrap().labels.insert(label.to_string())
  }

  pub fn drop_label(&mut self, label: &str) -> bool {
    self.inner.write().unwrap().labels.remove(label)
  }

  pub fn get_labels(&self) -> HashSet<String> {
    self.inner.read().unwrap().labels.clone()
  }

  pub fn type_label(&self) -> String {
    self.inner.read().unwrap().properties.get_type_label()
  }

  pub fn get_props(&self) -> Arc<G::Node> {
    self.inner.read().unwrap().properties.clone()
  }

  // ---- Edge/Path functions
  /// Create a relationship between the current node and the target using the given payload
  pub fn create_edge(&mut self, props: G::Edge, target: Node<G>) -> GraphtResult<Edge<G>> {
    // Create the new edge from current node to the target
    let edge: Edge<G> = Edge::new(&self, &target.into(), props.into());

    self.add_edge(edge.clone())?;
    Ok(edge)
  }

  pub(crate) fn add_edge(&mut self, edge: Edge<G>) -> GraphtResult<()> {
    // Add the new edge to the map by its label
    let mut inner = self.inner.write().unwrap();
    inner.edges.insert(edge.into())?;
    Ok(())
  }

  /// Retrieve a list of all the edges connecting this node
  ///
  /// FIXME: Make the query mean something
  pub fn edges(&self, _query: &str) -> Vec<Edge<G>> {
    self
      .inner
      .read()
      .unwrap()
      .edges
      .all()
      .iter()
      .map(|x| x.into())
      .collect()
  }

  // pub fn find_edges(&self, filter: &str) -> Vec<Edge<G>> {
  //   let inner = self.inner.read().unwrap();
  //   inner.edges.find_all(filter)
  // }

  /// Get all related entities via any path
  /// THINK: Is this different than matching a wildcard path

  //---   Query functionality

  // Serialize the key/values into a format that GQuery recognizes
  fn to_gql(&self, name: Option<&str>) -> GraphtResult<String> {
    let labels = self
      .get_labels()
      .iter()
      .fold("".to_string(), |acc, label| format!("{}:{}", acc, label));

    Ok(format!(
      "({}{} {})",
      name.unwrap_or(""),
      labels,
      self.get_props().to_gql()?
    ))
  }

  /// make a CREATE query for all node and all related edges and nodes
  ///
  /// FIXME: This needs to move into the GQuery AST. It should take a query result such as
  /// "(nodes)->[edges *0..]" and then all the named spots are decomposed into a distinct list of
  /// nodes, edges, and paths. From there, its a simple iteration to make the create query string.
  pub fn to_create(&self, _max_depth: Option<u16>) -> GraphtResult<String> {
    // Track the tags and nodes seen
    let mut counter = EntityCounter::new();
    // All the created node strings
    let mut nodes: Vec<String> = Vec::new();

    // Edge info
    // let labels = self.get_labels();
    let mut edges: Vec<Edge<G>> = Vec::new();

    // A queue of nodes that haven't been run yet
    let mut unprocessed = vec![self.clone()];

    while !unprocessed.is_empty() {
      if let Some(node) = unprocessed.pop() {
        // Get the tag and continue if it's already been created
        let tag = match counter.get_tag("Organization", node.get_guid()) {
          (false, _) => continue,
          (true, tag) => tag,
        };

        // Add the node to processed
        nodes.push(node.to_gql(Some(&tag))?);

        // Add all the edges their targets to the unprocessed queue
        for edge in node.edges("()->[]") {
          unprocessed.push(edge.get_target());
          edges.push(edge);
        }
      }
    }

    let mut query = format!("CREATE {}", nodes.pop().unwrap());
    for node in nodes {
      query = format!("{}, {}", query, node)
    }

    for edge in edges {
      // Now we append all the edges
      query = format!(
        "{}, ({})-[:{} {}]->({})",
        query,
        counter
          .get_tag("Organization", edge.get_source().get_guid())
          .1,
        edge.get_label(),
        edge.get_properties().to_gql()?,
        counter
          .get_tag("Organization", edge.get_target().get_guid())
          .1
      );
    }

    info!("Built query\n{}", query);
    Ok(query)
  }
}

/// A helper for creating query strings for nodes
struct EntityCounter {
  /// A count of entities in each label
  labels: HashMap<String, i32>,
  /// A pointer from each entity to their type/count id
  entities: HashMap<Uuid, HashMap<String, String>>,
}

impl EntityCounter {
  pub fn new() -> EntityCounter {
    EntityCounter {
      labels: HashMap::new(),
      entities: HashMap::new(),
    }
  }

  /// Spits out the name_type of the node
  pub fn get_tag(&mut self, label: &str, guid: Uuid) -> (bool, String) {
    let b_label = label.to_string();
    let counter = || match self.labels.entry(b_label) {
      Entry::Vacant(entry) => {
        entry.insert(1);
        1
      }
      Entry::Occupied(mut entry) => {
        entry.insert(entry.get() + 1);
        entry.get().clone()
      }
    };

    // Check the uuid if it is already entered and reuse any existing tags
    match self.entities.entry(guid) {
      Entry::Vacant(entry) => {
        let mut mapping = HashMap::new();
        let tag = format!("{}{}", label.clone(), counter());
        mapping.insert(label.to_string(), tag.clone());
        entry.insert(mapping);
        (true, tag)
      }
      Entry::Occupied(mut entry) => {
        let mapping = entry.get_mut();
        match mapping.entry(label.to_string()) {
          Entry::Vacant(entry) => {
            let tag = format!("{}{}", label, counter());
            entry.insert(tag.clone());
            (true, tag)
          }
          Entry::Occupied(entry) => (false, entry.get().clone()),
        }
      }
    }
  }
}

/// Internal contents of a node
#[derive(Debug, Clone)]
pub(crate) struct InnerNode<G>
where
  G: Graph,
{
  /// An entity containing the properties assigned to this node
  ///
  /// Using Arc to make sure that it copies on write. Only the graph should be able to modify a
  /// node, everybody else should share a copy.
  pub(self) properties: Arc<G::Node>,

  /// Searchable tags for the nodes
  pub(self) labels: HashSet<String>,

  /// A lookup for the edges that have this node as a starting point
  pub(self) edges: Lookup<G>,
  // Calculations based on the current values of the node and its edges
  // pub(self) aggregates: HashMap<String, Aggregate>,
}

impl<G> InnerNode<G>
where
  G: Graph,
{
  pub fn new(props: G::Node) -> InnerNode<G> {
    // Make sure the type label is always included
    let mut labels = HashSet::new();
    let _ = labels.insert(props.get_type_label());

    InnerNode {
      properties: Arc::new(props),
      labels,
      edges: Lookup::new(Index::Edge("Self".to_string())),
    }
  }

  /*
  pub fn has_label(&self, label: &str) -> bool {
    self.labels.contains(label)
  }

  pub fn add_label(&mut self, label: &str) -> bool {
    self.labels.insert(label.to_string())
  }

  pub fn drop_label(&mut self, label: &str) -> bool {
    self.labels.remove(label)
  }

  /// Add an already created edge to the node
  pub fn add_edge_unchecked(&mut self, edge: Edge<G>) -> GraphtResult<()> {
    self.edges.insert(edge.into())
  }

  pub fn type_label(&self) -> String {
    self.properties.get_type_label()
  }

  pub fn get_props(&self) -> Arc<G::Node> {
    self.properties.clone()
  }
  */
}
