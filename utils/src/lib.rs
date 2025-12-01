use std::io::Write;

use argh::FromArgs;
use async_openai::{error::OpenAIError};
pub use termcolor::Color::{Cyan, Green, Red};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn cprint(color: Color, text: &str) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut color_spec = ColorSpec::new();
    stdout
        .set_color(color_spec.set_fg(Some(color)))
        .expect("set color succeeds");
    writeln!(&mut stdout, "{text}").expect("write w/ color to stdout succeeds");
    color_spec.clear();
    stdout.set_color(&color_spec).expect("reset color succeeds");
}

pub fn cprintln(color: Color, text: &str) {
    cprint(color, &format!("{text}\n"));
}

pub fn parse_args() -> Args {
    let args: Args = argh::from_env();

    if args.temperature < 0.0 && args.temperature > 2.0 {
        cprintln(Red, "Temperature must be between 0.0 and 2.0");
        std::process::exit(1);
    }

    args
}

pub trait OpenAIErrorExt {
    fn try_extract_output(&self) -> Result<String, &OpenAIError>;
}

#[derive(FromArgs)]
/// CLI to run examples from A Common Sense Guide to AI Engineering
pub struct Args {
    /// base url to connect to
    #[argh(option, default = "String::from(\"https://api.openai.com/v1\")")]
    pub base_url: String,

    /// controls "creativity" in output; must be between 0.0 and 2.0 (both inclusive) where
    /// lower is less "creativity"
    #[argh(option, default = "0.0")]
    pub temperature: f32,

    /// llm model to use
    #[argh(option)]
    pub model: String,
}

impl OpenAIErrorExt for OpenAIError {
    fn try_extract_output(&self) -> Result<String, &OpenAIError> {
        if let OpenAIError::JSONDeserialize(_, content) = self {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(content) {
                let output = v["output"].as_array().expect("output array");
                let content = &output.last().expect("last item in output array").as_object().expect("content is a hashmap")["content"];
                content.as_str().map(|s| s.to_string()).ok_or(self)
            } else {
                Err(self)
            }
        } else {
            Err(self)
        }
    }
}
