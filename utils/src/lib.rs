use std::io::Write;

use argh::FromArgs;
use async_openai::{error::OpenAIError, types::responses::Response};
use serde::Deserialize;
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

pub fn get_output_text_from_response(response: Result<Response, OpenAIError>) -> String {
    match response {
        Ok(resp) => resp.output_text().unwrap_or("".to_string()),
        Err(e) => {
            if let Ok(text) = e.try_extract_output() {
                text
            } else {
                cprintln(Red, &format!("Error occurred: {e:?}"));
                "".to_string()
            }
        }
    }
}

pub fn parse_args() -> Args {
    let args: Args = argh::from_env();

    if args.temperature < 0.0 || args.temperature > 2.0 {
        cprintln(Red, "Temperature must be between 0.0 and 2.0");
        std::process::exit(1);
    }

    args
}

pub fn read_stdin(prompt: Option<String>) -> String {
    if let Some(text) = prompt {
        print!("{text}");
        std::io::stdout().flush().expect("flush output succeeds");
    }

    let mut s = String::new();
    std::io::stdin()
        .read_line(&mut s)
        .expect("read stdin succeeds");

    s.trim().to_string()
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
        let OpenAIError::JSONDeserialize(_, content) = self else {
            return Err(self);
        };
        let Ok(content) = serde_json::from_str::<ContentOnly>(content) else {
            return Err(self);
        };
        if let Some(Some(text)) = content
            .output
            .last()
            .map(|o| o.content.last().map(|c| c.text.to_string()))
        {
            Ok(text)
        } else {
            Err(self)
        }
    }
}

#[derive(Debug, Deserialize)]
struct ContentOnly {
    output: Vec<Output>,
}

#[derive(Debug, Deserialize)]
struct Output {
    content: Vec<Content>,
}

#[derive(Debug, Deserialize)]
struct Content {
    text: String,
}
