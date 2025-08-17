pub mod checker;
pub mod types;

pub use checker::typecheck_program;
pub use types::*;

#[cfg(test)]
mod tests;

#[cfg(feature = "examples")]
pub mod example;
