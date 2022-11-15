//! A wrapper for scanning, indexing, querying, and mutating items in a graph
//!
//! All DataSets
//!
//! THINK:
//! - How do datasets relate? When updating a node, how do all the other datasets receive an
//!   update? Idea: Maybe link each set to the root set and then push changes to the children?
//! - Memory Issues?
//!   - Store all nodes/edges in a vec and map to the index as opposed to cloning the Arc? This is a
//!     memory footprint issue
//!
//! TODO:
//! - Ordered subsets
//! - get_mut/update: Create a trait for getting a node from the set with a commit() function that
//!   automatically pushes a patch into the set
//! - Research/Add benchmarking to test optimizations
//! - Lazy indices (build on demand)

use crate::{local::*, prelude::*};

use std::{
  collections::HashMap,
  ops::{Add, AddAssign},
}; // , VecDeque};

// use sync::{
//   mpsc::{channel, Sender},
//   Arc,
// };

use uuid::Uuid;

/// A group of related nodes and edges
#[derive(Clone, Debug)]
pub struct DataSet<G>
where
  G: Graph,
{
  guid: Uuid,

  /// All the vertices known to this set mapped by its guid
  nodes: NodeSet<G>,

  /// All the vertices known to this set mapped by its guid
  edges: EdgeSet<G>,

  // / Generic indices which apply to any/all of the values in the DataSet
  // indices: Indices<G>,

  // -----    Thinking/YAGNI items
  /// Statistics about the current DataSet
  ///
  /// Eventually, This should be items like memory usage, cache hits/misses, and other cumulative
  /// stats in addition to the basic counts
  _stats: (),

  /// A consolidated patch query to to bring Grapht up to date with
  ///
  /// A dynamic query containing mutations since the last sync to Grapht
  _diff: (),
}

impl<G> PartialEq for DataSet<G>
where
  G: Graph,
{
  fn eq(&self, other: &Self) -> bool {
    self.guid == other.guid
  }
}

impl<G> DataSet<G>
where
  G: Graph,
{
  pub fn new() -> DataSet<G> {
    DataSet {
      guid: Uuid::new_v4(),
      nodes: NodeSet::new(),
      edges: EdgeSet::new(),
      // indices: Indices::new(),
      _stats: (),
      _diff: (),
    }
  }

  /// Basic database query, returning results in iterable lists
  pub fn query(&self, _query: &str) -> GraphtResult<HashMap<String, Vec<Value<G>>>> {
    todo!()
  }

  /// Create a new DataSet that matches the values matching the query
  ///
  /// Items such as ordering are created in the indices. All annotations are dropped and aggregates
  /// are re-calculated
  pub fn subset(&self, _query: &str) -> GraphtResult<DataSet<G>> {
    todo!()
  }

  /// Return a list of all the nodes included in the DataSet
  pub fn nodes(&self, _query: &str) -> GraphtResult<Vec<Node<G>>> {
    let mut matches = Vec::new();
    for node in self.nodes.into_iter() {
      matches.push(node.clone())
    }
    Ok(matches)
  }

  pub fn get_guid(&self) -> Uuid {
    self.guid.clone()
  }

  /// Add a value and all its related values (properties, edges, etc) to a graph
  ///
  /// The primary purpose of this insert is to make sure the value is indexed properly within the
  /// dataset. Errors are caused when finding mutated properties.
  /// THINK:
  /// - When should each node be hashed/diffed on insert so we can determine if there has been a
  ///   change? (use/update Patchwork library)
  /// - How are updated nodes handled?
  /// - If an edge is added,
  ///
  /// FIXME:
  /// - When edges are added to the
  pub fn insert(&mut self, value: Value<G>) -> GraphtResult<CrudResultStats<DataSetStats>> {
    let mut stats = CrudResultStats::<DataSetStats>::new();

    // To protect against stack overflows during recursion, we use a while loop containing all the
    // new nodes
    let mut unprocessed = vec![value];

    while !unprocessed.is_empty() {
      let value = unprocessed.pop().unwrap();
      debug!(
        "Inserting value {} with {} remaining unprocessed",
        value.get_guid(),
        unprocessed.len()
      );

      // We disassemble the value and process one node
      match value {
        // Only need to add the one node
        Value::Node(node) => {
          // Insert the node
          match self.nodes.insert(&node) {
            // TODO: This is where we need to check if the items are actually different
            Err(err) => match err.is(Kind::DuplicateKey) {
              true => continue,
              false => return Err(err),
            },
            Ok(value_stats) => {
              if let Some(values) = value_stats.created() {
                stats.add_created(values.into());
              }
            }
          }

          // Add the edge and it's target for processing
          for edge in node.edges("") {
            unprocessed.push(edge.get_target().into());
            unprocessed.push(edge.into())
          }
        }

        Value::Edge(edge) => {
          // Skip if the edge already exists if the edge already exists.
          if self.edges.contains(&edge.get_guid()) {
            debug!(
              "Skipping repeat edge {} {}",
              edge.get_type_label(),
              edge.get_guid()
            );
            continue;
          };

          // Add the target to be processed
          let target = edge.get_target();
          unprocessed.push(target.into());

          // Add the edge to the source node if it exists, otherwise just add to unprocessed
          let source = edge.get_source();
          match self.nodes.get(&source.get_guid()) {
            Some(node) => {
              debug!("Adding the edge to the source node");
              if let Err(err) = node.add_edge(edge.clone()) {
                match err.is(Kind::DuplicateKey) {
                  true => {
                    debug!("Couldn't add the edge because it already known by the node");
                    continue;
                  }
                  false => return Err(err),
                }
              };

              // self.update_node_unchecked(node.clone())?;
            }
            None => {
              unprocessed.push(source.into());
              continue;
            }
          };
          debug!(
            "Finished adding the edge. Still have {} unprocessed",
            unprocessed.len()
          );
        }

        // Simply convert the path to edges and trailing node and let it be processed normally
        Value::Path(_) => todo!("Cannot insert paths yet"),
      };
    }
    Ok(stats)
  }

  // /// Insert a single node into the graph
  // pub fn insert_node(&mut self, node: Node<G>) -> GraphtResult<Stats> {
  //   let mut stats = Stats::new();
  //   match self.nodes.entry(node.get_guid()) {
  //     Entry::Vacant(entry) => {
  //       debug!("Vacant node");
  //       let mut node = node.deep_clone()?;
  //       node.set_bound(true);
  //       entry.insert(node);
  //       stats.nodes.created = 1;
  //     }
  //     Entry::Occupied(_) => {
  //       return Err(err!(
  //         DuplicateKey,
  //         "Node with Uuid {} already exists in the graph {}",
  //         node.get_guid(),
  //         self.guid
  //       ));
  //     }
  //   };

  //   stats += self.indices.index_value(Value::Node(node))?;
  //   Ok(stats)
  // }

  // /// Update the properties, edges, and indices of a node and return the stats of the changes
  // pub fn update_node(&mut self, _node: Node<G>) -> GraphtResult<Stats> {
  //   todo!()
  // }

  // /// Replace an existing node with the current one without internal validation
  // pub(crate) fn update_node_unchecked(&mut self, node: Node<G>) -> GraphtResult<()> {
  //   debug!("Updating the node, unchecked");
  //   // let mut stats = Stats::new();
  //   match self.nodes.entry(node.get_guid()) {
  //     Entry::Vacant(_) => {
  //       return Err(err!(
  //         NotFound,
  //         "Could not get Node with Uuid {} in the graph {} for updating",
  //         node.get_guid(),
  //         self.guid
  //       ))
  //     }
  //     Entry::Occupied(mut entry) => {
  //       debug!("Found it occupied. Cloning and inserting");
  //       // FIXME: This needs to be much more granular than a simple replacement, since the indices
  //       // won't be updated
  //       let mut node = node.deep_clone()?;
  //       node.set_bound(true);
  //       entry.insert(node);
  //     }
  //   };
  //   Ok(())
  // }

  pub fn stats(&self) -> DataSetStats {
    let mut stats = DataSetStats::default();
    stats.nodes = self.nodes.stats();
    stats.edges = self.edges.stats();

    stats
  }
}

impl<G> fmt::Display for DataSet<G>
where
  G: Graph,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("DataSet")
      .field("guid", &self.guid)
      .field("nodes", &self.nodes.stats())
      // .field("indices", &format!("{}", self.indices))
      .finish()
  }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DataSetStats {
  pub nodes: NodeStats,
  pub edges: EdgeStats,
}

impl DataSetStats {
  pub fn new() -> DataSetStats {
    DataSetStats {
      nodes: NodeStats::new(),
      edges: EdgeStats::new(),
    }
  }
}

impl Add for DataSetStats {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    DataSetStats {
      nodes: self.nodes + rhs.nodes,
      edges: self.edges + rhs.edges,
    }
  }
}

impl AddAssign for DataSetStats {
  fn add_assign(&mut self, rhs: Self) {
    *self = self.clone().add(rhs);
  }
}

impl Diff for DataSetStats {
  fn diff(&self, rhs: &Self, name: Option<&str>) -> Difference {
    let mut diffs = Difference::new();
    diffs += self.nodes.diff(&rhs.nodes, Some("nodes"));
    diffs += self.edges.diff(&rhs.edges, Some("edges"));
    diffs.opt_tag(name)
  }
}

impl From<NodeStats> for DataSetStats {
  fn from(nodes: NodeStats) -> Self {
    DataSetStats {
      nodes,
      edges: EdgeStats::new(),
    }
  }
}

impl From<EdgeStats> for DataSetStats {
  fn from(edges: EdgeStats) -> Self {
    DataSetStats {
      nodes: NodeStats::new(),
      edges,
    }
  }
}

impl Stats for DataSetStats {}

impl From<&str> for DataSetStats {
  fn from(value: &str) -> Self {
    use nom::{
      character::complete::multispace0,
      character::complete::{alphanumeric1, char, digit1},
      combinator::opt,
      error::{Error as NomError, ErrorKind as NomErrorKind, ParseError},
      multi::fold_many0,
      sequence::{delimited, preceded, terminated},
      Err as NomErr, IResult,
    };

    /// Strip out whitespace and then run the next expected function
    fn trim<'a, F: 'a, O, E: ParseError<&'a str>>(
      inner: F,
    ) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
    where
      F: FnMut(&'a str) -> IResult<&'a str, O, E>,
    {
      preceded(multispace0, inner)
    }

    fn parse_u128(input: &str) -> IResult<&str, u128> {
      let (remainder, val) = trim(terminated(digit1, trim(opt(char(',')))))(input)?;

      match val.parse() {
        Err(_err) => Err(NomErr::Error(NomError::new(remainder, NomErrorKind::Fail))),
        Ok(value) => Ok((remainder, value)),
      }
    }

    // /// Treat the block as a map
    // fn parse_map<'a, F: 'a, O, E: ParseError<&'a str>>(
    //   inner: F,
    // ) -> impl FnMut(&'a str) -> IResult<&'a str, O, E> {
    //   todo!()
    // }

    /// Contents
    fn parse_block<'a, F: 'a, O, E: ParseError<&'a str>>(
      inner: F,
    ) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
    where
      F: FnMut(&'a str) -> IResult<&'a str, O, E>,
      O: 'a,
      E: 'a,
    {
      trim(delimited(char('{'), trim(inner), trim(char('}'))))
    }

    /// Uses a key/value pair to update the value of a field belonging to a mutable object captured
    /// by the closure
    fn map_field<'a, F: 'a, O: 'a, E: 'a + ParseError<&'a str>>(
      mut mapping: F,
    ) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
    where
      F: 'a + FnMut(&'a str, &'a str) -> IResult<&'a str, O, E>,
    {
      move |input: &'a str| {
        let (remainder, key) = trim(terminated(trim(alphanumeric1), trim(opt(char(':')))))(input)?;
        info!("Mapped Field: {}", key);
        mapping(key, remainder)
      }
    }

    fn parse_nodes(input: &str) -> IResult<&str, NodeStats> {
      let mapping = move |key, mut remainder| -> IResult<&str, NodeStats> {
        let value;
        info!("Matched NodeSet.{}", key);
        let mut stats = NodeStats::new();
        match key {
          "total" => {
            (remainder, value) = parse_u128(remainder)?;
            stats.total = value;
            debug!("Had value {:?}", value);
            info!("Stats: {:?}", stats.total);
          }
          "typed" => {
            // parse_block(parse_map)
            todo!()
          }
          "labels" => {
            todo!()
            // (remainder, value) = parse_u128(remainder)?;
            // stats.labels = value;
          }
          "properties" => {
            // (remainder, value) = parse_u128(remainder)?;
            // stats.properties = value;
          }
          x => panic!("Received unknown field for building NodeStats: {:?}", x),
        }

        Ok((remainder, stats))
      };

      let (remainder, stats) = trim(fold_many0(
        map_field(mapping),
        NodeStats::new,
        |acc: NodeStats, item| acc + item,
      ))(input)?;
      info!("Post Match Stats:\n{:?}", stats);

      Ok((remainder, stats))
    }

    fn parse_edges(_input: &str) -> IResult<&str, EdgeStats> {
      todo!()
    }

    let mapping = |key, mut remainder| -> IResult<&str, DataSetStats> {
      let (nodes, edges);
      let mut data_set_stats = DataSetStats::new();
      match key {
        "nodes" => {
          // let (remain, val) = trim(preceded(char('{'), trim(alphanumeric1)))(remainder)?;
          // debug!("Matched the test {{: {:?}\n{:?}", val, remain);

          (remainder, nodes) = parse_block(parse_nodes)(remainder)?;
          data_set_stats.nodes = nodes;
        }
        "edges" => {
          (remainder, edges) = parse_block(parse_edges)(remainder)?;
          data_set_stats.edges = edges;
        }
        x => panic!("Received unknown field for building DataSetStats: {:?}", x),
      }

      Ok((remainder, data_set_stats))
    };

    let (_remainder, data_set_stats) =
      fold_many0(map_field(mapping), DataSetStats::new, |stats, item| {
        stats + item
      })(value)
      .expect(&format!("Failed to parse the DataSet from\n{:?}", value));

    data_set_stats
  }
}

/*
#[derive(Clone, Debug)]
pub struct Activity {
  timestamp: (),
  item: ActivityItem,
  synced: bool,
}

#[derive(Clone, Debug)]
pub enum ActivityItem {
  // Makes sure the change is not included in the sync
  Local(Box<ActivityItem>),

  /// Select specific data from the Grapht instance
  Query,
  Insert,
  Update,
  Mutate,
  /// Excludes an item from the set, but doesn't persist any changes to
  Remove,
  Delete,
}
 */
