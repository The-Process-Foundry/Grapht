//! A simple diff maker for visualizing structs that are not the same
//!
//! TODO:
//! - Move this to Patchwork/Protean, if not too simplified
//! - Make a macro to build each primitive

use crate::local::*;

use std::{
  collections::{HashMap, HashSet},
  hash::Hash,
  ops::{Add, AddAssign},
};

/// A temporary diff for u128. This would be the default, execpt I don't want to require PartialEq
/// or Display
impl Diff for u128 {
  fn diff(&self, rhs: &Self, name: Option<&str>) -> Difference {
    if *self == *rhs {
      return Difference::Empty;
    }

    let value = Some((format!("{}", self), format!("{}", rhs)));
    match name {
      None => Difference::Node(value, HashMap::new()),
      Some(key) => Difference::Node(
        None,
        HashMap::from([(
          key.to_string(),
          Box::new(Difference::Node(value, HashMap::new())),
        )]),
      ),
    }
  }
}

impl<T, U> Diff for HashMap<T, U>
where
  T: Eq + Hash + Clone + Display,
  U: fmt::Debug + Diff,
{
  fn diff(&self, rhs: &Self, name: Option<&str>) -> Difference {
    // Drop the keys we have seen in the left hand side
    let mut right_only: HashSet<&T> = HashSet::new();
    for key in rhs.keys() {
      let _ = right_only.insert(key);
    }

    let mut differences = Difference::Empty;

    for (key, value) in self.iter() {
      differences += match rhs.get(key) {
        Some(r_value) => value.diff(r_value, Some(&key.to_string())),
        None => Difference::Node(
          None,
          HashMap::from([(
            key.to_string(),
            Box::new(Difference::Node(
              Some((format!("{:?}", value), "Null".to_string())),
              HashMap::new(),
            )),
          )]),
        ),
      }
    }
    match name {
      Some(name) => {
        let mut result = Difference::Empty;
        result.merge(differences, Some(vec![name.to_string()]));
        result
      }
      None => differences,
    }
  }
}

pub trait Diff: Sized {
  fn diff(&self, rhs: &Self, name: Option<&str>) -> Difference;
}

/// The result of a comparison
#[derive(Clone, Debug)]
pub enum Difference {
  /// There was no differences found as of yet
  Empty,

  /// Nested values, for use with structs
  Node(Option<(String, String)>, HashMap<String, Box<Difference>>),
}

impl Default for Difference {
  fn default() -> Difference {
    Difference::Empty
  }
}

impl Difference {
  pub fn new() -> Difference {
    Difference::default()
  }

  pub fn merge(&mut self, new: Difference, path: Option<Vec<String>>) {
    use Difference::*;

    // Identity check - inserting an empty always returns the same value
    if let Empty = new {
      return;
    };

    // find the proper location to insert the difference
    if let Some(mut path) = path {
      match path.pop() {
        // Ends the recursion and moves on to process the value
        None => (),
        // Otherwise we continue to traverse the tree
        Some(key) => {
          if let Empty = self {
            *self = Node(None, HashMap::new())
          }

          if let Node(_, mapping) = self {
            mapping
              .entry(key)
              .or_insert_with(|| Box::new(Node(None, HashMap::new())))
              .merge(new, Some(path));
          }
          return;
        }
      }
    }

    *self = self.clone() + new;
  }

  /// Runs an assert!, doing nothing if the diff is empty and panics with a pretty print if true
  pub fn assert(&self) {
    match self {
      Difference::Empty => return,
      _ => panic!(
        "The left and right hand sides had the following differences:\n{:?}",
        self
      ),
    }
  }
}

impl Add for Difference {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    use Difference::*;

    let (r_value, r_mapping) = match &rhs {
      Empty => return self,
      Node(val, map) => (val, map),
    };

    let (mut value, mut mapping) = match self {
      Empty => return rhs,
      Node(val, map) => (val, map),
    };

    if r_value.is_some() {
      value = r_value.clone();
    }

    for (key, value) in r_mapping {
      mapping
        .entry(key.clone())
        .and_modify(|inner| **inner += *value.clone());
    }

    Node(value.clone(), mapping.clone())
  }
}

impl AddAssign for Difference {
  fn add_assign(&mut self, rhs: Self) {
    *self = self.clone().add(rhs);
  }
}
