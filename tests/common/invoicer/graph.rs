//! Graph data stored in Redis

use super::fhl_prelude::*;

use std::{
  collections::{HashMap, HashSet},
  hash::{Hash, Hasher},
  sync::{Arc, Mutex},
};

// use grapht::backends::redis_graph::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::info;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct FhlGraph;

impl Graph for FhlGraph {
  type Node = FhlNode;
  type Edge = FhlEdge;
}

/// Wrappers for the various node entity types that FHL knows about
///
/// TODO: Make a macro for this, as it should always be an Enum here
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum FhlNode {
  Organization(Organization),
  Submission,
  Payment,
  Deliverable,
  Invoice,
}

impl GraphtEntity for FhlNode {
  fn get_type_label(&self) -> String {
    match self {
      FhlNode::Organization(_) => "__Organization",
      FhlNode::Submission => "",
      FhlNode::Payment => "",
      FhlNode::Deliverable => "",
      FhlNode::Invoice => "",
    }
    .to_string()
  }

  fn get_key(&self) -> Uuid {
    match &self {
      FhlNode::Organization(org) => org.get_key(),
      _ => todo!(
        "Still need to do the other FhlNode types: {}",
        self.get_type_label()
      ),
    }
  }

  fn to_gql(&self) -> GraphtResult<String> {
    match self {
      FhlNode::Organization(org) => org.to_gql(),
      _ => todo!("No to_gql yet for other nodes: {:?}", self),
    }
  }

  fn get_inner<T: Clone + core::fmt::Debug>(&self) -> std::borrow::Cow<T> {
    todo!()
  }

  fn from_gql(value: &[u8]) -> GraphtResult<Self> {
    todo!()
  }
}

impl From<FhlNode> for Node<FhlGraph> {
  fn from(node: FhlNode) -> Self {
    Node::new(node)
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FhlEdge {
  guid: Uuid,
  entity: FhlEdgeType,
}

impl FhlEdge {
  pub fn new(entity: FhlEdgeType) -> FhlEdge {
    FhlEdge {
      guid: Uuid::new_v4(),
      entity,
    }
  }
}

impl GraphtEntity for FhlEdge {
  fn get_type_label(&self) -> String {
    match self.entity {
      FhlEdgeType::OrganizationParent => "__OrganizationParent",
      FhlEdgeType::OrganizationChild => "__OrganizationChild",
      _ => todo!("Type Labels for FHLEdge: {:?}", &self),
    }
    .to_string()
  }

  fn get_key(&self) -> Uuid {
    todo!()
  }

  fn get_inner<T: Clone + core::fmt::Debug>(&self) -> std::borrow::Cow<T> {
    todo!()
  }

  fn to_gql(&self) -> GraphtResult<String> {
    todo!()
  }

  fn from_gql(value: &[u8]) -> GraphtResult<Self> {
    todo!()
  }
}

impl From<FhlEdgeType> for FhlEdge {
  fn from(entity: FhlEdgeType) -> Self {
    FhlEdge::new(entity)
  }
}

/// Wrappers for the various edge types that FHL knows about and included payload
#[derive(Hash, Clone, Debug, Eq, PartialEq)]
pub enum FhlEdgeType {
  /// Pointer to the parent organization. Null is allowed
  OrganizationParent,

  /// Pointer to one of the children of the organization
  OrganizationChild,

  /// A payment made to an organization
  OrganizationPayment,

  /// A billable unit of work
  OrganizationDeliverable,

  /// A set of items Payments, Deliverables, and Unpaid Invoices for a given organization
  OrganizationInvoice,
}

/* TODO: Add the backend in when ready

/// A Graph is the pool of all nodes/edges and manages querying them
#[derive(Debug, Clone)]
pub struct FhlGraph {
  name: String,
  url: String,
  db: Arc<Grapht<FhlGraph>>,
}

impl FhlGraph {
  // Configures
  pub fn new(name: &str, url: &str) -> GraphtResult<FhlGraph> {
    // HACK: FhlGraph should exist as a singleton outside of this graph
    let graph = Grapht::new();


    // Set up the graph if it doesn't exist
    match graph.get_backend(name) {
      Err(err) => {
        if err.is(gquery::errors::Kind::NotFound) {
          info!("Creating new Redis backend");
          let backend = RedisGraph::new(RedisGraphConfig::new(name, url));
          graph
            .add_backend(name, BackendImpl::RedisGraph(backend))
            .expect("Failed to add backend");
        } else {
          panic!(
            "TypeMismatch: Expected a RedisGraph backend, but received {:?}",
            err
          )
        }
      }

      Ok(backend) => {
        if let BackendImpl::RedisGraph(_) = &*backend.read().unwrap() {
          info!("Graph already created, checking connection");
        } else {
          panic!(
            "TypeMismatch: Expected a RedisGraph backend, but received {:?}",
            backend
          )
        }
      }
    };


    Ok(FhlGraph {
      name: name.to_string(),
      url: url.to_string(),
      db: Arc::new(graph),
    })
  }

  /// Clear all the data out of the current graph
  /// THINK: Since this is warehouse, should it push the data to watchers by default
  ///
  /// Force: When true, this will clear all the watchers for this graph and push the changes otherwise
  /// TODO: Create stats for dropped items
  pub fn clear(&self, force: bool) -> GraphtResult<()> {
    warn!("Clearing the graph");

    // Try to get the graph
    match self.db.get_backend(&self.name) {
      Ok(backend) => {
        // Delete the graph
        let dropped = self.db.send(&self.name, "MATCH (n) DETACH DELETE n");
        warn!("Delete Result:\n{:?}", dropped);
      }
      Err(err) if err.is(Kind::NotFound) => {
        info!("There was no backend named {}. Creating it now", self.name);
      }
      Err(err) => Err(err)?,
    }
    Ok(())
  }

  pub fn query(&self, query: &str) -> GraphtResult<QuerySet> {
    info!("Sending Query: {}", query);
    let result = self.db.send(&self.name, query);
    info!("Received:\n{:?}", result);
    result
  }

  /// Send a node and all its related elements to the backend
  pub fn store(&self, node: Node<FhlGraph>) -> GraphtResult<QuerySet> {
    self.query(&node.to_create(None)?[..])
  }

  /// Create some basic data in an FhlGraph graph in the given namespace
  pub fn bootstrap(grapht: &mut Grapht<FhlGraph>, name: &str) -> GraphtResult<()> {
    /// Create a child for the bootstrapped org
    fn add_child_org(parent: &mut Node<FhlGraph>, index: u32) -> Node<FhlGraph> {
      if let FhlNode::Organization(entity) = &*parent.get_props() {
        let child = Node::new(FhlNode::Organization(Organization::new(
          &format!("{}_{}", entity.pretty_id, index)[..],
          &format!("SubOrg {}, Child of {} ", index, entity.org_name)[..],
          entity.balance * Decimal::from(index),
        )));

        parent.add_edge(child.clone(), FhlEdge::new(FhlEdgeType::OrganizationParent));

        return child;
      } else {
        panic!("add_child_org received a non-organization node")
      }
    }

    info!("\n\t---------------\nRunning the internal bootstrap");
    // Create/Clear the graph
    self.clear(true)?;

    // Temporary data for testing. This should be in a Grapht::QuerySet
    for i in 1..10 {
      // Do each root individually.
      let mut org: Node<FhlGraph> = Node::new(FhlNode::Organization(Organization::new(
        &format!("pretty_{}", i)[..],
        &format!("Org {}", i)[..],
        dec!(0),
      )));
      org.add_label("RootOrg");

      for j in 1..10 {
        let mut child = add_child_org(&mut org, j);

        for k in 1..10 {
          let _ = add_child_org(&mut child, k);
        }
      }

      // Add the node with all children
      let _ = self.store(org)?;
    }

    info!("\n\t--------------\nFinished Bootstrap Insert. Querying graph\n\n");
    info!("{:?}", self.query("MATCH (n) RETURN n"));
    Ok(())
  }

  /// Get a pointer to the primary graph
  pub fn graph(&self) -> Arc<Grapht> {
    self.db.clone()
  }
}

*/
