//! A local data repository, responsible for indexing and querying knows values
//!
//! The store acts as a queryable data warehouse, keeping as much locally as seems reasonable. The
//! goal for this is to cut down network traffic in cases where having external dedicated services
//! updating the data is overkill for the current action. For example, sorting the data in a table
//! makes more sense to do onsite rather than re-download everything again in a new order.

/// Organization tools for manipulating a store
pub mod dataset;
pub use dataset::*;

pub mod nodeset;
pub use nodeset::*;

pub mod edgeset;
pub use edgeset::*;

// pub mod index;
// pub use index::*;

pub mod value;
pub use value::*;
