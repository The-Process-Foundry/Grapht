//! A mutable subset of the data store
//!
//! The query_set also acts as a visitor to the response, able to parse

use std::{
  collections::{hash_map::Entry, HashMap, HashSet},
  hash::{Hash, Hasher},
};

use crate::prelude::*;
use rust_decimal::Decimal;
use tracing::info;

/// A type to convert the result of a query into and merge it into a query set
pub trait ValueParser {
  /// Add the value into the results
  fn parse(&self, query: &GQuery) -> GQResult<QuerySet>;
}

/// A set of pointers to a shared graph
#[derive(Debug, Clone)]
pub struct DataSet {
  query: GQuery,

  // Pointer to the common data source
  // grapht: Option<Arc<RwLock<GraphCache<G>>>,
  values: Vec<ValueType>,

  /// Logging data
  pub stats: HashSet<Statistic>,
}

impl QuerySet {
  pub fn new(query: GQuery) -> QuerySet {
    QuerySet {
      query,
      values: Vec::new(),
      stats: HashSet::new(),
    }
  }

  // pub fn from<T: ValueParser>(v: &T) -> GQResult<QuerySet> {
  //   v.parse(&mut query_set)?;
  //   Ok(query_set)
  // }

  // /// Update the set with the results of a new statement
  // pub fn upsert<T: ValueParser>(&mut self, v: T) -> GQResult<()> {
  //   v.upsert(self)?;
  //   Ok(())
  // }

  pub fn add_stat(&mut self, stat: Statistic) -> GQResult<()> {
    match self.stats.get(&stat) {
      Some(val) => {
        let added = match (val, &stat) {
          (Statistic::Nodes(left), Statistic::Nodes(right)) => Statistic::Nodes(left + right),
          (Statistic::NodesCreated(left), Statistic::NodesCreated(right)) => {
            Statistic::NodesCreated(left + right)
          }
          _ => todo!("Connot handle adding {:?} with {:?}", val, stat),
        };
        self.stats.insert(added)
      }
      None => self.stats.insert(stat),
    };
    Ok(())
  }

  pub fn check_stat(&self, stat: &Statistic) -> GQResult<bool> {
    if let Some(rhs) = self.stats.get(stat) {
      info!("Checking side effect: {:?} == {:?}", rhs, stat);
      Ok(stat == rhs)
    } else {
      Err(err!(
        NotFound,
        "There was no stat to compare {:?} with: {:?}",
        stat,
        self.stats
      ))
    }
  }
}

#[derive(Debug, Clone, Hash)]
pub enum ValueType {
  /// Un-encoded binary, all data that cannot be converted get turned into this
  Raw(u8),

  /// A decoded String
  String(String),

  Node(String),

  Edge(String),

  Path(String),

  /// An ordered list of values
  List(Vec<Box<ValueType>>),

  /// Key/value pairs
  Map(ValueHash),
}

/// Metadata about the query set
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statistic {
  /// Time it took to generate this query set
  ExecutionTime(Decimal),

  /// The total number of node pointers returned
  Nodes(i32),
  /// The total number of unique nodes
  UniqueNodes,
  /// Count of nodes that were generated
  NodesCreated(i32),
  NodesUpdated,
  NodesDeleted(i32),

  /// Number of unique tags added to various nodes
  /// FIXME: Is there a difference between created and added?
  LabelsAdded(i32),

  Edges,
  UniqueEdges,
  EdgesCreated(i32),
  EdgesUpdated,
  EdgesRemoved,

  // Entity Values
  /// How many node/edge properties were changed by the query
  PropertiesSet(i32),

  Paths,
}

impl Hash for Statistic {
  fn hash<H: Hasher>(&self, state: &mut H) {
    match self {
      Statistic::Nodes(_) => 100,
      Statistic::NodesCreated(_) => 101,
      Statistic::NodesDeleted(_) => 102,
      Statistic::LabelsAdded(_) => 110,
      Statistic::Edges => 200,
      Statistic::EdgesCreated(_) => 201,
      Statistic::PropertiesSet(_) => 301,
      Statistic::Paths => 400,
      Statistic::ExecutionTime(_) => 1000,
      _ => todo!("Hasher for Statistic is not ready yet: {:?}", self),
    }
    .hash(state);
  }
}

#[derive(Debug, Clone)]
pub struct ValueHash(HashMap<Box<ValueType>, Box<ValueType>>);

impl Hash for ValueHash {
  fn hash<H: Hasher>(&self, state: &mut H) {
    unimplemented!("Currently not planning on using a hashmap as a key for ValueType")
  }
}
