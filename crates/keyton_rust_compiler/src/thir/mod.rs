pub mod checker;
pub mod types;

pub use checker::{typecheck_program, typecheck_program_with_env};
pub use types::*;

#[cfg(test)]
mod tests;

#[cfg(feature = "examples")]
pub mod example;
