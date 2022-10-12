//! An entity is the payload of individual graph item, node or edge.
//!
//!

use crate::{local::*, prelude::*};

use sync::{Arc, RwLock};

use std::borrow::Cow;
use uuid::Uuid;

/// A trait allowing a struct to be used as a data payload for nodes and/or edges
pub trait GraphtEntity: Clone + Debug + std::hash::Hash + std::cmp::Eq + Sized {
  /// Return a string for the type of entity. This is used as a label on nodes
  fn get_type_label(&self) -> String;

  /// Get a unique identifier (generally calculated so it can be preserved)
  fn get_key(&self) -> Uuid;

  /// A pointer to the inner value of the entity.
  ///
  /// Wrapping it in a Cow, since this is usually used for read only purposes
  fn get_inner<T: Clone + fmt::Debug>(&self) -> Cow<T>;

  // TODO: Query the entity/property values using BSON
  // fn find(&self, query: bson) -> GQResult<Vec<Edge<G>>>

  /// Serialize into a GQL string (similar to JSON with a few tweaks)
  fn to_gql(&self) -> GraphtResult<String>;

  /// Deserialize the entity from a u8 array as returned by the database
  fn from_gql(value: &[u8]) -> GraphtResult<Self>;
}

// ---  Primitive entities

// Unit as an entity
impl GraphtEntity for () {
  fn get_type_label(&self) -> String {
    "__unit".to_string()
  }

  /// The key for unit is always nil
  fn get_key(&self) -> Uuid {
    Uuid::nil()
  }

  fn get_inner<T: Clone + fmt::Debug>(&self) -> Cow<T> {
    todo!("No way to get the inner value ")
  }

  fn to_gql(&self) -> GraphtResult<String> {
    Ok(String::new())
  }

  fn from_gql(value: &[u8]) -> GraphtResult<Self> {
    todo!()
  }
}
