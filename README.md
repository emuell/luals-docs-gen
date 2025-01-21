# LuaLS API Documentation Generator

This is a rust appplication and library that generates generate markdown docs from [Lua LS](https://github.com/LuaLS/lua-language-server) type annotated API definitions. The generated markdown and folder structure is tuned to be consumed by a [mdbook](https://github.com/rust-lang/mdBook).

This crate was created and is used to automatically generate Lua API docs in the [Renoise](https://renoise.github.io/xrnx/) and [afseq](https://emuell.github.io/afseq/) Lua APIs.

## Acknowledgements

It is based on matt-allan's [mdbook-luacat](https://github.com/matt-allan/mdbook-luacats) tool.

[unless](https://github.com/unlessgames) created the first working initial version of the generator and the pest Lua parser.  

## Usage

### Command Line App

To create or update API definitions, simply build and run the application.

```bash
# List all options
cargo run -- --help

# Build and update the example mdbook in this crate
cargo run -- ./test/definitions ./test/src
```

### Library

In your Cargo.toml:

```toml
[dependencies]
luals-docs-gen = { git = "https://github.com/emuell/luals-docs-gen" }
```

In your rust app:

```rust
use luals_docs_gen::*;

fn main() -> Result<(), Error> {
    // when using relative paths
    std::env::set_current_dir(env!("CARGO_MANIFEST_DIR"))?;
    // run generator with given options
    let options = Options {
        library: "../../some/path_to/definitions".into(),
        output: "../some/path_to/out".into(),
        excluded_classes: vec!["HideThisClass".to_string()],
        order: OutputOrder::ByClass,
        namespace: "acme".into(),
    };
    generate_docs(&options)
}
```

## Building 

### Requirements

[rust](https://www.rust-lang.org/tools/install) v1.78 or higher.

NOTE: The first time the binary (or generator as library) is run, a copy of the Lua language server is downloaded and patched to build the documentation.  Subsequent runs will reuse the existing Lua LS binaries. 

Unfortunately, patching the LuaLS installation is necessary to change the configuration to make it useful as a document generator instead of a language server. See [applied patches](./src/parser/json.rs#126).

## Debugging

If you have vscode installed, run the `Debug: Build API` action.

To debug and build the full API definition, change the launch arguments in the file [.vscode/launch.json](../.vscode/launch.json).


## Licence

This project is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
