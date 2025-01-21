use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use itertools::Itertools;

use crate::{
    generator::{
        library::Library,
        options::{Options, OutputOrder},
    },
    parser::types::*,
};

// -------------------------------------------------------------------------------------------------

impl Library {
    /// render each page inside the library as a list of string tuples (name, content)
    pub fn export_docs(&self, options: &Options) -> Vec<(String, String)> {
        // collect and sort by file
        let mut globals = vec![];
        let mut modules = vec![];
        match options.order {
            // split classes into globals and modules and organize by source file
            OutputOrder::ByFile => {
                for (path, classes) in
                    self.classes_by_file_in_scopes(&[Scope::Global, Scope::Local])
                {
                    if path.to_string_lossy().is_empty() {
                        // skip classes which have no file path (Lua internals)
                        continue;
                    }
                    let file_stem = path
                        .file_stem()
                        .map(|v| v.to_string_lossy())
                        .expect("expecting class to have a valid source file path");

                    let mut content = String::new();
                    content.push_str(&h1(&file_stem));
                    content.push_str("\n<!-- toc -->\n");

                    for class in Self::sort_classes(classes) {
                        let url_root = "../";
                        let render_toc = false; // we already added a toc here
                        content.push_str(&class.render(
                            url_root,
                            render_toc,
                            &self.classes,
                            &self.aliases,
                            options,
                        ));
                        content.push_str("\n\n");
                    }
                    globals.push((file_stem.to_string(), content));
                }

                for class in self.classes_in_scopes(&[Scope::Modules]) {
                    let url_root = "../../";
                    let render_toc = false;
                    let content =
                        class.render(url_root, render_toc, &self.classes, &self.aliases, options);
                    modules.push((String::from("modules/") + &class.name, content));
                }
            }
            // create separate files for each class in the root namespace
            OutputOrder::ByClass => {
                for (class_name, class) in &self.classes {
                    // url_root is the path to /API folder
                    let url_root = if class_name == &options.namespace {
                        "../" // namespace root
                    } else {
                        "../../" // namespace childs
                    };
                    let mut content = String::new();
                    let render_toc = true;
                    content.push_str(&class.render(
                        url_root,
                        render_toc,
                        &self.classes,
                        &self.aliases,
                        options,
                    ));
                    match class.scope {
                        Scope::Global => globals.push((class_name.clone(), content)),
                        Scope::Modules => {
                            modules.push(("modules/".to_string() + class_name, content))
                        }
                        Scope::Local => (),    // inlined in global classes
                        Scope::Builtins => (), // handled separately below
                    }
                }
            }
        }

        // add builtin classes
        let mut builtins = vec![];
        for class in Library::builtin_classes() {
            let url_root = "../../";
            let render_toc = false;
            let content = class.render(url_root, render_toc, &self.classes, &self.aliases, options);
            builtins.push((String::from("builtins/") + &class.name, content));
        }

        // create final docs
        let mut docs: Vec<(String, String)> = vec![];
        docs.append(&mut globals);

        if !modules.is_empty() {
            docs.push(("modules".to_string(), "# Lua Module Extensions".to_string()));
            docs.append(&mut modules);
        }
        if !builtins.is_empty() {
            docs.push(("builtins".to_string(), "# Lua Builtin Types".to_string()));
            docs.append(&mut builtins);
        }
        docs = docs
            .iter()
            .unique_by(|(name, _)| name.to_ascii_lowercase())
            .cloned()
            .collect::<Vec<_>>();
        Self::sort_docs(docs)
    }

    fn classes_in_scopes(&self, scopes: &[Scope]) -> Vec<Class> {
        self.classes
            .values()
            .filter(|&c| scopes.contains(&c.scope))
            .cloned()
            .collect()
    }

    fn classes_by_file_in_scopes(&self, scopes: &[Scope]) -> HashMap<PathBuf, Vec<Class>> {
        let mut map = HashMap::<PathBuf, Vec<Class>>::new();
        for class in self.classes_in_scopes(scopes) {
            let file = class.file.clone().unwrap_or_default();
            if let Some(classes) = map.get_mut(&file) {
                classes.push(class.clone());
            } else {
                map.insert(file.clone(), vec![class.clone()]);
            }
        }
        map
    }

    fn sort_classes(mut classes: Vec<Class>) -> Vec<Class> {
        let custom_weight = |name: &str| -> usize {
            if name == "global" {
                0
            } else {
                1
            }
        };
        classes.sort_by_key(|class| (custom_weight(&class.name), class.name.to_lowercase()));
        classes
    }

    fn sort_docs(mut docs: Vec<(String, String)>) -> Vec<(String, String)> {
        let custom_weight = |name: &str| -> usize {
            if name == "global" {
                0
            } else if name.starts_with("modules") {
                99
            } else if name.starts_with("builtins") {
                100
            } else {
                10
            }
        };
        docs.sort_by_key(|(name, _)| (custom_weight(name), name.to_lowercase()));
        docs
    }
}

// -------------------------------------------------------------------------------------------------

fn heading(text: &str, level: usize) -> String {
    format!("{} {}", "#".repeat(level), text)
}

fn h1(text: &str) -> String {
    heading(text, 1)
}

fn h2(text: &str) -> String {
    heading(text, 2)
}

fn h3(text: &str) -> String {
    heading(text, 3)
}

fn file_link(text: &str, url: &str) -> String {
    format!("[`{}`]({}.md)", text, url)
}

fn class_link(text: &str, url: &str, hash: &str) -> String {
    format!("[`{}`]({}.md#{})", text, url, hash)
}

fn local_class_link(text: &str, hash: &str) -> String {
    format!("[`{}`](#{})", text, hash.to_lowercase())
}

fn enum_link(text: &str, url: &str, hash: &str) -> String {
    format!("[`{}`]({}.md#{})", text, url, hash)
}

fn alias_link(text: &str, hash: &str) -> String {
    format!("[`{}`](#{})", text, hash)
}

fn quote(text: &str) -> String {
    format!("> {}", text.replace('\n', "\n> "))
}

fn description(desc: &str) -> String {
    quote(
        desc.replace("### examples", "#### examples")
            .trim_matches('\n'),
    )
}

fn hash(text: &str, hash: &str) -> String {
    format!("{}<a name=\"{}\"></a>", text, hash)
}

// -------------------------------------------------------------------------------------------------

impl LuaKind {
    fn link(&self, url_root: &str) -> String {
        let text = self.show();
        file_link(&text, &(format!("{}API/builtins/", url_root) + &text))
    }
}

// -------------------------------------------------------------------------------------------------

impl Kind {
    fn link(&self, url_root: &str, file: &Path, options: &Options) -> String {
        match self {
            Kind::Lua(lk) => lk.link(url_root),
            Kind::Literal(k, s) => match k.as_ref() {
                LuaKind::String => format!("`\"{}\"`", s),
                LuaKind::Integer | LuaKind::Number => format!("`{}`", s.clone()),
                _ => s.clone(),
            },
            Kind::Class(class) => match class.scope {
                Scope::Local | Scope::Global => match options.order {
                    OutputOrder::ByFile => {
                        let file = class.file.clone().unwrap_or_default();
                        let file_stem = file
                            .file_stem()
                            .map(|v| v.to_string_lossy())
                            .unwrap_or("[unknown file]".into());
                        class_link(
                            &class.name,
                            &(url_root.to_string()
                                + &class.scope.path_prefix(&options.namespace)
                                + &file_stem),
                            &class.name,
                        )
                    }
                    OutputOrder::ByClass => {
                        if class.scope == Scope::Local {
                            local_class_link(&class.name, &class.name)
                        } else {
                            file_link(
                                &class.name,
                                &(url_root.to_string()
                                    + &class.scope.path_prefix(&options.namespace)
                                    + &class.name),
                            )
                        }
                    }
                },
                _ => file_link(
                    &class.name,
                    &(url_root.to_string()
                        + &class.scope.path_prefix(&options.namespace)
                        + &class.name),
                ),
            },
            Kind::Enum(kinds) => kinds
                .iter()
                .map(|k| k.link(url_root, file, options))
                .collect::<Vec<String>>()
                .join(" | "),
            Kind::EnumRef(enumref) => match options.order {
                OutputOrder::ByFile => {
                    let file = enumref.file.clone().unwrap_or(PathBuf::new());
                    let file_stem = file
                        .file_stem()
                        .map(|v| v.to_string_lossy())
                        .unwrap_or("[unknown file]".into());
                    enum_link(
                        &enumref.name,
                        &(url_root.to_string()
                            + &Scope::Global.path_prefix(&options.namespace)
                            + &file_stem),
                        &enumref.name,
                    )
                }
                OutputOrder::ByClass => enum_link(
                    &enumref.name,
                    Class::get_base(&enumref.name).unwrap_or(&enumref.name),
                    Class::get_end(&enumref.name).unwrap_or_default(),
                ),
            },
            Kind::SelfArg => format!("[*self*]({}API/builtins/self.md)", url_root),
            Kind::Array(k) => format!("{}[]", k.link(url_root, file, options)),
            Kind::Nullable(k) => format!(
                "{}{}",
                k.as_ref().link(url_root, file, options),
                file_link("?", &format!("{}API/builtins/nil", url_root))
            ),
            Kind::Alias(alias) => alias_link(&alias.name, &alias.name),
            Kind::Function(f) => f.short(url_root, file, options),
            Kind::Table(k, v) => format!(
                "table<{}, {}>",
                k.as_ref().link(url_root, file, options),
                v.as_ref().link(url_root, file, options)
            ),
            Kind::Object(hm) => {
                let mut keys = hm.iter().map(|(k, _)| k.clone()).collect::<Vec<String>>();
                keys.sort();
                let fields = keys
                    .iter()
                    .map(|k| {
                        format!(
                            "{} : {}",
                            k,
                            hm.get(k).unwrap().link(url_root, file, options)
                        )
                    })
                    .collect::<Vec<String>>()
                    .join(", "); // TODO print on newlines?
                format!("{{ {} }}", fields)
            }
            Kind::Variadic(k) => format!("...{}", k.link(url_root, file, options)),
            Kind::Unresolved(s) => s.clone(),
        }
    }
}

// -------------------------------------------------------------------------------------------------

impl Var {
    fn short(&self, url_root: &str, file: &Path, options: &Options) -> String {
        if matches!(self.kind, Kind::SelfArg) {
            self.kind.link(url_root, file, options)
        } else if let Some(name) = self.name.clone() {
            format!("{} : {}", name, self.kind.link(url_root, file, options))
        } else {
            self.kind.link(url_root, file, options)
        }
    }

    fn long(&self, url_root: &str, file: &Path, options: &Options) -> String {
        let desc = self.desc.clone().unwrap_or_default();
        format!(
            "{}{}",
            hash(
                &h3(&self.short(url_root, file, options)),
                &self.name.clone().unwrap_or_default()
            ),
            if desc.is_empty() {
                desc
            } else {
                format!("\n{}\n", description(&desc))
            }
        )
    }
}

// -------------------------------------------------------------------------------------------------

impl Alias {
    fn render(&self, url_root: &str, file: &Path, options: &Options) -> String {
        format!(
            "{}\n{}  \n{}",
            hash(&h3(&self.name), &self.name),
            self.kind.link(url_root, file, options),
            self.desc
                .clone()
                .map(|d| description(d.as_str()))
                .unwrap_or_default()
        )
    }
}

// -------------------------------------------------------------------------------------------------

impl Function {
    fn long(&self, url_root: &str, file: &Path, options: &Options) -> String {
        let name = self.name.clone().unwrap_or("fun".to_string());
        if self.params.is_empty() {
            let name = hash(&h3(&format!("`{}()`", &name)), &name);
            self.with_desc(&self.with_returns(&name, url_root, file, options))
        } else {
            let params = self
                .params
                .iter()
                .map(|v| v.short(url_root, file, options))
                .collect::<Vec<String>>()
                .join(", ");

            self.with_desc(&self.with_returns(
                &hash(&format!("### {}({})", &name, params), &name),
                url_root,
                file,
                options,
            ))
        }
    }
    fn short(&self, url_root: &str, file: &Path, options: &Options) -> String {
        if self.params.is_empty() && self.returns.is_empty() {
            return self.empty();
        }
        let returns = Self::render_vars(&self.returns, url_root, file, options);
        format!(
            "{}({}){}",
            &self.name.clone().unwrap_or_default(),
            Self::render_vars(&self.params, url_root, file, options),
            if returns.is_empty() {
                returns
            } else {
                format!(" `->` {}", returns)
            }
        )
    }
    fn empty(&self) -> String {
        format!("{}()", &self.name.clone().unwrap_or("fun".to_string()))
    }
    fn render_vars(vars: &[Var], url_root: &str, file: &Path, options: &Options) -> String {
        vars.iter()
            .map(|v| v.short(url_root, file, options))
            .collect::<Vec<String>>()
            .join(", ")
    }
    fn with_desc(&self, head: &str) -> String {
        let desc = self.desc.clone().unwrap_or_default();
        if desc.is_empty() {
            head.to_string()
        } else {
            format!("{}\n{}", head, description(&desc))
        }
    }
    fn with_returns(&self, head: &str, url_root: &str, file: &Path, options: &Options) -> String {
        let returns = self
            .returns
            .iter()
            .map(|v| v.short(url_root, file, options))
            .collect::<Vec<String>>()
            .join(", ");
        if returns.is_empty() {
            head.to_string()
        } else {
            format!("{}\n`->`{}  \n", head, returns)
        }
    }
}

// -------------------------------------------------------------------------------------------------

impl Class {
    fn render(
        &self,
        url_root: &str,
        render_toc: bool,
        structs: &HashMap<String, Class>,
        aliases: &HashMap<String, Alias>,
        options: &Options,
    ) -> String {
        let name = if self.name == "global" {
            "Global"
        } else {
            &self.name
        };
        let file = self.file.clone().unwrap_or_default();

        let mut content = vec![h1(&hash(name, name))];

        if !self.desc.is_empty() {
            content.push(description(&self.desc))
        }

        if render_toc {
            content.push("\n<!-- toc -->\n".to_string());
        }

        if !self.enums.is_empty() || !self.constants.is_empty() {
            let enums = &self.enums;
            let constants = &self.constants;
            content.push(format!(
                "{}\n{}\n{}",
                h2("Constants"),
                enums
                    .iter()
                    .map(|e| {
                        let name = e.name.clone();
                        let end = Class::get_end(&name).unwrap_or(&name);
                        format!("{}\n{}", hash(&h3(end), end), description(&e.desc))
                    })
                    .collect::<Vec<String>>()
                    .join("\n"),
                constants
                    .iter()
                    .map(|v| v.long(url_root, &file, options))
                    .collect::<Vec<String>>()
                    .join("\n")
            ))
        }

        if !self.fields.is_empty() {
            content.push("\n---".to_string());
            content.push(format!(
                "{}\n{}\n",
                h2("Properties"),
                self.fields
                    .iter()
                    .map(|v| v.long(url_root, &file, options))
                    .collect::<Vec<String>>()
                    .join("\n")
            ))
        }

        let functions = &self.functions;
        if !functions.is_empty() {
            content.push("\n---".to_string());
            content.push(format!(
                "{}\n{}",
                h2("Functions"),
                functions
                    .iter()
                    .map(|f| f.long(url_root, &file, options))
                    .collect::<Vec<String>>()
                    .join("\n")
            ))
        }

        // append used local classes and aliases
        let (local_class_names, local_alias_names) = match options.order {
            // when organizing by files, inline used aliases only
            OutputOrder::ByFile => (HashSet::new(), self.collect_local_aliases(aliases)),
            // when organizing by class, inline everything the class refers to
            OutputOrder::ByClass => self.collect_local_types(structs, aliases),
        };

        // append all used local classes (structs)
        if self.scope != Scope::Local && !local_class_names.is_empty() {
            content.push("\n\n\n---".to_string());
            content.push(h2("Structs"));
            let mut class_names: Vec<&String> = structs.keys().collect();
            class_names.sort();
            for name in class_names {
                if local_class_names.contains(name) {
                    let struct_ = structs.get(name).unwrap();
                    let render_toc = false;
                    content.push(struct_.render(url_root, render_toc, structs, aliases, options));
                }
            }
        }

        // append all used local aliases
        if !local_alias_names.is_empty() {
            content.push("\n\n\n---".to_string());
            content.push(h2("Aliases"));
            let mut alias_names: Vec<&String> = aliases.keys().collect();
            alias_names.sort();
            for name in alias_names {
                if local_alias_names.contains(name) {
                    let file = self.file.clone().unwrap_or_default();
                    content.push(aliases.get(name).unwrap().render(url_root, &file, options));
                    content.push(String::new());
                }
            }
        }

        content.push("\n".to_string());
        content.join("  \n")
    }
}
