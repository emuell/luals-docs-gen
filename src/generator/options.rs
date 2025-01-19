use std::path::PathBuf;

// -------------------------------------------------------------------------------------------------

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
}
