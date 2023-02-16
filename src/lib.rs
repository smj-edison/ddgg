// allow tests to use std
#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod errors;
pub mod gen_vec;
pub mod graph;
pub mod graph_diff;

#[cfg(test)]
mod graph_tests;
