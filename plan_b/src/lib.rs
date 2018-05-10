// Copyright © 2018 Po Huit
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Plan B: EVE route planner library with options
//!
//! This crate provides facilities for routing in the New
//! Eden universe.

#[macro_use]
extern crate serde_derive;

pub mod map;
pub mod search;

pub use map::*;
pub use search::*;
