#![doc = include_str!("../README.md")]

// -------------------------------------------------------------------------------------------------

mod error;
mod generator;
mod parser;

// -------------------------------------------------------------------------------------------------

use clap::Parser;

use crate::{
    error::Error,
    generator::{generate_docs, options::Options},
};

fn main() -> Result<(), Error> {
    // generate with options from the command line
    generate_docs(&Options::parse())
}
