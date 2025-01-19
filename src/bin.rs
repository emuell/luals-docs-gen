#![doc = include_str!("../README.md")]

// -------------------------------------------------------------------------------------------------

mod error;
mod generator;
mod parser;

// -------------------------------------------------------------------------------------------------

use clap::Parser;
use std::path::Path;

use crate::{
    error::Error,
    generator::{generate_docs, options::Options},
};

fn main() -> Result<(), Error> {
    // get and validate args
    let options = Options::parse();
    if !Path::exists(&options.library) {
        return Err(Error::Options(format!(
            "source path does not exists: `{}`",
            options.library.as_path().to_string_lossy(),
        )));
    }
    if !Path::exists(&options.output) {
        return Err(Error::Options(format!(
            "output path does not exists: `{}`",
            options.output.as_path().to_string_lossy(),
        )));
    }
    // generate with options from arg...
    generate_docs(&options)
}
