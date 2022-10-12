//! Some common testing tools

// Initialize a test
// use lazy_static::lazy_static;
use std::sync::Once;

static LOGGING: Once = Once::new();

pub mod invoicer;

// use grapht::prelude::*;
//
// lazy_static! {
//   /// A Grapht instance that exists between tests
//   pub static ref DB: Grapht<FhlGraph> = {
//     Grapht::new()
//   };
// }

/// Individual test setup
pub fn init_test() {
  LOGGING.call_once(|| {
    tracing_subscriber::fmt()
      .without_time()
      .compact()
      .with_env_filter("debug")
      .init()
  })
}

/// Simple wrapper to initialize each test with logging to the screen
macro_rules! db_test_fn {
  // Configure the logging
  (@init logging) => {
    pub use tracing::{debug, error, info, trace, warn, Subscriber};
    crate::common::init_test();
  };

  (@init db $name:expr) => {

  };
  (fn $name:ident () $body:expr) => {
    #[test]
    fn $name() {

      fn type_name_of<T>(_: T) -> &'static str {
        std::any::type_name::<T>()
      }
      let test_name = type_name_of($name);

      db_test_fn!(@init logging);
      info!(
        "\n\t\t\tStarting to run test: {}\n<------------------------------------------------------------------------------------->\n", test_name
      );
      debug!("Running test with args: {:?}", std::env::args());

      // Initialize the test database

      $body
    }
  };
}
