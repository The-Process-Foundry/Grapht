//! Pre-compiled lookups for getting node, edge, and paths

use crate::{local::*, prelude::*};

use std::{
  collections::{hash_map::Entry, HashMap, HashSet},
  hash::Hash,
};
use uuid::Uuid;

/// The values that belong to the same index
#[derive(Debug, Clone)]
pub struct Lookup<G>
where
  G: Graph,
{
  /// The common definition that links all the saved values
  index: Index,

  /// The known set of values that match the given index
  values: HashSet<Value<G>>,
}

impl<G> Lookup<G>
where
  G: Graph,
{
  pub fn new(index: Index) -> Lookup<G> {
    Lookup {
      index,
      values: HashSet::new(),
    }
  }

  pub fn len(&self) -> usize {
    self.values.len()
  }

  pub fn all(&self) -> Vec<Value<G>> {
    self.values.iter().map(|x| x.clone()).collect()
  }

  pub fn find_all(&self, _filter: &str) -> Vec<Value<G>> {
    todo!("Lookup::find_all")
  }

  pub fn insert(&mut self, value: Value<G>) -> GraphtResult<()> {
    if self.values.contains(&value) {
      Err(err!(
        DuplicateKey,
        "Value with Uuid {} already exists in the lookup {:?}",
        value.get_guid(),
        self.index
      ))
    } else {
      self.values.insert(value.clone());
      Ok(())
    }
  }
}

/// Definitions for how to build the given index
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Index {
  /// An ephemeral label, temporarily grouping together of nodes and edges in memory
  Tag(String),

  /// An ordered list of nodes based on a comparison criteria.
  Sorted(String),
  // A cache that can be used for optionally storing subsets that match a piece of a query
  // QueryFragment(QueryFragment),

  // A binary search tree of characters, where the search string is
  // THINK:
  // Is this a plugin/generic/crate or specifically coded in Grapht? This seems like finding
  // a path to match is valid. Also, the index definition is not static, being updated with each
  // node added
  // TextSearch(),
}

impl Index {
  /// Checks if the value
  pub fn belongs<G: Graph>(&self, _value: Value<G>) -> bool {
    todo!("Index::belongs is not implemented yet")
  }
}

/// A full set of lookup tables used by a DataSet for improving query speed
///
/// THINK: How does this
#[derive(Debug, Clone)]
pub struct Indices<G>
where
  G: Graph,
{
  lookups: HashMap<Index, Lookup<G>>,
}

impl<G> Indices<G>
where
  G: Graph,
{
  pub fn new() -> Indices<G> {
    Indices {
      lookups: HashMap::new(),
    }
  }

  pub fn contains(&self, index: &Index, _guid: &Uuid) -> bool {
    match self.lookups.get(index) {
      Some(_lookup) => todo!(),
      None => false,
    }
  }

  /// Add a new value to each index to which it belongs.
  ///
  /// Tags are skipped, as they are created separately and have no internal rules defining the
  /// values that can be included.
  pub fn index_value(&mut self, value: Value<G>) -> GraphtResult<Stats> {
    let mut stats = Stats::default();

    let labels: Vec<String> = {
      match &value {
        Value::Node(node) => node.get_labels().iter().map(|x| x.clone()).collect(),
        Value::Edge(edge) => vec![edge.get_label()],
        // This option doesn't make sense when indexing a value
        Value::Path(_) => Vec::new(),
      }
    };

    for label in labels {
      stats += self.add_value(&Index::Label(label), value.clone())?;
    }

    Ok(stats)
  }

  /// Add a value to a lookup index
  ///
  /// This double checks that the value being inserted matches the criteria for the index.
  /// TODO: Add the validation
  pub fn add_value(&mut self, key: &Index, value: Value<G>) -> GraphtResult<Stats> {
    // key.belongs(&value)?;
    self.add_value_unchecked(key, value)
  }

  /// Add a value to an index without validation
  ///
  /// This is
  pub fn add_value_unchecked(&mut self, key: &Index, value: Value<G>) -> GraphtResult<Stats> {
    let mut stats = Stats::new();

    // Get the lookup or create it if this is the first
    if !self.lookups.contains_key(key) {
      let _ = self.lookups.insert(key.clone(), Lookup::new(key.clone()));
      match key {
        Index::Label(value) | Index::Tag(value) => {
          let mut mutation = Mutations::new();
          mutation.created = 1;
          stats.indices.labels.total += mutation.clone();
          match stats.indices.labels.labels.entry(value.clone()) {
            Entry::Vacant(entry) => {
              entry.insert(mutation);
            }
            Entry::Occupied(mut entry) => *entry.get_mut() += mutation,
          }
          stats.indices.labels.total.created += 1;
        }
        _ => todo!("No stats ready except for Label"),
      }
    };

    let _ = self.lookups.get_mut(key).unwrap().insert(value.clone());

    Ok(stats)
  }

  /// Remove all references of the given value from each index
  pub fn remove_value(&mut self, _value: Value<G>) -> GraphtResult<()> {
    todo!("Indices::remove_value")
  }

  pub fn stats(&self) -> IndexStats {
    let mut stats = IndexStats::new();
    // Iterate through each lookup and gather the stats from it
    for (index, lookup) in self.lookups {
      match index {
        Index::Edge(name) => match stats.edges.entry(name) {
          Entry::Vacant(entry) => {
            let mut edges = Mutations::new();
            edges.created = lookup.len() as u128;
            let _ = entry.insert(edges);
          }
          Entry::Occupied(entry) => {
            let mut edges = entry.get_mut();
            *edges.created = lookup.len() as u128;
          }
        },
        _ => todo!("Only Index is available for stats right now"),
      }
    }
    stats
  }
}

impl<G> Display for Indices<G>
where
  G: Graph,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut output: &mut fmt::DebugStruct = &mut f.debug_struct("Indices");

    // Group the indices by type so they can be formatted uniformly
    let mut indices = HashMap::<&str, Vec<&Lookup<G>>>::new();
    for value in self.lookups.values() {
      let label = match value.index {
        Index::Label(_) => "Label",
        Index::Edge(_) => "Edge",
        Index::Tag(_) => "Tag",
        Index::Sorted(_) => "Sorted",
      };

      if !indices.contains_key(label) {
        indices.insert(label, Vec::new());
      }

      if let Some(x) = indices.get_mut(label) {
        x.push(value)
      }
    }

    // Labels only print a count of the nodes having that tag
    if let Some(lookups) = indices.get_mut("Label") {
      lookups.sort_by(|a, b| a.index.cmp(&b.index));
      let values: Vec<String> = lookups
        .iter()
        .map(|lookup| {
          if let Index::Label(name) = &lookup.index {
            format!("{}: {} Members", name, lookup.values.len())
          } else {
            unreachable!("It's always a label")
          }
        })
        .collect();
      output = output.field("Labels", &values);
    }

    // Edge only print a count of the edges of that type
    if let Some(lookups) = indices.get_mut("Edge") {
      lookups.sort_by(|a, b| a.index.cmp(&b.index));
      let values: Vec<String> = lookups
        .iter()
        .map(|lookup| {
          if let Index::Label(name) = &lookup.index {
            format!("{}: {} Members", name, lookup.values.len())
          } else {
            unreachable!("It's always a label")
          }
        })
        .collect();
      output = output.field("Edges", &values);
    }

    // Tags print a count of the nodes with the tag
    if let Some(lookups) = indices.get_mut("Tag") {
      lookups.sort_by(|a, b| a.index.cmp(&b.index));
      let values: Vec<String> = lookups
        .iter()
        .map(|lookup| {
          if let Index::Label(name) = &lookup.index {
            format!("{}: {} Members", name, lookup.values.len())
          } else {
            unreachable!("It's always a tag")
          }
        })
        .collect();
      output = output.field("Labels", &values);
    }

    output.finish()
  }
}
