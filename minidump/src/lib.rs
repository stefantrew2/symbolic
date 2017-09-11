//! Provides minidump support.
extern crate breakpad;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde_derive;
extern crate symbolic_common;
extern crate goblin;
extern crate gimli;

// TODO(ja): Move
mod errors;
mod common;
mod debuginfo;

pub mod cfi;
pub mod minidump;
pub mod registers;

pub use errors::*;
pub use debuginfo::*;
pub use common::*;
pub use cfi::*;
pub use minidump::*;
