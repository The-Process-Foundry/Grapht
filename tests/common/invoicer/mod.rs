//! Models used for my FHL demo. These are temporary and will be moved back to the FHL project

mod organization;
pub use organization::{Organization, OrganizationChild, OrganizationParent};

mod graph;
pub use graph::{FhlEdge, FhlEdgeType, FhlGraph, FhlNode};

pub mod fhl_prelude {
  pub use super::*;

  pub use grapht::prelude::*;

  pub use tracing::{info, warn};

  // This should be the crate::local, but for testing we'll drop it here
  pub use std::sync;
}
