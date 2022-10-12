//! The organization node type

// use crate::{local::*, prelude::*};
use grapht::{err, prelude::*};

use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::value::Value as JsonValue;
use uuid::Uuid;

use super::FhlEdge;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct OrganizationParent {}

impl OrganizationParent {
  pub fn new() -> OrganizationParent {
    OrganizationParent {}
  }
}

impl GraphtEntity for OrganizationParent {
  fn get_type_label(&self) -> String {
    return "__OrganizationParent".to_string();
  }

  fn get_key(&self) -> Uuid {
    todo!()
  }

  fn get_inner<T: Clone + core::fmt::Debug>(&self) -> std::borrow::Cow<T> {
    todo!()
  }

  fn to_gql(&self) -> GraphtResult<String> {
    todo!()
  }

  fn from_gql(value: &[u8]) -> GraphtResult<Self> {
    todo!()
  }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct OrganizationChild {}

impl OrganizationChild {
  pub fn new() -> OrganizationChild {
    OrganizationChild {}
  }
}

impl GraphtEntity for OrganizationChild {
  fn get_type_label(&self) -> String {
    return "__OrganizationChild".to_string();
  }

  fn get_key(&self) -> Uuid {
    todo!()
  }

  fn get_inner<T: Clone + core::fmt::Debug>(&self) -> std::borrow::Cow<T> {
    todo!()
  }

  fn to_gql(&self) -> GraphtResult<String> {
    todo!()
  }

  fn from_gql(value: &[u8]) -> GraphtResult<Self> {
    todo!()
  }
}

/// A temporary test node for defining rows. This will be Grapht eventually
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Organization {
  pub guid: Uuid,
  pub pretty_id: String,
  pub org_name: String,
  pub balance: Decimal,
  // Edge Type:
  // pub parent: Option<Arc<Mutex<Organization>>>,
  // pub children: Vec<Arc<Mutex<Organization>>>,
}

impl Organization {
  pub fn new(pretty_id: &str, org_name: &str, balance: Decimal) -> Organization {
    Organization {
      guid: Uuid::new_v4(),
      pretty_id: pretty_id.to_string(),
      org_name: org_name.to_string(),
      balance,
      // parent: None,
      // children: Vec::new(),
    }
  }

  //---  This will be the GraphtNode Trait, but not going to worry about that yet
}

// impl From<Organization> for Node<Organization, FhlEdge> {
//   fn from(org: Organization) -> Self {
//     Node::new(org)
//   }
// }

impl GraphtEntity for Organization {
  fn get_type_label(&self) -> String {
    "__Organization".to_string()
  }

  fn get_key(&self) -> Uuid {
    self.guid.clone()
  }

  fn get_inner<T: Clone + core::fmt::Debug>(&self) -> std::borrow::Cow<T> {
    todo!()
  }

  fn to_gql(&self) -> GraphtResult<String> {
    let json = serde_json::to_value(self).map_err(|err| {
      err!(
        SerializationError,
        "Error serializing Organization.\nOrg:\n{:?}\nErr:\n{:?}",
        self,
        err
      )
    })?;

    if let JsonValue::Object(mapping) = json {
      let mut values = "{".to_string();
      let mut first = true;
      for (key, value) in mapping {
        if &key == "guid" {
          continue;
        }

        if first {
          first = false
        } else {
          values.push_str(", ");
        }

        values.push_str(&format!("{}: {}", key, value)[..]);
      }
      values.push('}');
      Ok(values)
    } else {
      unreachable!(
        "Serializing an organization should always return an object. Instead, it returned:\n{:?} ",
        json
      )
    }
  }

  fn from_gql(value: &[u8]) -> GraphtResult<Self> {
    todo!()
  }
}
