//! Test internal data structures

use grapht::{err, prelude::*};

#[macro_use]
mod common;
use common::invoicer::*;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::{debug, info};

/*
  Next
  Build Query:
  Insert:
  - Nodes
  - Edges
  - Paths
    Tests
    - Guid consistency (Same props, same edges, same nodes, etc.)
  - Stats:
    - Action Stats: Mutations caused by queries and calls
  - Indices
    - Labels
    - Query pieces as keys?
    - Aggregates
*/

db_test_fn! {
  fn test_create_nodes() {
    let mut data_set: DataSet<FhlGraph> = DataSet::new();
    let mut expected = Stats::default();


    // We will create 3 new labels to go with the type

    // Create 10 nodes and insert them individually, showing that it works
    let mut orgs: Vec<Node<FhlGraph>> = Vec::new();
    for i in 1..10 {
      let mut org =
      Node::new(FhlNode::Organization(Organization::new(
        &format!("pretty_{}", i)[..],
        &format!("Org {}", i)[..],
        dec!(0),
      )));

      // Add an extra label, so we can see
      let _ = org.add_label(&format!("Test_Label_{}", i % 3));
      orgs.push(org.clone());
    }

    for i in 0..9 {
      let org = orgs.get(i).unwrap();

      // let result = data_set.insert(GraphItem::Node(org));
      let result = data_set.insert(org.clone());

      expected.nodes.created = 1;
      assert_eq!(expected, result.expect("Received an error inserting a new node"));

      expected.nodes.created = (i + 1) as u128;
      assert_eq!(expected, data_set.stats(), );

    }

    // Adding duplicate returns error (duplicate key)
    for org in &orgs {
      assert_eq!(
        data_set.insert(org.clone()),
        Err(err!(
          DuplicateKey,
          "Node with Uuid {} already exists in the graph {}",
          org.get_guid(),
          data_set.get_guid()
        ))
      );
    }
  }
}

db_test_fn! {
  fn test_create_edges() {
    let data_set: DataSet<FhlGraph> = DataSet::new();
    let mut expected = Stats::default();

    // let orgs = Vec::new();

    // Create 10 nodes and insert them individually, showing that it works
    let mut orgs: Vec<Node<FhlGraph>> = Vec::new();
    for i in 1..10 {
      let org =
      Node::new(FhlNode::Organization(Organization::new(
        &format!("pretty_{}", i)[..],
        &format!("Org {}", i)[..],
        dec!(0),
      )));
      orgs.push(org.clone());
    }

    // Empty data set
    assert_eq!(data_set.stats(), expected);

    // Now create child orgs for each
    for i in 1..9 {
      // let result =
    }

    // Insert edge with no new nodes.
    // Insert edge with one new node.
    // Insert edge with two new nodes.


    // Insert new node with edges pointing to new nodes


    // Insert one node and see all children/edges get entered

    // Make an edge between already inserted nodes and see insert still works




  }
}

db_test_fn! {
  fn test_create_paths() {
    let data_set: DataSet<FhlGraph> = DataSet::new();

  }
}

db_test_fn! {
fn test_query_orgs() {
  let data_set: DataSet<FhlGraph> = DataSet::new();


;
}}
