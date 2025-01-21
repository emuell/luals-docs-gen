#![doc = include_str!("../README.md")]

// -------------------------------------------------------------------------------------------------

mod error;
mod generator;
mod parser;

// -------------------------------------------------------------------------------------------------

// re-export generator and error as public interface
pub use error::Error;
pub use generator::{
    generate_docs,
    options::{Options, OutputOrder},
};
