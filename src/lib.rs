// allow tests to use std
#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod errors;
mod gen_vec;
mod graph;
mod graph_diff;

pub use errors::*;
pub use graph::*;
pub use graph_diff::GraphDiff;

#[cfg(test)]
mod graph_tests;
