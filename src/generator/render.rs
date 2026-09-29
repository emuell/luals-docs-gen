use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
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

struct RenderContext<'a> {
    url_root: &'a str,
    options: &'a Options,
    /// if the element is being rendered nested inside another class
    /// the parent will be prepended to header ids
    parent: Option<String>,
}

impl<'a> RenderContext<'a> {
    fn id(&self, name: &str) -> String {
        self.parent
            .as_ref()
            .map(|p| format!("{p}.{name}"))
            .unwrap_or(name.to_string())
    }
}

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

                    let ctx = RenderContext {
                        url_root: "../",
                        options,
                        parent: None,
                    };
                    content.extend(
                        sorted_classes
                            .iter()
                            .fold(TocTree::new(), |mut toc, class| {
                                let inner_toc = class.toc(
                                    ctx.url_root,
                                    &self.classes,
                                    &self.aliases,
                                    ctx.options,
                                    true,
                                );
                                toc.item(header_link(&ctx, &class.name));
                                toc.inner(&inner_toc);
                                toc
                            })
                            .tree,
                    );

                    content.extend(sorted_classes.iter().map(|class| {
                        let url_root = "../";
                        let render_toc = false;
                        class.render(
                            url_root,
                            render_toc,
                            &self.classes,
                            &self.aliases,
                            options,
                            true,
                        )
                    }));

                    globals.push((file_stem.to_string(), content.join("  \n")));
                }

                let ctx = RenderContext {
                    url_root: "../../",
                    options,
                    parent: None,
                };
                for class in self.classes_in_scopes(&[Scope::Modules]) {
                    let render_toc = false;
                    let content = class.render(
                        ctx.url_root,
                        render_toc,
                        &self.classes,
                        &self.aliases,
                        ctx.options,
                        true,
                    );
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
                        class.render(url_root, true, &self.classes, &self.aliases, options, false);

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
            let content = class.render(
                url_root,
                render_toc,
                &self.classes,
                &self.aliases,
                options,
                false,
            );
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

fn header_link(ctx: &RenderContext, name: &str) -> String {
    format!("[{}](#{})", name, ctx.id(name))
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
    fn link(&self, ctx: &RenderContext) -> String {
        let text = self.show();
        file_link(&text, &(format!("{}API/builtins/", ctx.url_root) + &text))
    }
}

// -------------------------------------------------------------------------------------------------

impl Kind {
    fn link(&self, ctx: &RenderContext) -> String {
        match self {
            Kind::Lua(lk) => lk.link(ctx),
            Kind::Literal(k, s) => match k.as_ref() {
                LuaKind::String => format!("`\"{}\"`", s),
                LuaKind::Integer | LuaKind::Number => format!("`{}`", s.clone()),
                _ => s.clone(),
            },
            Kind::Class(class) => match class.scope {
                Scope::Local | Scope::Global => match ctx.options.order {
                    OutputOrder::ByFile => {
                        let file = class.file.clone().unwrap_or_default();
                        let file_stem = file
                            .file_stem()
                            .map(|v| v.to_string_lossy())
                            .unwrap_or("[unknown file]".into());

                        class_link(
                            &class.name,
                            // if the path prefix is the namespace here then the link will expect
                            // a folder for the namespace, but this is not how the per-file
                            // test is structured: some_class.md has acme namespace but not inside acme folder
                            // same is true for enums
                            &(ctx.url_root.to_string() + &class.scope.path_prefix("") + &file_stem),
                            &class.name,
                        )
                    }
                    OutputOrder::ByClass => {
                        if class.scope == Scope::Local {
                            local_class_link(&class.name, &class.name)
                        } else {
                            file_link(
                                &class.name,
                                &(ctx.url_root.to_string()
                                    + &class.scope.path_prefix(&ctx.options.namespace)
                                    + &class.name),
                            )
                        }
                    }
                },
                _ => file_link(
                    &class.name,
                    &(ctx.url_root.to_string()
                        + &class.scope.path_prefix(&ctx.options.namespace)
                        + &class.name),
                ),
            },
            Kind::Enum(kinds) => kinds
                .iter()
                .map(|k| k.link(ctx))
                .collect::<Vec<String>>()
                .join(" | "),
            Kind::EnumRef(enumref) => match ctx.options.order {
                OutputOrder::ByFile => {
                    let file = enumref.file.clone().unwrap_or(PathBuf::new());
                    let file_stem = file
                        .file_stem()
                        .map(|v| v.to_string_lossy())
                        .unwrap_or("[unknown file]".into());
                    enum_link(
                        &enumref.name,
                        &(ctx.url_root.to_string() + &Scope::Global.path_prefix("") + &file_stem),
                        &enumref.name,
                    )
                }
                OutputOrder::ByClass => enum_link(
                    &enumref.name,
                    Class::get_base(&enumref.name).unwrap_or(&enumref.name),
                    Class::get_end(&enumref.name).unwrap_or_default(),
                ),
            },
            Kind::SelfArg => format!("[*self*]({}API/builtins/self.md)", ctx.url_root),
            Kind::Array(k) => format!(
                "{}{}",
                k.link(ctx),
                file_link("[]", &format!("{}API/builtins/array", ctx.url_root))
            ),
            Kind::Nullable(k) => format!(
                "{}{}",
                k.as_ref().link(ctx),
                file_link("?", &format!("{}API/builtins/nil", ctx.url_root))
            ),
            Kind::Alias(alias) => alias_link(&alias.name, &alias.name),
            Kind::Function(f) => f.short(ctx, NameFormat::Omit, NameFormat::Plain),
            Kind::Table(k, v) => format!(
                "{}`<`{}, {}`>`",
                file_link("table", &format!("{}API/builtins/table", ctx.url_root)),
                k.as_ref().link(ctx),
                v.as_ref().link(ctx)
            ),
            Kind::Object(hm) => {
                let mut keys = hm.keys().cloned().collect::<Vec<String>>();
                keys.sort();
                let fields = keys
                    .iter()
                    .map(|k| format!("{} : {}", k, hm.get(k).unwrap().link(ctx)))
                    .collect::<Vec<String>>()
                    .join(", "); // TODO print on newlines?
                format!("{{ {} }}", fields)
            }
            Kind::Variadic(k) => format!("...{}", k.link(ctx)),
            Kind::Unresolved(s) => s.clone(),
            Kind::Generic(s, parent_type) => {
                let generic_link = file_link(s, &format!("{}/API/builtins/generic", ctx.url_root));
                if let Some(parent_type) = parent_type {
                    format!("{}:{}", generic_link, parent_type.link(ctx))
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
    fn short(&self, ctx: &RenderContext, name_format: NameFormat) -> String {
        let kind = self.kind.link(ctx);

        if matches!(self.kind, Kind::SelfArg) {
            kind
        } else if let Some(name) = self.name.clone() {
            match name_format {
                NameFormat::Plain => format!("{} : {}", name, kind),
                NameFormat::Link => format!("{} : {}", header_link(ctx, &name), kind),
                NameFormat::Omit => kind,
            }
        } else {
            kind
        }
    }

    fn long(&self, ctx: &RenderContext) -> String {
        let desc = self.desc.clone().unwrap_or_default();
        format!(
            "{}{}",
            hash(
                &h3(&self.short(ctx, NameFormat::Plain)),
                &ctx.id(&self.name.clone().unwrap_or_default())
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
    fn render(&self, ctx: &RenderContext) -> String {
        format!(
            "{}\n{}  \n{}",
            hash(&h3(&self.name), &ctx.id(&self.name)),
            self.kind.link(ctx),
            self.desc
                .clone()
                .map(|d| description(d.as_str()))
                .unwrap_or_default()
        )
    }
}

// -------------------------------------------------------------------------------------------------

impl Function {
    fn long(&self, ctx: &RenderContext) -> String {
        let name = self.name.clone().unwrap_or("fun".to_string());
        if self.params.is_empty() {
            let name = hash(&h3(&format!("`{}()`", name)), &ctx.id(&name));
            self.with_desc(&self.with_returns(&name, ctx, NameFormat::Plain))
        } else {
            let params = self
                .params
                .iter()
                .map(|v| v.short(ctx, NameFormat::Plain))
                .collect::<Vec<String>>()
                .join(", ");

            self.with_desc(&self.with_returns(
                &hash(&format!("### {}({})", name, params), &ctx.id(&name)),
                ctx,
                NameFormat::Plain,
            ))
        }
    }
    fn short(
        &self,
        ctx: &RenderContext,
        name_format: NameFormat,
        arg_format: NameFormat,
    ) -> String {
        let name = self
            .name
            .clone()
            .map(|n| match name_format {
                NameFormat::Plain => n,
                NameFormat::Link => header_link(ctx, &n),
                NameFormat::Omit => String::default(),
            })
            .unwrap_or_default();
        let params = Self::render_vars(&self.params, ctx, arg_format);
        let returns = Self::render_vars(&self.returns, ctx, arg_format);

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
    fn render_vars(vars: &[Var], ctx: &RenderContext, name_format: NameFormat) -> String {
        vars.iter()
            .map(|v| v.short(ctx, name_format))
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
    fn with_returns(&self, head: &str, ctx: &RenderContext, arg_format: NameFormat) -> String {
        let returns = self
            .returns
            .iter()
            .map(|v| v.short(ctx, arg_format))
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
        inner: bool,
    ) -> Vec<String> {
        let mut toc = TocTree::new();

        let ctx = RenderContext {
            url_root,
            options,
            parent: if inner { Some(self.name.clone()) } else { None },
        };

        if !self.enums.is_empty() || !self.constants.is_empty() {
            toc.item(header_link(&ctx, Self::CONSTANTS));
            toc.push();
            toc.list(&self.constants, |v| v.short(&ctx, NameFormat::Link));
            toc.list(&self.enums, |e| {
                let name = e.name.clone();
                let end = Class::get_end(&name).unwrap_or(&name);
                header_link(&ctx, end)
            });
            toc.pop();
        };

        if !self.fields.is_empty() {
            toc.section(header_link(&ctx, Self::PROPERTIES), &self.fields, |v| {
                v.short(&ctx, NameFormat::Link)
            });
        };

        if !self.functions.is_empty() {
            toc.section(header_link(&ctx, Self::FUNCTIONS), &self.functions, |f| {
                f.short(&ctx, NameFormat::Link, NameFormat::Omit)
            });
        };

        let (resolved_structs, resolved_aliases) = self.resolve(structs, aliases, options);

        if !resolved_structs.is_empty() {
            toc.item(header_link(&ctx, Self::STRUCTS));
            toc.push();
            resolved_structs.iter().for_each(|c| {
                let inner_toc = c.toc(url_root, structs, aliases, options, true);
                toc.item(header_link(&ctx, &c.name));
                toc.inner(&inner_toc);
            });
            toc.pop();
        };

        if !resolved_aliases.is_empty() {
            toc.item(header_link(&ctx, Self::ALIASES));
            toc.push();
            resolved_aliases.iter().for_each(|c| {
                toc.item(header_link(&ctx, &c.name));
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
        inner: bool,
    ) -> Vec<String> {
        let ctx = RenderContext {
            url_root,
            options,
            parent: if inner { Some(self.name.clone()) } else { None },
        };

        let mut body = vec![];

        if !self.enums.is_empty() || !self.constants.is_empty() {
            body.push(divider());
            body.push(hash(&h2(Self::CONSTANTS), &ctx.id(Self::CONSTANTS)));
            body.extend(self.enums.iter().map(|e| {
                let name = e.name.clone();
                let end = Class::get_end(&name).unwrap_or(&name);
                format!("{}\n{}", hash(&h3(end), &ctx.id(end)), description(&e.desc))
            }));
            body.extend(self.constants.iter().map(|v| v.long(&ctx)));
        };

        if !self.fields.is_empty() {
            body.push(divider());
            body.push(hash(&h2(Self::PROPERTIES), &ctx.id(Self::PROPERTIES)));
            body.extend(self.fields.iter().map(|v| v.long(&ctx)));
        };

        if !self.functions.is_empty() {
            body.push(divider());
            body.push(hash(&h2(Self::FUNCTIONS), &ctx.id(Self::FUNCTIONS)));
            body.extend(self.functions.iter().map(|f| f.long(&ctx)));
        };

        let (resolved_structs, resolved_aliases) = self.resolve(structs, aliases, options);

        if !resolved_structs.is_empty() {
            body.push(divider());
            body.push(hash(&h1(Self::STRUCTS), &ctx.id(Self::STRUCTS)));
            for s in resolved_structs.iter() {
                body.push({
                    let render_toc = false;
                    s.render(url_root, render_toc, structs, aliases, options, true)
                })
            }
        };

        if !resolved_aliases.is_empty() {
            body.push(divider());
            body.push(hash(&h1(Self::ALIASES), &ctx.id(Self::ALIASES)));
            for a in resolved_aliases.iter() {
                body.push(divider());
                body.push(a.render(&ctx))
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
        inner: bool,
    ) -> String {
        let mut page = vec![];

        page.extend(self.header());

        if render_toc {
            page.extend(self.toc(url_root, structs, aliases, options, inner))
        }

        page.extend(self.body(url_root, structs, aliases, options, inner));

        page.join("\n")
    }
}
