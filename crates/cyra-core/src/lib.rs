//! Cyra's offline foundation. No scientific simulator is implemented yet.
pub mod artifact;
pub mod protocol;
pub mod scientific;
pub mod service;
pub mod strict_json;

pub const PROJECT_NAME: &str = "Cyra";
pub const PROJECT_EXPANSION: &str = "Cybernetic Cellular Intelligence";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
