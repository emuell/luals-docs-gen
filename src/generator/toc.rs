use std::{
    fs::*,
    io::{Read, Write},
    path::Path,
};

use crate::{error::Error, generator::options::Options, parser::types::Class};

// -------------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(crate) struct TocEntry {
    pub file_path: String,
    pub file_name: String,
    pub link: String,
}

impl TocEntry {
    pub fn from(name: &str, options: &Options) -> Self {
        let mut file_name = name.to_string();
        let mut display_name = name.to_string();
        let mut file_path = String::new();
        let mut level = String::from("  ");
        if name.contains('/') {
            // reconstruct relative paths in TOC
            let full_name = file_name.clone();
            let mut splits = full_name.split('/').collect::<Vec<_>>();
            level = "  ".repeat(splits.len());
            file_name = splits.remove(splits.len() - 1).to_string();
            display_name.clone_from(&file_name);
            file_path = splits
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
                .join("/");
            file_path.push('/');
        } else if Class::belongs_to_namespace(name, &options.namespace) {
            if name == options.namespace {
                // namespace root
                display_name = name.to_string();
            } else {
                // members of namespace
                display_name = name.replace(&(options.namespace.clone() + "."), "");
                file_path = options.namespace.clone() + "/";
                level = "  ".repeat(2);
            }
        } else {
            // everything else...
            display_name = match name {
                "builtins" => "Builtin Types".to_string(),
                "modules" => "Module Extensions".to_string(),
                "structs" => "Helper Types".to_string(),
                "global" => "Globals".to_string(),
                _ => name.to_string(),
            }
        }
        let link = format!(
            "{}- [{}](API/{}{}.md)",
            level, display_name, file_path, file_name
        );
        Self {
            file_path,
            file_name,
            link,
        }
    }
}

// -------------------------------------------------------------------------------------------------

pub(crate) fn replace_toc_in_file(file_path: &Path, toc_links: &[String]) -> Result<(), Error> {
    let mut file = File::open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    drop(file);

    let mut lines = content.lines().collect::<Vec<&str>>();
    let toc_start_line = lines
        .iter()
        .enumerate()
        .find(|(_i, l)| l.contains("<!-- API TOC START -->"))
        .expect("Failed to locate <API TOC START> line in Summary.md")
        .0;
    let toc_end_line = lines
        .iter()
        .enumerate()
        .find(|(_i, l)| l.contains("<!-- API TOC END -->"))
        .expect("Failed to locate <API TOC END> line in Summary.md")
        .0;
    lines.splice(
        (toc_start_line + 1)..toc_end_line,
        toc_links.iter().map(String::as_str),
    );

    let mut file = File::create(file_path)?;
    file.write_all(lines.join("\n").as_bytes())?;
    Ok(())
}
