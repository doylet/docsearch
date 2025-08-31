/// Query expansion module for enhanced recall
///
/// This module implements various query expansion strategies to improve search recall
/// by generating multiple query variants and combining their results.
pub mod expansion;
pub mod strategies;

#[cfg(test)]
pub mod examples;

pub use expansion::*;
pub use strategies::*;
