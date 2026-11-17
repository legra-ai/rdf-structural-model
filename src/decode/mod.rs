//! Depth-checked Serde decoding of recursive triple terms.
//!
//! The derived wire shapes are reproduced exactly; only the recursion carries
//! a [`crate::NestingDepth`] so hostile input cannot exhaust the stack.
mod generalized;
mod identifier;
mod standard;
