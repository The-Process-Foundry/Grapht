//! A simple diff maker for visualizing structs that are not the same
//!
//! TODO:
//! - Move this to Patchwork/Protean, if not too simplified

use crate::local::*;

use std::{
  collections::{HashMap, HashSet},
  hash::Hash,
  ops::{Add, AddAssign},
};

/// Simple diffs that always return a value rather than nested children
macro_rules! primitive_diffs {
  ($ty:ty) => {
    /// Create a diff for primitive $ty
    impl Diff for $ty {
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
  };

  ($($ty:ty), +) => {
    $(
      primitive_diffs!($ty);
    )+
  }
}

primitive_diffs!(u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, f32, f64, &str, String);

// /// A temporary diff for u128. This would be the default, execpt I don't want to require PartialEq
// /// or Display
// impl Diff for u128 {
//   fn diff(&self, rhs: &Self, name: Option<&str>) -> Difference {
//     if *self == *rhs {
//       return Difference::Empty;
//     }

//     let value = Some((format!("{}", self), format!("{}", rhs)));
//     match name {
//       None => Difference::Node(value, HashMap::new()),
//       Some(key) => Difference::Node(
//         None,
//         HashMap::from([(
//           key.to_string(),
//           Box::new(Difference::Node(value, HashMap::new())),
//         )]),
//       ),
//     }
//   }
// }

impl<T> Diff for Option<T>
where
  T: Diff + Default,
{
  fn diff(&self, rhs: &Self, name: Option<&str>) -> Difference {
    match (self, rhs) {
      (None, None) => Difference::Empty,
      (Some(lhs), None) => lhs.diff(&T::default(), None),
      (None, Some(rhs)) => T::default().diff(rhs, None),
      (Some(lhs), Some(rhs)) => lhs.diff(rhs, None),
    }
    .opt_tag(name)
  }
}

impl<T, U> Diff for HashMap<T, U>
where
  T: Eq + Hash + Clone + Display,
  U: fmt::Debug + Diff,
{
  fn diff(&self, rhs: &Self, name: Option<&str>) -> Difference {
    let mut differences = Difference::Empty;

    // Drop the keys we have seen in the left hand side
    let mut right_only: HashSet<&T> = HashSet::new();
    for key in rhs.keys() {
      let _ = right_only.insert(key);
    }

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

    differences.opt_tag(name)
  }
}

impl<T> Diff for Vec<T>
where
  T: Diff + fmt::Display,
{
  // Currently, this simply checks that each index matches, but doesn't properly describe the diff.
  // FIXME: This should be more clever in finding differences so removing the first item from the
  //   left hand side doesn't force it to return both vectors in full
  //   the middle of the left hand side. Think in terms of tracking DNA edit errors such as
  //   transpose, copy, reverse
  // THINK: If this is in patchwork/protean, should each diff enumerate the type of change: eg
  //   mutation, delete, and add?
  fn diff(&self, rhs: &Self, name: Option<&str>) -> Difference {
    let mut differences = Difference::Empty;

    for (i, l_val) in self.iter().enumerate() {
      if let Some(r_val) = rhs.get(i) {
        differences.merge(l_val.diff(r_val, Some(&i.to_string())), None)
      } else {
        differences.merge(
          Difference::Node(Some((format!("{}", l_val), String::new())), HashMap::new()),
          Some(vec![i.to_string()]),
        );
      }
    }

    // Add on any remaining items from the right hand side
    for i in self.len()..rhs.len() {
      if let Some(r_val) = rhs.get(i) {
        differences.merge(
          Difference::Node(Some((String::new(), format!("{}", r_val))), HashMap::new()),
          Some(vec![i.to_string()]),
        );
      }
    }

    differences.opt_tag(name)
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
  ///
  /// Node((Lefthand Value, Righthand Value), HashMap<key, nodes)
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
  pub fn is_empty(&self) -> bool {
    let pruned = self.prune();
    match pruned {
      Difference::Empty => true,
      _ => false,
    }
  }

  /// Make sure there was no difference and panics if there was
  pub fn assert_empty(&self) {
    let pruned = self.prune();
    match pruned.is_empty() {
      true => return,
      false => panic!("The values had a difference of:\n{:#?}", pruned),
    }
  }

  /// A helper to move the root node into a named child, if desired
  ///
  /// Since this is mostly used in the diff implementation, we make the name optional so it's
  /// easier to pass along the parameter than check beforehand
  pub fn opt_tag(self, name: Option<&str>) -> Difference {
    match name {
      Some(name) => self.tag(name),
      None => self,
    }
  }

  /// Creates a new root node with the current node mapped as a named child
  ///
  /// Since this is mostly used in the diff definition, we make the name optional so it's easier to
  /// just pass along the parameter than check beforehand
  pub fn tag(self, name: &str) -> Difference {
    Difference::Node(None, HashMap::from([(name.to_string(), Box::new(self))]))
  }

  /// Traverse the difference tree and remove all empty nodes
  pub fn prune(&self) -> Difference {
    match self {
      Difference::Empty => return self.clone(),
      Difference::Node(value, mapping) => {
        let mut non_empty = HashMap::new();
        // Clear out the map
        for (key, value) in mapping.iter() {
          let pruned = value.prune();
          if !pruned.is_empty() {
            let _ = non_empty.insert(key.clone(), Box::new(pruned));
          }
        }

        match (value.is_none(), non_empty.is_empty()) {
          (true, true) => Difference::Empty,
          (_, _) => Difference::Node(value.clone(), non_empty),
        }
      }
    }
  }
}

/// Pretty print for the diff.
///
/// TODO: Fix the indenting. Base it on https://doc.rust-lang.org/src/core/fmt/builders.rs.html#89
impl Display for Difference {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Difference::Empty => f.write_str("Empty"),
      Difference::Node(value, mapping) => {
        let mut children = Vec::new();
        for (key, value) in mapping.iter() {
          if let Difference::Empty = **value {
            continue;
          }

          children.push(format!("  {}: {}", key, **value));
        }

        let mut indent = "";
        if children.len() > 0 {
          f.write_str("{\n")?;
          indent = "  ";
        }

        if let Some(value) = value {
          f.write_str(&format!("{}{} != {}\n", indent, value.0, value.1))?;
        }

        for child in &children {
          f.write_str(child)?;
        }

        if children.len() > 0 {
          f.write_str("}")?;
        }

        Ok(())
      }
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

    // Replace the string value with right hand side, if it exists
    if r_value.is_some() {
      value = r_value.clone();
    }

    for (key, value) in r_mapping {
      mapping
        .entry(key.clone())
        .and_modify(|inner| **inner += *value.clone())
        .or_insert_with(|| value.clone());
    }

    Node(value.clone(), mapping.clone())
  }
}

impl AddAssign for Difference {
  fn add_assign(&mut self, rhs: Self) {
    *self = self.clone().add(rhs);
  }
}
