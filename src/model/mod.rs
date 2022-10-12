//! A local data store for all known remote data as well as locally generated ephemeral data
//!
//! The store acts as a queryable data warehouse, keeping as much locally as seems reasonable. The
//! goal for this is to cut down network traffic in cases where having external dedicated services
//! updating the data is overkill for the current action. For example, sorting the data in a table
//! makes more sense to do onsite rather than re-download everything again in a new order.

use crate::local::*;

pub mod node;
pub use node::Node;

pub mod edge;
pub use edge::{Edge, EdgeMap};

pub mod path;
pub use path::Path;

pub mod entity;
pub use entity::GraphtEntity;

pub mod graph;
pub use graph::{Graph, GraphItem};
