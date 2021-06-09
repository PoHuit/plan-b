// Copyright © 2018 Po Huit
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Plan B: EVE route planner library with options
//!
//! This crate provides facilities for routing in the New
//! Eden universe.

pub mod map;
pub mod search;

pub use crate::map::*;
pub use crate::search::*;
