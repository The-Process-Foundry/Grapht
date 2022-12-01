//! Context sensitive statistics about a data set
//!
//! This is used both as the result of a query and of a static data set

use crate::{errors::GraphtError, local::*, utils::*};

use std::{
  collections::{hash_map::Entry, HashMap},
  ops::{Add, AddAssign},
};

use serde::{Deserialize, Serialize};

/// A value used for tracking activity data
pub trait Stats: Debug + Clone + PartialEq + Eq + Diff + Add + AddAssign + Default {
  /// A value that can be used to mutate the statistic
  ///
  /// THINK: Does this need to be explicit or can it be inferred?
  type Item;

  /// Increase the statistic using the value passed in
  ///
  /// For example, the simple counter takes an integer and will change by the value passed in
  fn increase(&mut self, _value: Self::Item) {
    unimplemented!("'Stats::increase()' still needs to be implemented")
  }

  /// Reset the statistic to an initial value
  fn clear(&mut self) {
    unimplemented!("'Stats::clear()' still needs to be implemented")
  }
}

/// A basic counter
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatCount {
  count: u128,
}

impl StatCount {
  pub fn new() -> StatCount {
    StatCount { count: 0 }
  }

  /// Increment the counter by 1
  pub fn incr(&mut self) {
    self.count += 1;
  }

  /// Decrement the counter by 1
  pub fn decr(&mut self) {
    match self.count > 0 {
      true => self.count -= 1,
      false => (),
    }
  }
}

impl Stats for i128 {
  type Item = i128;
}

impl Stats for StatCount {
  type Item = i128;

  fn clear(&mut self) {
    self.count = 0
  }

  /// Add/subtract a specific amount
  fn increase(&mut self, value: Self::Item) {
    match value > 0 {
      true => self.count += value as u128,
      false => {
        let neg = -value as u128;
        match neg > self.count {
          true => self.count = 0,
          false => self.count -= neg,
        }
      }
    }
  }
}

impl Add for StatCount {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    StatCount {
      count: self.count + rhs.count,
    }
  }
}

impl AddAssign for StatCount {
  fn add_assign(&mut self, rhs: Self) {
    *self = self.clone().add(rhs);
  }
}

impl Diff for StatCount {
  fn diff(&self, rhs: &Self, name: Option<&str>) -> Difference {
    self.count.diff(&rhs.count, None).opt_tag(name)
  }
}

/// A rolling average
pub struct StatAverage<T> {
  _count: u128,
  _total: f64,
  _values: Vec<T>,
}

impl<T> StatAverage<T> {
  pub fn new() -> StatAverage<T> {
    StatAverage {
      _count: 0,
      _total: 0.0,
      _values: Vec::new(),
    }
  }
}

/// Grouping stats by a derived value
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatMap<T, U, I>
where
  T: Debug + Clone + hash::Hash + Eq,
  U: Stats<Item = I> + Clone,
  I: Clone,
{
  data: HashMap<T, U>,
}

impl<T, U, I> StatMap<T, U, I>
where
  T: Debug + Display + Clone + hash::Hash + Eq,
  U: Stats<Item = I> + Clone,
  I: Clone,
{
  pub fn new() -> StatMap<T, U, I> {
    StatMap {
      data: HashMap::new(),
    }
  }
}

impl<T, U, I> Stats for StatMap<T, U, I>
where
  T: Debug + Clone + hash::Hash + Eq + Display,
  U: Stats<Item = I> + Add<Output = U> + Clone,
  I: Clone + Eq + Debug,
{
  type Item = (T, I);

  fn increase(&mut self, value: Self::Item) {
    let (key, item) = value;

    match self.data.entry(key) {
      Entry::Occupied(mut entry) => entry.get_mut().increase(item),
      Entry::Vacant(entry) => {
        let mut value: U = Default::default();
        value.increase(item);
        let _ = entry.insert(value);
      }
    }
  }

  fn clear(&mut self) {
    self.data = HashMap::new()
  }
}

impl<T, U, I> Default for StatMap<T, U, I>
where
  T: Debug + Display + Clone + hash::Hash + Eq,
  U: Stats<Item = I> + Default,
  I: Clone,
{
  fn default() -> Self {
    Self {
      data: Default::default(),
    }
  }
}

impl<T, U, I> Add for StatMap<T, U, I>
where
  T: Debug + Display + Clone + hash::Hash + Eq,
  U: Stats<Item = I>,
  I: Clone,
{
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    let mut data = self.data;
    for (key, value) in &rhs.data {
      data
        .entry(key.clone())
        .and_modify(|val| *val += value.clone())
        .or_insert(value.clone());
    }

    StatMap { data }
  }
}

impl<T, U, I> AddAssign for StatMap<T, U, I>
where
  T: Debug + Display + Clone + hash::Hash + Eq,
  U: Stats<Item = I> + Add<Output = U>,
  I: Clone,
{
  fn add_assign(&mut self, rhs: Self) {
    *self = self.clone() + rhs;
  }
}

impl<T, U, I> Diff for StatMap<T, U, I>
where
  T: Debug + Display + Clone + hash::Hash + Eq,
  U: Stats<Item = I> + Default,
  I: Clone,
{
  fn diff(&self, rhs: &Self, name: Option<&str>) -> Difference {
    self.data.diff(&rhs.data, name)
  }
}
/// CRUD operation statistics created by wrapping existing statistics by mutation type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrudResultStats<T>
where
  T: Stats + Add<Output = T> + Default,
{
  #[serde(default)]
  created: Option<T>,
  #[serde(default)]
  read: Option<T>,
  #[serde(default)]
  updated: Option<T>,
  #[serde(default)]
  deleted: Option<T>,
  #[serde(default)]
  errors: Vec<GraphtError>,
}

impl<T> CrudResultStats<T>
where
  T: Stats + Add<Output = T> + Default,
{
  pub fn new() -> CrudResultStats<T> {
    CrudResultStats {
      read: None,
      created: None,
      updated: None,
      deleted: None,
      errors: Vec::new(),
    }
  }

  pub fn created(&self) -> Option<T> {
    self.created.clone()
  }

  /// Get the diff of a single subtype of the result stats
  ///
  /// Mostly used in testing, as it is easier to manipulate the internal T to match stats than to
  /// change the individual Options for crud.
  pub fn diff_crud(&self, crud_type: CrudType, rhs: &T) -> Difference {
    match crud_type {
      CrudType::Create => self.created.diff(&Some(rhs.clone()), None),
      CrudType::Read => self.read.diff(&Some(rhs.clone()), None),
      CrudType::Update => self.updated.diff(&Some(rhs.clone()), None),
      CrudType::Delete => self.deleted.diff(&Some(rhs.clone()), None),
      CrudType::Error => panic!("Cannot use cmp_crud to match errors. Use diff_errors instead"),
    }
  }

  /// Check expected errors
  pub fn diff_errors(&self, rhs: &Vec<GraphtError>) -> Difference {
    self.errors.diff(rhs, None)
  }

  pub fn add_crud(&mut self, crud_type: CrudType, rhs: T) {
    match crud_type {
      CrudType::Create => self.add_created(rhs),
      CrudType::Read => self.add_read(rhs),
      CrudType::Update => self.add_updated(rhs),
      CrudType::Delete => self.add_deleted(rhs),
      CrudType::Error => panic!("Cannot use cmp_crud to match errors. Use add_errors instead"),
    }
  }

  pub fn add_read(&mut self, stats: T) {
    if let Some(s) = &mut self.created {
      *s += stats;
    } else {
      self.read = Some(stats);
    }
  }

  pub fn add_created(&mut self, stats: T) {
    if let Some(s) = &mut self.created {
      *s += stats;
    } else {
      self.created = Some(stats);
    }
  }

  pub fn updated(&self) -> Option<T> {
    self.updated.clone()
  }

  pub fn add_updated(&mut self, stats: T) {
    if let Some(s) = &mut self.updated {
      *s += stats;
    } else {
      self.updated = Some(stats);
    }
  }

  pub fn deleted(&self) -> Option<T> {
    self.deleted.clone()
  }

  pub fn add_deleted(&mut self, stats: T) {
    if let Some(s) = &mut self.deleted {
      *s += stats;
    } else {
      self.deleted = Some(stats);
    }
  }

  /// Diff two sets of stats and throw an error if they are not the same
  ///
  /// This is primarily for testing, but handy to have integrated into the live code.
  pub fn assert_eq(&self, rhs: &Self) {
    let diff = self.diff(&rhs, None).prune();
    if let Difference::Node(_, _) = diff {
      error!(
        "The two sets of stats had the following differences:\n{}",
        diff
      );
      panic!()
    }
  }
}

impl<T> Add for CrudResultStats<T>
where
  T: Stats + Add<Output = T> + Default,
{
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    let created = match (self.created, rhs.created) {
      (None, None) => None,
      (Some(x), None) => Some(x),
      (None, Some(y)) => Some(y),
      (Some(x), Some(y)) => Some(x + y),
    };

    let read = match (self.read, rhs.read) {
      (None, None) => None,
      (Some(x), None) => Some(x),
      (None, Some(y)) => Some(y),
      (Some(x), Some(y)) => Some(x + y),
    };

    let updated = match (self.updated, rhs.updated) {
      (None, None) => None,
      (Some(x), None) => Some(x),
      (None, Some(y)) => Some(y),
      (Some(x), Some(y)) => Some(x + y),
    };

    let deleted = match (self.deleted, rhs.deleted) {
      (None, None) => None,
      (Some(x), None) => Some(x),
      (None, Some(y)) => Some(y),
      (Some(x), Some(y)) => Some(x + y),
    };

    let mut errors = self.errors.clone();
    errors.extend(rhs.errors);

    Self {
      created,
      read,
      updated,
      deleted,
      errors,
    }
  }
}

impl<T> AddAssign for CrudResultStats<T>
where
  T: Stats + Add<Output = T> + Default,
{
  fn add_assign(&mut self, rhs: Self) {
    *self = self.clone().add(rhs)
  }
}

impl<T> Diff for CrudResultStats<T>
where
  T: Stats + Add<Output = T> + Diff + Default,
{
  fn diff(&self, rhs: &Self, _name: Option<&str>) -> Difference {
    let mut diff = Difference::new();
    diff += self.created.diff(&rhs.created, Some("created"));
    diff += self.updated.diff(&rhs.updated, Some("updated"));
    diff += self.deleted.diff(&rhs.deleted, Some("deleted"));
    diff += self.errors.diff(&rhs.errors, Some("failed"));

    diff
  }
}

pub enum CrudType {
  Create,
  Read,
  Update,
  Delete,
  Error,
}
