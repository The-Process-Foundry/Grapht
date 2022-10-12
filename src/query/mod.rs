//! The query object for creating/mutating data sets
//!
//! This will be replaced with GQuery when I'm ready to focus on it
//!
//! FIXME: This is not my current focus, so I'm just putting a placeholder here for the moment so I
//! have something to pass around

pub struct GQuery {}

impl GQuery {
  pub fn new() -> GQuery {
    GraphtQuery {}
  }
}

pub enum Statement {
  Insert,
}
