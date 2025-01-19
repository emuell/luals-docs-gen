/// The errors that may happen when running the generator.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid options: `{0}`")]
    Options(String),

    #[error("file IO error")]
    Io(#[from] std::io::Error),

    #[error("download error")]
    Http(#[from] reqwest::Error),

    #[error("decompression error")]
    Decompress(#[from] decompress::DecompressError),

    #[error("patch error")]
    Patch(#[from] regex::Error),

    #[error("failed to execute lua-language-server: `{0}`")]
    Exec(String),

    #[error("unable to parse doc JSON")]
    JsonParse(#[from] serde_json::Error),
}
