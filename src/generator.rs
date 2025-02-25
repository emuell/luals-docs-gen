pub(crate) mod library;
pub(crate) mod options;
pub(crate) mod render;
pub(crate) mod toc;

// -------------------------------------------------------------------------------------------------

use std::{fs::*, io::Write, path::Path};

use crate::{
    error::Error,
    generator::{
        library::Library,
        options::{Options, OutputOrder},
        toc::{replace_toc_in_file, TocEntry},
    },
};

/// Generate API docs with the given [`Options`](options::Options).
///
/// This will download a copy of a lua language server, if necessary, patch it and
/// then runs it on the given type annotated library file to generate documentation.
///
/// Resulting markdown files are generated and written to the output path as specified
/// by the options.
pub fn generate_docs(options: &Options) -> Result<(), Error> {
    // validate options
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
    if options.order == OutputOrder::ByClass && options.namespace.is_empty() {
        return Err(Error::Options(
            "the order by-class option requires a namespace to be set too".to_string(),
        ));
    }

    // parse API and create docs
    let lib = Library::from_path(&options.library, options)?;
    let docs = lib.export_docs(options);

    // clear previously generated API doc files (except README.md)
    let api_path = options.output.clone().join("API");
    println!("Writing docs to: `{}`", api_path.to_string_lossy());
    if !api_path.exists() {
        create_dir(api_path.clone())?;
    } else {
        for entry in read_dir(&api_path)?.flatten() {
            if entry.path().is_dir() {
                remove_dir_all(entry.path())?;
            } else if entry
                .path()
                .file_name()
                .is_some_and(|file| !file.eq_ignore_ascii_case("README.md"))
            {
                remove_file(entry.path())?;
            }
        }
    }

    // write docs to files and keep track of the TOC
    let mut toc_links = vec![];
    for (name, content) in &docs {
        let toc_entry = TocEntry::from(name, options);
        toc_links.push(toc_entry.link);
        let dir_path = api_path.clone().join(toc_entry.file_path.clone());
        let file_path = dir_path.clone().join(toc_entry.file_name + ".md");
        if !dir_path.exists() {
            create_dir(dir_path)?;
        }
        println!("Creating '{}'", file_path.to_string_lossy());
        let mut file = File::create(file_path)?;
        file.write_all(content.as_bytes())?;
    }

    // update API TOC in SUMMARY.md, if it exists
    let summary_file = options.output.clone().join("SUMMARY.md");
    if summary_file.exists() {
        println!("Updating TOC at: '{}'", summary_file.to_string_lossy());
        replace_toc_in_file(&summary_file, &toc_links)?;
    }
    Ok(())
}
