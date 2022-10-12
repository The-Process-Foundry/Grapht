//! Implementation of a node in a graph

use crate::local::*;
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
  pub fn add_edge(&mut self, target: Node<G>, props: G::Edge) -> GraphtResult<Edge<G>> {
    // Create the new edge from current node to the target
    let edge: Edge<G> = Edge::new(&self, &target.into(), props.into());

    // Add the new edge to the map by its label
    let mut inner = self.inner.write().unwrap();
    inner.edges.insert(edge.clone())?;
    Ok(edge)
  }

  pub fn edges_by_label(&self, matcher: &str) -> Vec<Edge<G>> {
    let inner = self.inner.read().unwrap();
    inner.edges.find_all(&matcher)
  }

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
  pub fn to_create(&self, max_depth: Option<u16>) -> GraphtResult<String> {
    // Track the tags and nodes seen
    let mut counter = EntityCounter::new();
    // All the created node strings
    let mut nodes: Vec<String> = Vec::new();

    // Edge info
    let labels = vec!["__OrganizationParent"];
    let mut edges: Vec<Edge<G>> = Vec::new();

    // A queue of nodes that haven't been run yet
    let mut unprocessed = vec![self.clone()];

    while unprocessed.len() != 0 {
      if let Some(node) = unprocessed.pop() {
        // Get the tag and continue if it's already been created
        let tag = match counter.get_tag("Organization", node.get_guid()) {
          (false, _) => continue,
          (true, tag) => tag,
        };

        // Render the node
        info!("Created new tag {:?}", tag);
        nodes.push(node.to_gql(Some(&tag))?);

        // Add all the edges to edges
        for label in &labels {
          for edge in node.edges_by_label(label.clone()) {
            unprocessed.push(edge.get_target());
            edges.push(edge);
          }
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
        "{}, ({})-[:OrganizationParent]->({})",
        query,
        counter
          .get_tag("Organization", edge.get_source().get_guid())
          .1,
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

/// Wrapped contents of a node
#[derive(Debug, Clone)]
pub struct InnerNode<G>
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
  pub(self) edges: EdgeMap<G>,
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
      edges: EdgeMap::new(),
    }
  }

  pub fn has_label(&self, label: &str) -> bool {
    self.labels.contains(label)
  }

  pub fn add_label(&mut self, label: &str) -> bool {
    self.labels.insert(label.to_string())
  }

  pub fn drop_label(&mut self, label: &str) -> bool {
    self.labels.remove(label)
  }

  pub fn type_label(&self) -> String {
    self.properties.get_type_label()
  }

  pub fn get_props(&self) -> Arc<G::Node> {
    self.properties.clone()
  }
}