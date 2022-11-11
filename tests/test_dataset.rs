//! Test internal data structures

use grapht::{err, prelude::*};

#[macro_use]
mod common;
use common::invoicer::*;

// use rust_decimal::Decimal;
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

//
macro_rules! node {
  ($graph:ty, $node_type:ident, $($param:expr),+) => {
    Node::<$graph>::new($node_type::new($($param), +).into())
  };
}

macro_rules! edge {
  ($edge_type:expr $(, $($param:expr),*)?) => {
    FhlEdge::new($edge_type $( ( $($param), * ) )?.into())
  };
}

db_test_fn! {
  fn test_create_nodes() {
    let mut data_set: DataSet<FhlGraph> = DataSet::new();

    // Counts of items in the DataSet
    let mut data_set_stats = DataSetStats::default();

    // Counts of actions performed by the insert
    let mut insert_stats = CrudResultStats::<DataSetStats>::default();

    // Data Set is totally empty
    assert_eq!(data_set.stats(), data_set_stats);

    // Create a root org
    info!("   ---> Creating the root node and inserting");
    let mut root: Node<FhlGraph> = node!(
      FhlGraph, Organization, "RootNode", "Ruler of all the Nodes", dec!(0)
    );

    root.add_label("RootOrganization");

    // And insert it
    let result = data_set.insert(root.clone().into()).expect("Failed to insert the root node");

    // insert_stats.nodes.created = 1;
    // insert_stats.labels.created = 2;
    insert_stats.assert_eq(&result);

    // data_set_stats.nodes.created = 1;
    // data_set_stats.labels.created = 2;
    // data_set_stats.assert_eq(&data_set.stats());

    info!("  --->Attempting to insert the root node a second time");
    let result = data_set.insert(root.clone().into()).expect("Failed to insert the root node");

    // insert_stats.nodes.created = 0;
    // insert_stats.labels.created = 0;
    insert_stats.assert_eq(&result);
    // data_set_stats.assert_eq(&data_set.stats());

    info!("\n\n  ---> Add a single child with an edge from the root");
    let heir = node!(
      FhlGraph, Organization, "Heir", "Prince of Nodes", dec!(0)
    );

    let edged = root.create_edge(edge!(FhlEdgeType::ParentOf), heir.clone()).expect("Could not create the new edge");

    debug!("Try to Insert root again and still a duplicate though it has changed");
    let result = data_set.insert(root.clone().into()).expect("Failed to insert the root node");

    // insert_stats.nodes.created = 0;
    // insert_stats.labels.created = 0;
    insert_stats.assert_eq(&result);
    // data_set_stats.assert_eq(&data_set.stats());

    info!("\n\n  ---> Adding the child using the edge and now edge and child are now in the set");
    let result = data_set.insert(edged.clone().into()).expect("Failed to insert the edge");

    // insert_stats.nodes.created = 1;
    // insert_stats.nodes.updated = 1;
    // insert_stats.edges.created = 1;
    // insert_stats.labels.created = 0;
    insert_stats.assert_eq(&result);

    // data_set_stats.nodes.created = 2;
    // data_set_stats.edges.created = 1;
    // data_set_stats.assert_eq(&data_set.stats());


    // Make a child with the root with 10 grandchildren, add the edge root->child to add all of them

    // Make an empty set

    // Add root - stats should be the same as the previous item


  //   for i in 0..9 {
  //     let org = orgs.get(i).unwrap();

  //     // let result = data_set.insert(GraphItem::Node(org));
  //     let result = data_set.insert(org.clone());

  //     expected.nodes.created = 1;
  //     assert_eq!(expected, result.expect("Received an error inserting a new node"));

  //     expected.nodes.created = (i + 1) as u128;
  //     assert_eq!(expected, data_set.stats(), );

  //   }

  //   // Adding duplicate returns error (duplicate key)
  //   for org in &orgs {
  //     assert_eq!(
  //       data_set.insert(org.clone()),
  //       Err(err!(
  //         DuplicateKey,
  //         "Node with Uuid {} already exists in the graph {}",
  //         org.get_guid(),
  //         data_set.get_guid()
  //       ))
  //     );
  //   }
  }
}

db_test_fn! {
  fn test_create_edges() {
    let data_set: DataSet<FhlGraph> = DataSet::new();
    // let mut expected = Stats::default();

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
    // assert_eq!(data_set.stats(), expected);

    // expected.nodes.created = 1;
    // Now create child orgs for each
    for _i in 1..9 {
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
  fn test_insert() {

  }
}

db_test_fn! {
  fn test_create_paths() {
    let _data_set: DataSet<FhlGraph> = DataSet::new();

  }
}

db_test_fn! {
fn test_query_orgs() {
  let _data_set: DataSet<FhlGraph> = DataSet::new();
}}
