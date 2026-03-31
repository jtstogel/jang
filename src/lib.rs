#![cfg_attr(not(test), deny(clippy::unwrap_used))]

mod error;
pub mod interpreter;
pub mod parser;
mod source_location;
