use std::path::PathBuf;

// -------------------------------------------------------------------------------------------------

/// How to structure the documentation output.
#[derive(Debug, Clone, Default, PartialEq, clap::ValueEnum)]
pub enum OutputOrder {
    #[default]
    /// Generate a markdown file for each **Lua source file** and only inline used aliases.
    ByFile,
    /// Generate a markdown file for each **Lua class** and inline all used local structs
    /// and aliases into it. Requires a root namespace to be set too.
    /// 
    /// All classes which are not part of the root namespace will be added to a `modules`
    /// file as globals.
    ByClass,
}

/// Options for the API doc generator.
///
/// Includes clap argument definitions, when using the generator from the command line.
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Options {
    /// LuaLS documented library source path.
    #[arg(name = "library_path")]
    pub library: PathBuf,
    /// Target path where markdown files are written.
    #[arg(name = "output_path")]
    pub output: PathBuf,
    /// Classes which should be excluded from the docs.
    #[arg(name = "excluded_classes", value_parser, num_args = 0..)]
    pub excluded_classes: Vec<String>,
    /// The output structure of the docs.
    #[arg(name = "order", short, long, value_enum, default_value_t)]
    pub order: OutputOrder,
    /// When set, use the given Lua table/class name as root namespace. 
    /// This only applies when `order` is set to `"by-class"`.  
    #[arg(name = "namespace", short, long, default_value = "")]
    pub namespace: String,
}
