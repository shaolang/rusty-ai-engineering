/// Rusty AI Engineering Demos in Rust
#[derive(Clone, argh::FromArgs)]
pub struct Args {
    /// model to use
    #[argh(option)]
    pub model: String,

    /// url to connect to; defaults to https://api.openai.com/v1
    #[argh(option, default = "String::from(\"https://api.openai.com/v1\")")]
    pub(crate) base_url: String,

    /// temperature; defaults to zero
    #[argh(option, default = "0.0")]
    pub temperature: f64,
}

impl Args {
    pub fn parse() -> Self {
        argh::from_env()
    }
}
