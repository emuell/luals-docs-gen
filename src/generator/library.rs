use std::{collections::HashMap, path::Path};

use itertools::Itertools;

use crate::{
    error::Error,
    generator::options::{Options, OutputOrder},
    parser::{json::JsonDoc, types::*},
};

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct Library {
    pub classes: HashMap<String, Class>,
    pub enums: HashMap<String, Enum>,
    pub aliases: HashMap<String, Alias>,
}

impl Library {
    /// generate a library from a given root directory or lua file with the given options
    pub fn from_path(path: &Path, options: &Options) -> Result<Self, Error> {
        println!("Parsing definitions: '{}'", path.to_string_lossy());
        let mut defs: Vec<Def> = vec![];
        let definitions = JsonDoc::get(path)?;
        defs.append(
            &mut definitions
                .iter()
                .filter_map(|d| Def::from_definition(d, &options.namespace))
                .collect::<Vec<Def>>(),
        );
        Ok(Self::from_defs(defs, options))
    }

    // a list of classes that correspond to lua types
    pub fn builtin_classes() -> Vec<Class> {
        let self_example = "```lua\nlocal object = SomeClass()\nobject:do_something(123)\n```";
        vec![
            Self::builtin_class_desc(
                "self",
                &format!("A type that represents an instance that you call a function on. When you see a function signature starting with this type, you should use `:` to call the function on the instance, this way you can omit this first argument.\n{}", self_example),
            ),
            Self::builtin_class_desc(
                "nil",
                "A built-in type representing a non-existant value, [see details](https://www.lua.org/pil/2.1.html). When you see `?` at the end of types, it means they can be nil.",
            ),
            Self::builtin_class_desc(
                "boolean",
                "A built-in type representing a boolean (true or false) value, [see details](https://www.lua.org/pil/2.2.html)",
            ),
            Self::builtin_class_desc(
                "number",
                "A built-in type representing floating point numbers, [see details](https://www.lua.org/pil/2.3.html)",
            ),
            Self::builtin_class_desc(
                "string",
                "A built-in type representing a string of characters, [see details](https://www.lua.org/pil/2.4.html)",
            ),
            Self::builtin_class_desc("function", "A built-in type representing functions, [see details](https://www.lua.org/pil/2.6.html)"),
            Self::builtin_class_desc("table", "A built-in type representing associative arrays, [see details](https://www.lua.org/pil/2.5.html)"),
            Self::builtin_class_desc("userdata", "A built-in type representing array values, [see details](https://www.lua.org/pil/28.1.html)."),
            Self::builtin_class_desc(
                "lightuserdata",
                "A built-in type representing a pointer, [see details](https://www.lua.org/pil/28.5.html)",
            ),

            Self::builtin_class_desc("integer", "A helper type that represents whole numbers, a subset of [number](number.md)"),
            Self::builtin_class_desc(
                "any",
                "A type for a dynamic argument, it can be anything at run-time.",
            ),
            Self::builtin_class_desc(
                "unknown",
                "A dummy type for something that cannot be inferred before run-time.",
            ),
        ]
    }

    fn resolve_string(&self, s: &str) -> Option<Kind> {
        #[allow(clippy::manual_map)]
        if let Some(class) = self.classes.get(s) {
            Some(Kind::Class(class.clone()))
        } else if let Some(alias) = self.aliases.get(s) {
            Some(Kind::Alias(Box::new(alias.clone())))
        } else if let Some(enumref) = self.enums.get(s) {
            Some(Kind::EnumRef(Box::new(enumref.clone())))
        } else {
            None
        }
    }

    // cross-reference parsed Kinds as existing classes, enums and aliases
    fn resolve_kind(&self, kind: &Kind) -> Kind {
        match kind.clone() {
            Kind::Unresolved(s) => self.resolve_string(&s).unwrap_or(kind.clone()),
            Kind::Array(bk) => Kind::Array(Box::new(self.resolve_kind(bk.as_ref()))),
            Kind::Nullable(bk) => Kind::Nullable(Box::new(self.resolve_kind(bk.as_ref()))),
            Kind::Table(key, value) => Kind::Table(
                Box::new(self.resolve_kind(key.as_ref())),
                Box::new(self.resolve_kind(value.as_ref())),
            ),
            Kind::Enum(kinds) => Kind::Enum(kinds.iter().map(|k| self.resolve_kind(k)).collect()),
            Kind::Function(f) => {
                let mut fun = f.clone();
                self.resolve_function(&mut fun);
                Kind::Function(fun)
            }
            Kind::Variadic(v) => Kind::Variadic(Box::new(self.resolve_kind(v.as_ref()))),
            Kind::Object(hm) => {
                let mut obj = hm.clone();
                for (key, value) in hm.iter() {
                    obj.insert(key.clone(), Box::new(self.resolve_kind(value.as_ref())));
                }
                Kind::Object(obj)
            }
            _ => kind.clone(),
        }
    }

    fn resolve_function(&self, f: &mut Function) {
        for p in f.params.iter_mut() {
            p.kind = self.resolve_kind(&p.kind)
        }
        for r in f.returns.iter_mut() {
            r.kind = self.resolve_kind(&r.kind)
        }
    }

    fn resolve_classes(&mut self) {
        let l = self.clone();
        for (_, c) in self.classes.iter_mut() {
            for f in c.fields.iter_mut() {
                f.kind = l.resolve_kind(&f.kind)
            }
            for f in c.functions.iter_mut() {
                l.resolve_function(f)
            }
        }
    }

    // helper to create built-in dummy classes
    fn builtin_class_desc(name: &str, desc: &str) -> Class {
        Class {
            file: None,
            line_number: None,
            scope: Scope::Builtins,
            name: name.to_string(),
            desc: desc.to_string(),
            fields: vec![],
            functions: vec![],
            constants: vec![],
            enums: vec![],
        }
    }

    // generate Library from a list of Defs, applying the given options.
    fn from_defs(defs: Vec<Def>, options: &Options) -> Self {
        // sort defs into hasmaps of classes, enums and aliases
        let mut classes = HashMap::new();
        let mut enums = HashMap::new();
        let mut aliases = HashMap::new();
        let mut dangling_functions = vec![];
        for d in defs.iter() {
            match d {
                Def::Alias(a) => {
                    aliases.insert(a.name.clone(), a.clone());
                }
                Def::Enum(e) => {
                    enums.insert(e.name.clone(), e.clone());
                }
                Def::Class(c) => {
                    classes.insert(c.name.clone(), c.clone());
                }
                Def::Function(f) => dangling_functions.push(f.clone()),
            }
        }

        // collect library contents
        let mut library = Self {
            classes,
            enums,
            aliases,
        };

        // transform any unresolved Kind to the appropriate class or alias
        // by cross referencing the hashmaps of the library
        library.resolve_classes();
        let mut aliases = library.aliases.clone();
        aliases
            .iter_mut()
            .for_each(|(_, a)| a.kind = library.resolve_kind(&a.kind));
        library.aliases = aliases;
        dangling_functions
            .iter_mut()
            .for_each(|f| library.resolve_function(f));

        // assign enums to new or existing classes
        for (k, e) in library.enums.iter() {
            let base = Class::get_base(k).unwrap_or("global").to_string();
            if let Some(class) = library.classes.get_mut(&base) {
                class.enums.push(e.clone())
            } else {
                let mut target_class_name = base.clone();
                let mut added_to_existing_class = false;
                match options.order {
                    OutputOrder::ByFile => {
                        if let Some(file) = &e.file {
                            let file_stem = file.file_stem().map(|f| f.to_string_lossy());
                            target_class_name =
                                format!("{} globals", file_stem.unwrap()).to_string();
                        }
                        // move globals into a separate "globals" file
                        if let Some((_, class)) = library
                            .classes
                            .iter_mut()
                            .find(|(name, c)| *name == &target_class_name && c.file == e.file)
                        {
                            let f = e.strip_base();
                            if !class.enums.iter().any(|f2| f2.name == f.name) {
                                class.enums.push(e.clone())
                            }
                            added_to_existing_class = true;
                        }
                    }
                    OutputOrder::ByClass => {
                        // move globals into the source file
                        if let Some((_, class)) =
                            library.classes.iter_mut().find(|(name, _c)| *name == &base)
                        {
                            let e = e.strip_base();
                            if !class.enums.iter().any(|e2| e2.name == e.name) {
                                class.enums.push(e)
                            }
                            added_to_existing_class = true;
                        }
                    }
                }
                if !added_to_existing_class {
                    library.classes.insert(
                        target_class_name,
                        Class {
                            file: e.file.clone(),
                            line_number: e.line_number,
                            scope: Scope::from_name(&base, &options.namespace),
                            name: base,
                            functions: vec![],
                            fields: vec![],
                            enums: vec![e.strip_base()],
                            constants: vec![],
                            desc: String::new(),
                        },
                    );
                }
            }
        }

        // assign global functions to new or existing classes
        for f in dangling_functions.iter_mut() {
            let function_name = f.name.clone().unwrap_or_default();
            let class_name = Class::get_base(&function_name)
                .unwrap_or("global")
                .to_string();
            let mut target_class_name = class_name.clone();
            let mut added_to_existing_class = false;
            match options.order {
                OutputOrder::ByFile => {
                    if target_class_name == "global" {
                        if let Some(file) = &f.file {
                            let file_stem = file.file_stem().map(|f| f.to_string_lossy());
                            target_class_name =
                                format!("{} globals", file_stem.unwrap()).to_string();
                        }
                    }
                    // move globals into a separate "globals" file
                    if let Some((_, class)) = library
                        .classes
                        .iter_mut()
                        .find(|(name, c)| *name == &target_class_name && c.file == f.file)
                    {
                        let f = f.strip_base();
                        if !class.functions.iter().any(|f2| f2.name == f.name) {
                            class.functions.push(f)
                        }
                        added_to_existing_class = true;
                    }
                }
                OutputOrder::ByClass => {
                    // move globals into the source file
                    if let Some((_, class)) = library
                        .classes
                        .iter_mut()
                        .find(|(name, _c)| *name == &class_name)
                    {
                        let f = f.strip_base();
                        if !class.functions.iter().any(|f2| f2.name == f.name) {
                            class.functions.push(f)
                        }
                        added_to_existing_class = true;
                    }
                }
            }
            if !added_to_existing_class {
                library.classes.insert(
                    target_class_name,
                    Class {
                        file: f.file.clone(),
                        line_number: f.line_number,
                        scope: Scope::from_name(&class_name, &options.namespace),
                        name: class_name,
                        functions: vec![f.strip_base()],
                        fields: vec![],
                        enums: vec![],
                        constants: vec![],
                        desc: String::new(),
                    },
                );
            }
        }

        // apply class excludes
        library
            .classes
            .retain(|name, _| !options.excluded_classes.contains(name));

        // extract constants, make functions, fields and constants unique and sort them
        for class in library.classes.values_mut() {
            let mut functions = class
                .functions
                .clone()
                .into_iter()
                .unique_by(|f| f.to_string())
                .collect::<Vec<_>>();
            functions.sort_by_key(|f| (f.file.clone(), f.line_number));

            let mut enums = class
                .enums
                .clone()
                .into_iter()
                .unique_by(|e| e.name.clone())
                .collect::<Vec<_>>();
            enums.sort_by_key(|e| (e.file.clone(), e.line_number));

            let mut fields = class
                .fields
                .clone()
                .into_iter()
                .filter(Var::is_not_constant)
                // HACK: remove table properties from classes, assuming they are nested classes
                .filter(|v| !matches!(v.kind, Kind::Lua(LuaKind::Table)))
                .unique_by(|f| f.name.clone())
                .collect::<Vec<_>>();
            fields.sort_by_key(|f| (f.file.clone(), f.line_number));

            let mut constants = class
                .fields
                .clone()
                .into_iter()
                .filter(Var::is_constant)
                .unique_by(|f| f.name.clone())
                .collect::<Vec<_>>();
            constants.sort_by_key(|c| (c.file.clone(), c.line_number));

            class.functions = functions;
            class.fields = fields;
            class.enums = enums;
            class.constants = constants;
        }

        // debug print everything that includes some unresolved Kind or is empty
        if !library.classes.is_empty() {
            println!("classes:");
            for class in library.classes.values() {
                let is_empty = library
                    .classes
                    .get(&class.name)
                    .is_some_and(|v| v.is_empty());
                let unresolved = class.has_unresolved();

                if is_empty || unresolved {
                    println!("  {}", class.name);
                }
                if unresolved {
                    println!("{}\n", class.show());
                }
                if is_empty {
                    println!("  \x1b[33m^--- has no fields, methods or enums\x1b[0m")
                }
            }
        }
        if library
            .aliases
            .iter()
            .any(|(__, a)| a.kind.has_unresolved())
        {
            println!("aliases:");
            for alias in library.aliases.values() {
                if alias.kind.has_unresolved() {
                    println!("  {}", alias.name);
                    println!("\n{}\n", alias.show());
                }
            }
        }
        library
    }
}
