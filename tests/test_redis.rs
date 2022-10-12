//! Test the redis backend

use grapht::prelude::*;

#[macro_use]
mod common;
use common::invoicer::*;

use tracing::{debug, info};

const REDIS_DB: &'static str = "FhlTest";
const REDIS_URL: &'static str = "redis://127.0.0.1:6379/";

db_test_fn! {
  fn test_bootstrap() {
    let data_set: DataSet<FhlGraph> = DataSet::new();

  }
}

/*

db_test_fn! {
fn test_redis() {
  // let db: Grapht<FhlGraph> = Grapht::new();

  // register the backend (Definition of the redis backend data)
  // db.add_backend(REDIS_DB, RedisGraph::new(REDIS_URL));

  // Get the existing data set - all orgs and their relationship to each other
  let data_set = db.query(r#"
    MATCH (:__Organization)-[r:__ParentOf|__ChildOf*0..1)-(org:__Organization)
    RETURN (n, r)
  "#).expect("Query did not successfully retrieve a data set");
}}
*/

/*
  **All items should use temporary exact string matches instead of parsing. Grammars come once PivoTable can actually display the org data as requested.**

  1) Get the DataSet working
    Steps:
      a) Grapht::query(base_query) -> GraphtResult<DataSet>
    Exposed
      a) DataSet::find(&self, query) -> Vec<Path>
      b) Stats - Edges, Nodes
      c) On drop: print warn level to trace as a placeholder

    Use Case:

    - create empty DataSet (outside of grapht)
    -
  2) Insert paths to dataset
    Steps:
      a) Bootstrap: Create standalone nodes and edges for orgs
      b) DataSet::insert(path, related) -> Always Ok.
        - Path is a node, edge, or path
        - related:
          true: follows all possible paths (nodes/edges)
          false: Inserts that item and that item only. Edges/Paths that are not already in the set
            will cause an error.
    Tests:
      None needed

  3) Add Aggregates to Node
    - Enum for each type and custom trait Aggregated (Debug, Clone, Ord, Eq)
  4) Mutations
    - Add the consolidated change/mutation list to a patch
  5) Add Messenger to update local Grapht (Same mechanism as will be used for updating backends)
    Steps:
      a) Add new messenger (Sender from Grapht, full channel )
      b) Add Set sender to Grapht
      c)
    Tests:
       - Duplicate entry (not in query)
       - Test dataset base: Add items to base so they are included in the base
       - Cleanup on Drop

  ...) Later
    - DataSet transactions - Do multiple operations before pushing the final result
    - DataSet subscriptions
    - Grammar for parsing Cypher
  New dataset
  for each insert on dataset, send insert message to grapht
  add nodes/edges to grapht
  echo insert command to redis

*/
