use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use itertools::Itertools;
use regex::Regex;

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

                    let mut content = vec![];
                    content.push(h1(&file_stem));

                    let sorted_classes = Self::sort_classes(classes);

                    content.extend(
                        sorted_classes
                            .iter()
                            .fold(TocTree::new(), |mut toc, class| {
                                let url_root = "../";
                                let inner_toc =
                                    class.toc(url_root, &self.classes, &self.aliases, options);
                                toc.item(header_link(&class.name));
                                toc.inner(&inner_toc);
                                toc
                            })
                            .tree,
                    );

                    content.extend(sorted_classes.iter().map(|class| {
                        let url_root = "../";
                        let render_toc = false;
                        class.render(url_root, render_toc, &self.classes, &self.aliases, options)
                    }));

                    globals.push((file_stem.to_string(), content.join("  \n")));
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

                    let content =
                        class.render(url_root, true, &self.classes, &self.aliases, options);

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
        let custom_weight = |name: &str| -> usize { if name == "global" { 0 } else { 1 } };
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
    format!("[`{}`](#{})", text, hash)
}

fn enum_link(text: &str, url: &str, hash: &str) -> String {
    format!("[`{}`]({}.md#{})", text, url, hash)
}

fn alias_link(text: &str, hash: &str) -> String {
    format!("[`{}`](#{})", text, hash)
}

fn plain_link(text: &str, lowercase: bool) -> String {
    format!(
        "[{}](#{})",
        text,
        if lowercase {
            text.to_lowercase()
        } else {
            text.to_string()
        }
    )
}

fn section_link(section: &str) -> String {
    plain_link(section, true)
}

fn header_link(header: &str) -> String {
    plain_link(header, false)
}

fn quote(text: &str) -> String {
    format!("> {}", text.replace('\n', "\n> "))
}

fn description(desc: &str) -> String {
    if desc.is_empty() {
        String::new()
    } else {
        // remove file markdown file links from @see annotations
        let file_link_re = Regex::new(r"\[([^\]]+)\]\(file://[^\)]*\)").unwrap();
        // TODO: such links could in theory be resolved and rewritten as internal links while
        // generating the library...
        let desc = file_link_re.replace_all(desc, "`$1`");
        // add one more h level for examples
        let desc = desc.replace("### examples", "#### examples");
        // remove leading and trailing newlines
        let desc = desc.trim_matches('\n');
        quote(desc)
    }
}

fn hash(text: &str, hash: &str) -> String {
    format!("{} {{ #{} }}", text, hash)
}

fn divider() -> String {
    String::from("---")
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
            Kind::Array(k) => format!(
                "{}{}",
                k.link(url_root, file, options),
                file_link("[]", &format!("{}API/builtins/array", url_root))
            ),
            Kind::Nullable(k) => format!(
                "{}{}",
                k.as_ref().link(url_root, file, options),
                file_link("?", &format!("{}API/builtins/nil", url_root))
            ),
            Kind::Alias(alias) => alias_link(&alias.name, &alias.name),
            Kind::Function(f) => {
                f.short(url_root, file, options, NameFormat::Omit, NameFormat::Plain)
            }
            Kind::Table(k, v) => format!(
                "{}`<`{}, {}`>`",
                file_link("table", &format!("{}API/builtins/table", url_root)),
                k.as_ref().link(url_root, file, options),
                v.as_ref().link(url_root, file, options)
            ),
            Kind::Object(hm) => {
                let mut keys = hm.keys().cloned().collect::<Vec<String>>();
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
            Kind::Generic(s, parent_type) => {
                let generic_link = file_link(s, &format!("{}/API/builtins/generic", url_root));
                if let Some(parent_type) = parent_type {
                    format!(
                        "{}:{}",
                        generic_link,
                        parent_type.link(url_root, file, options)
                    )
                } else {
                    generic_link
                }
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Copy, Clone)]
enum NameFormat {
    Plain,
    Link,
    Omit,
}

impl Var {
    fn short(
        &self,
        url_root: &str,
        file: &Path,
        options: &Options,
        name_format: NameFormat,
    ) -> String {
        let kind = self.kind.link(url_root, file, options);

        if matches!(self.kind, Kind::SelfArg) {
            kind
        } else if let Some(name) = self.name.clone() {
            match name_format {
                NameFormat::Plain => format!("{} : {}", name, kind),
                NameFormat::Link => format!("{} : {}", header_link(&name), kind),
                NameFormat::Omit => kind,
            }
        } else {
            kind
        }
    }

    fn long(&self, url_root: &str, file: &Path, options: &Options) -> String {
        let desc = self.desc.clone().unwrap_or_default();
        format!(
            "{}{}",
            hash(
                &h3(&self.short(url_root, file, options, NameFormat::Plain)),
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
            let name = hash(&h3(&format!("`{}()`", name)), &name);
            self.with_desc(&self.with_returns(&name, url_root, file, options, NameFormat::Plain))
        } else {
            let params = self
                .params
                .iter()
                .map(|v| v.short(url_root, file, options, NameFormat::Plain))
                .collect::<Vec<String>>()
                .join(", ");

            self.with_desc(&self.with_returns(
                &hash(&format!("### {}({})", name, params), &name),
                url_root,
                file,
                options,
                NameFormat::Plain,
            ))
        }
    }
    fn short(
        &self,
        url_root: &str,
        file: &Path,
        options: &Options,
        name_format: NameFormat,
        arg_format: NameFormat,
    ) -> String {
        let name = self
            .name
            .clone()
            .map(|n| match name_format {
                NameFormat::Plain => n,
                NameFormat::Link => header_link(&n),
                NameFormat::Omit => String::default(),
            })
            .unwrap_or_default();
        let params = Self::render_vars(&self.params, url_root, file, options, arg_format);
        let returns = Self::render_vars(&self.returns, url_root, file, options, arg_format);

        format!(
            "{} ({}){}",
            name,
            params,
            if returns.is_empty() {
                String::default()
            } else {
                format!(" `->` {}", returns)
            }
        )
    }
    fn render_vars(
        vars: &[Var],
        url_root: &str,
        file: &Path,
        options: &Options,
        name_format: NameFormat,
    ) -> String {
        vars.iter()
            .map(|v| v.short(url_root, file, options, name_format))
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
    fn with_returns(
        &self,
        head: &str,
        url_root: &str,
        file: &Path,
        options: &Options,
        arg_format: NameFormat,
    ) -> String {
        let returns = self
            .returns
            .iter()
            .map(|v| v.short(url_root, file, options, arg_format))
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

struct TocTree {
    depth: usize,
    tree: Vec<String>,
}

impl TocTree {
    fn new() -> Self {
        Self {
            depth: 0,
            tree: vec![],
        }
    }
    fn indent(depth: usize, s: &str) -> String {
        let indent = "\t".repeat(depth);
        format!("{}{}", indent, s)
    }
    fn li(depth: usize, s: &str) -> String {
        Self::indent(depth, &format!("* {}", s))
    }
    fn item(&mut self, item: String) {
        self.tree.push(Self::li(self.depth, &item));
    }
    fn list<T>(&mut self, items: &[T], map_fun: impl Fn(&T) -> String) {
        for item in items.iter().map(map_fun) {
            self.item(item);
        }
    }
    fn push(&mut self) {
        self.depth += 1;
    }
    fn pop(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }
    fn section<T>(&mut self, header: String, items: &[T], map_fun: impl Fn(&T) -> String) {
        self.item(header);
        self.push();
        self.list(items, map_fun);
        self.pop();
    }
    fn inner(&mut self, items: &[String]) {
        self.push();
        for item in items.iter() {
            self.tree.push(Self::indent(self.depth, item))
        }
        self.pop();
    }
}

impl Class {
    const CONSTANTS: &'static str = "Constants";
    const PROPERTIES: &'static str = "Properties";
    const FUNCTIONS: &'static str = "Functions";
    const STRUCTS: &'static str = "Structs";
    const ALIASES: &'static str = "Aliases";

    fn resolve(
        &self,
        structs: &HashMap<String, Class>,
        aliases: &HashMap<String, Alias>,
        options: &Options,
    ) -> (Vec<Class>, Vec<Alias>) {
        // append used local classes and aliases
        let (local_class_names, local_alias_names) = match options.order {
            // when organizing by files, inline used aliases only
            OutputOrder::ByFile => (HashSet::new(), self.collect_local_aliases(aliases)),
            // when organizing by class, inline everything the class refers to
            OutputOrder::ByClass => self.collect_local_types(structs, aliases),
        };
        (
            if self.scope != Scope::Local && !local_class_names.is_empty() && !structs.is_empty() {
                let mut class_keys: Vec<String> = structs.keys().cloned().collect();
                class_keys.sort();
                class_keys
                    .into_iter()
                    .filter(|n| local_class_names.contains(n))
                    .map(|n| structs.get(&n).unwrap().clone())
                    .collect::<Vec<_>>()
            } else {
                vec![]
            },
            if !local_alias_names.is_empty() {
                let mut alias_keys: Vec<String> = aliases.keys().cloned().collect();
                alias_keys.sort();
                alias_keys
                    .into_iter()
                    .filter(|n| local_alias_names.contains(n))
                    .map(|n| aliases.get(&n).unwrap().clone())
                    .collect::<Vec<_>>()
            } else {
                vec![]
            },
        )
    }

    fn toc(
        &self,
        url_root: &str,
        structs: &HashMap<String, Class>,
        aliases: &HashMap<String, Alias>,
        options: &Options,
    ) -> Vec<String> {
        let mut toc = TocTree::new();

        let file = self.file.clone().unwrap();

        if !self.enums.is_empty() || !self.constants.is_empty() {
            toc.item(section_link(Self::CONSTANTS));
            toc.push();
            toc.list(&self.constants, |v| {
                v.short(url_root, &file, options, NameFormat::Link)
            });
            toc.list(&self.enums, |e| {
                let name = e.name.clone();
                let end = Class::get_end(&name).unwrap_or(&name);
                header_link(end)
            });
            toc.pop();
        };

        if !self.fields.is_empty() {
            toc.section(section_link(Self::PROPERTIES), &self.fields, |v| {
                v.short(url_root, &file, options, NameFormat::Link)
            });
        };

        if !self.functions.is_empty() {
            toc.section(section_link(Self::FUNCTIONS), &self.functions, |f| {
                f.short(url_root, &file, options, NameFormat::Link, NameFormat::Omit)
            });
        };

        let (resolved_structs, resolved_aliases) = self.resolve(structs, aliases, options);

        if !resolved_structs.is_empty() {
            toc.item(section_link(Self::STRUCTS));
            toc.push();
            resolved_structs.iter().for_each(|c| {
                let inner_toc = c.toc(url_root, structs, aliases, options);
                toc.item(header_link(&c.name));
                toc.inner(&inner_toc);
            });
            toc.pop();
        };

        if !resolved_aliases.is_empty() {
            toc.item(section_link(Self::ALIASES));
            toc.push();
            resolved_aliases.iter().for_each(|c| {
                toc.item(header_link(&c.name));
            });
            toc.pop();
        };

        toc.tree
    }

    fn header(&self) -> Vec<String> {
        let name = if self.name == "global" {
            "global"
        } else {
            &self.name
        };

        // add an invisible element with the basename of the class to
        // force the search index to include it
        let basename = name.split('.').next_back().unwrap();
        let with_tag = if basename != name {
            format!(
                "{} <span style=\"visibility: hidden\">{}</span>",
                name, basename
            )
        } else {
            name.to_string()
        };

        let mut header = vec![h1(&hash(&with_tag, name))];

        if !self.desc.is_empty() {
            header.push(description(&self.desc))
        }

        header
    }

    fn body(
        &self,
        url_root: &str,
        structs: &HashMap<String, Class>,
        aliases: &HashMap<String, Alias>,
        options: &Options,
    ) -> Vec<String> {
        let file = self.file.clone().unwrap_or_default();

        let mut body = vec![];

        if !self.enums.is_empty() || !self.constants.is_empty() {
            body.push(divider());
            body.push(h2(Self::CONSTANTS));
            body.extend(self.enums.iter().map(|e| {
                let name = e.name.clone();
                let end = Class::get_end(&name).unwrap_or(&name);
                format!("{}\n{}", hash(&h3(end), end), description(&e.desc))
            }));
            body.extend(
                self.constants
                    .iter()
                    .map(|v| v.long(url_root, &file, options)),
            );
        };

        if !self.fields.is_empty() {
            body.push(divider());
            body.push(h2(Self::PROPERTIES));
            body.extend(self.fields.iter().map(|v| v.long(url_root, &file, options)));
        };

        if !self.functions.is_empty() {
            body.push(divider());
            body.push(h2(Self::FUNCTIONS));
            body.extend(
                self.functions
                    .iter()
                    .map(|f| f.long(url_root, &file, options)),
            );
        };

        let (resolved_structs, resolved_aliases) = self.resolve(structs, aliases, options);

        if !resolved_structs.is_empty() {
            body.push(divider());
            body.push(h1(Self::STRUCTS));
            for s in resolved_structs.iter() {
                body.push({
                    let render_toc = false;
                    s.render(url_root, render_toc, structs, aliases, options)
                })
            }
        };

        if !resolved_aliases.is_empty() {
            body.push(divider());
            body.push(h1(Self::ALIASES));
            for a in resolved_aliases.iter() {
                body.push(divider());
                body.push({
                    let file = self.file.clone().unwrap_or_default();
                    a.render(url_root, &file, options)
                })
            }
            body.push(divider());
        };

        body
    }

    fn render(
        &self,
        url_root: &str,
        render_toc: bool,
        structs: &HashMap<String, Class>,
        aliases: &HashMap<String, Alias>,
        options: &Options,
    ) -> String {
        let mut page = vec![];

        page.extend(self.header());

        if render_toc {
            page.extend(self.toc(url_root, structs, aliases, options))
        }

        page.extend(self.body(url_root, structs, aliases, options));

        page.join("\n")
    }
}
