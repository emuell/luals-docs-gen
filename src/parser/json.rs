use std::{
    fmt, fs,
    path::{absolute, Path, PathBuf},
    process::Command,
    str::FromStr,
};

use serde::{Deserialize, Deserializer, Serialize};
use tempdir::TempDir;
use url::Url;

use crate::{error::Error, parser::types::LuaKind};

// -------------------------------------------------------------------------------------------------

pub struct JsonDoc {}

impl JsonDoc {
    /// Generate a list of definitions from a file
    pub fn get(path: &Path) -> Result<Vec<Definition>, Error> {
        let defs = Self::export(path)?;
        Ok(Self::strip(path, defs))
    }

    /// Export and parse the JSON docs from lua-language-server
    fn export(path: &Path) -> Result<Vec<Definition>, Error> {
        let tmp_dir = TempDir::new("docs")?;
        let tmp_path = tmp_dir.path();
        let ls_filename = if cfg!(windows) {
            "lua-language-server.exe"
        } else {
            "lua-language-server"
        };
        // allow running from within the "generate" path and from the repository root path
        let ls_path = PathBuf::from("./lua-language-server/bin/").join(ls_filename);
        if !ls_path.exists() {
            Self::download_luals()?;
        }
        if !ls_path.exists() {
            return Err(Error::Exec(format!(
                "lua-language-server binary does not exist: '{}'",
                absolute(ls_path.clone())
                    .unwrap_or(ls_path)
                    .to_string_lossy()
            )));
        }
        let output = Command::new(ls_path)
            .arg("--doc")
            .arg(path)
            .arg("--doc_out_path")
            .arg(tmp_path)
            .arg("--logpath")
            .arg(tmp_path)
            .output()?;

        if !output.status.success() {
            Err(Error::Exec(
                String::from_utf8(output.stderr).unwrap_or("unknown error".to_string()),
            ))
        } else {
            let json_doc_path = tmp_dir.path().join("doc.json");
            let json_doc = fs::read_to_string(json_doc_path)?;
            Ok(serde_json::from_str(&json_doc)?)
        }
    }

    fn download_luals() -> Result<(), Error> {
        use decompress::{decompress, ExtractOptsBuilder};
        use reqwest::{
            blocking::Client,
            header::{HeaderMap, USER_AGENT},
        };

        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
                .parse()
                .unwrap(),
        );

        // download
        const LUA_LS_VERSION: &str = "3.9.3";
        const LUA_LS_RELEASES_URL: &str =
            "https://github.com/LuaLS/lua-language-server/releases/download";

        #[cfg(target_os = "linux")]
        const LUA_LS_BINARY: &str = "linux-x64.tar.gz";
        #[cfg(target_os = "windows")]
        const LUA_LS_BINARY: &str = "win32-x64.zip";
        #[cfg(target_os = "macos")]
        const LUA_LS_BINARY: &str = "darwin-arm64.tar.gz";

        let luals_url = format!("{LUA_LS_RELEASES_URL}/{LUA_LS_VERSION}/lua-language-server-{LUA_LS_VERSION}-{LUA_LS_BINARY}");
        println!("Downloading lua-ls from {}...", luals_url);

        let http = Client::builder().default_headers(headers).build()?;
        let request = http.get(luals_url).send()?;
        let response = request.error_for_status()?;
        let content = response.bytes()?;

        let tmp_dir = TempDir::new("docs")?;
        let tmp_file = tmp_dir.path().join(LUA_LS_BINARY);

        fs::write(tmp_file.clone(), content).expect("Unable to write file");

        // decompress
        let dest_dir = PathBuf::from("./lua-language-server");
        fs::create_dir(dest_dir.clone())?;

        println!(
            "Extracting lua-ls to {}...",
            dest_dir.as_path().to_string_lossy()
        );
        decompress(
            tmp_file.clone(),
            dest_dir.clone(),
            &ExtractOptsBuilder::default()
                .build()
                .expect("failed to build default decompression options"),
        )?;

        fs::remove_file(tmp_file)?;

        // patch
        Self::patch_file(
            // raise enum display limits
            &dest_dir.clone().join("script/config/template.lua"),
            r#"(\['Lua.hover.enumsLimit'\]\s*=\s*Type.Integer\s*>>\s*)5(,)"#,
            r#"${1}100${2}"#,
        )?;
        Self::patch_file(
            // don't resolve aliases - we do
            &dest_dir.clone().join("script/config/template.lua"),
            r#"(\['Lua.hover.expandAlias'\]\s*=\s*Type.Boolean\s*>>\s*)true(,)"#,
            r#"${1}false${2}"#,
        )?;
        Self::patch_file(
            // avoid truncating output in general
            &dest_dir.clone().join("script/vm/infer.lua"),
            r#"if \#view > 200 then"#,
            r#"if false then -- no limit"#,
        )?;

        Ok(())
    }

    fn patch_file(path: &Path, search: &str, replace: &str) -> Result<(), Error> {
        println!("Patching: `{}`", path.to_string_lossy());
        let content = fs::read_to_string(path)?;

        let re = regex::Regex::new(search)?;
        let result = re.replace_all(&content, replace);
        if content != result {
            fs::write(path, result.as_bytes())?;
            Ok(())
        } else {
            Err(Error::Patch(regex::Error::Syntax(
                "patch does not apply".to_string(),
            )))
        }
    }

    fn file_url_matches(file_url: &str, base_path: &Path) -> bool {
        assert!(
            Url::from_str(file_url).is_ok(),
            "expecting an url file string"
        );
        let file_path = Url::from_str(file_url)
            .unwrap()
            .to_file_path()
            .unwrap_or_default()
            .canonicalize()
            .unwrap_or_default();
        let base_path = base_path.canonicalize().unwrap_or_default();
        file_path.starts_with(base_path)
    }

    /// Exclude standard lua
    fn strip(path: &Path, defs: Vec<Definition>) -> Vec<Definition> {
        defs.into_iter()
            .map(|d| {
                // remove standard define from the list of defines (for type())
                let mut def = d.clone();
                def.defines
                    .retain(|define| Self::file_url_matches(&define.file, path));
                def
            })
            .collect()
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq, Serialize, Clone, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VisibleType {
    Public,
    Private,
    Package,
    Protected,
}

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, PartialEq, Deserialize, Clone, Eq, PartialOrd, Ord)]
pub enum Doc {
    #[serde(rename = "doc.field")]
    Field,
    #[serde(rename = "doc.enum")]
    Enum,
    #[serde(rename = "doc.alias")]
    Alias,
    #[serde(rename = "doc.class")]
    Class,
    #[serde(rename = "doc.extends.name")]
    ExtendsName,
    #[serde(rename = "doc.type")]
    Type,
    #[serde(rename = "doc.type.name")]
    TypeName,
    #[serde(rename = "doc.type.integer")]
    TypeInteger,
    #[serde(rename = "doc.type.number")]
    TypeNumber,
    #[serde(rename = "doc.type.array")]
    TypeArray,
    #[serde(rename = "doc.type.boolean")]
    TypeBoolean,
    #[serde(rename = "doc.type.string")]
    TypeString,
    #[serde(rename = "doc.type.table")]
    TypeTable,
    #[serde(rename = "doc.type.sign")]
    TypeSign,
    #[serde(rename = "doc.type.function")]
    TypeFunction,
}

// -------------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq, Serialize, Clone, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Doc(Doc),
    // fields, defines
    GetField,
    SetField,
    SetMethod,
    // definitions
    #[serde(rename = "type")]
    Def,
    Variable,
    #[serde(rename = "luals.config")]
    LuaLsConfig,
    // defines
    TableField,
    SetGlobal,
    // function returns
    #[serde(rename = "function.return")]
    FunctionReturn,
    // extends
    Lua(LuaKind),
}

impl<'de> Deserialize<'de> for Type {
    fn deserialize<D>(deserializer: D) -> Result<Type, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        match s {
            "type" => Ok(Type::Def),
            "variable" => Ok(Type::Variable),
            "getfield" => Ok(Type::GetField),
            "setfield" => Ok(Type::SetField),
            "setmethod" => Ok(Type::SetMethod),
            "setglobal" => Ok(Type::SetGlobal),
            "tablefield" => Ok(Type::TableField),
            "function.return" => Ok(Type::FunctionReturn),
            "luals.config" => Ok(Type::LuaLsConfig),
            _ => {
                let quoted = format!("\"{}\"", s);
                match serde_json::from_str::<LuaKind>(quoted.as_str()) {
                    Ok(lk) => Ok(Type::Lua(lk)),
                    Err(_) => match serde_json::from_str::<Doc>(quoted.as_str()) {
                        Ok(d) => Ok(Type::Doc(d)),
                        Err(_) => Err(serde::de::Error::unknown_variant(s, &[])),
                    },
                }
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Field {
    pub desc: Option<String>,
    pub rawdesc: Option<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub lua_type: Type,
    pub file: String,
    pub start: u32,
    pub finish: u32,
    pub visible: VisibleType,
    #[serde(default, deserialize_with = "deserialize_extends")]
    pub extends: Option<Extend>,
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} : {:?}",
            self.name,
            self.lua_type,
            // self.extends.clone().map(|e| e.view).unwrap_or_default()
        )
    }
}

impl Field {
    pub fn is_field(&self) -> bool {
        self.lua_type == Type::Doc(Doc::Field)
            || (self.lua_type == Type::SetField
                && !Self::extend_has_type(&self.extends, Type::Lua(LuaKind::Function)))
    }

    pub fn is_function(&self) -> bool {
        self.lua_type == Type::SetMethod
            || (self.lua_type == Type::SetField
                && Self::extend_has_type(&self.extends, Type::Lua(LuaKind::Function)))
    }

    fn extend_has_type(e: &Option<Extend>, t: Type) -> bool {
        if let Some(e) = e {
            e.lua_type == t
        } else {
            false
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Definition {
    #[serde(rename = "type")]
    pub lua_type: Type,
    pub name: String,
    pub desc: Option<String>,
    pub rawdesc: Option<String>,
    pub defines: Vec<Define>,
    #[serde(default)]
    pub fields: Vec<Field>,
}

impl fmt::Display for Definition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} : {:?} d[{}] f[{}]{}{}",
            self.name,
            self.lua_type,
            self.defines.len(),
            self.fields.len(),
            // "",
            if !self.defines.is_empty() {
                format!(
                    "\n{}",
                    self.defines
                        .iter()
                        .enumerate()
                        .map(|(i, d)| format!("    <{}> - {}", i, d))
                        .collect::<Vec<String>>()
                        .join("\n")
                )
            } else {
                String::from("")
            },
            if !self.fields.is_empty() {
                format!(
                    "\n{}",
                    self.fields
                        .clone()
                        .iter()
                        .enumerate()
                        .map(|(i, f)| format!("    [{}] - {}", i, f))
                        .collect::<Vec<String>>()
                        .join("\n")
                )
            } else {
                String::from("")
            },
        )
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Define {
    #[serde(rename = "type")]
    pub lua_type: Type,
    pub file: String,
    pub start: u32,
    pub finish: u32,
    #[serde(default, deserialize_with = "deserialize_extends")]
    pub extends: Option<Extend>,
}

impl fmt::Display for Define {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.lua_type,)
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ExtendType {
    #[serde(rename = "type")]
    pub lua_type: Type,
    pub start: u32,
    pub finish: u32,
    pub view: String,
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Extend {
    #[serde(rename = "type")]
    pub lua_type: Type,
    pub types: Option<Vec<ExtendType>>,
    pub start: u32,
    pub finish: u32,
    pub view: String,
    pub desc: Option<String>,
    pub rawdesc: Option<String>,
    /// Only present for functions (type = "function") with args
    #[serde(default)]
    pub args: Vec<ArgDef>,
    /// Only present for functions (type = "function") with returns
    #[serde(default)]
    pub returns: Vec<ReturnDef>,
}

// -------------------------------------------------------------------------------------------------

/// Extends can be either null, an object or an array of objects
fn deserialize_extends<'de, D>(deserializer: D) -> Result<Option<Extend>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize, Debug)]
    #[serde(untagged)]
    enum ExtendInput {
        None,
        Map(Extend),
        Array(Vec<Extend>),
        Object(Extend),
        Other(serde_json::Value),
    }

    impl From<ExtendInput> for Option<Extend> {
        fn from(input: ExtendInput) -> Self {
            match input {
                ExtendInput::None => None,
                ExtendInput::Map(extend) => Some(extend),
                // there is only one possible extends per define in the API
                ExtendInput::Array(vec) => vec.first().cloned(),
                ExtendInput::Object(extend) => Some(extend),
                #[allow(unused_variables)]
                ExtendInput::Other(value) => {
                    #[cfg(debug_assertions)]
                    panic!("Unexpected extend {:?}", value);
                    #[cfg(not(debug_assertions))]
                    None
                }
            }
        }
    }
    Ok(ExtendInput::deserialize(deserializer)?.into())
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ArgType {
    #[serde(rename = "self")]
    SelfArg,
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "...")]
    Variadic,
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ArgDef {
    #[serde(rename = "type")]
    pub lua_type: ArgType,
    pub name: Option<String>,
    pub view: String,
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ReturnDef {
    // NB: type for returns will always be "function.return"
    #[serde(rename = "type")]
    pub lua_type: Type,
    pub name: Option<String>,
    pub view: String,
}
