use std::io::Write;

use async_openai::{Client, config::OpenAIConfig, error::OpenAIError, types::responses::Response};
use clap::{Parser, ValueEnum};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use tracing::Level;
pub use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;

pub fn extract_content_from_json(s: &str) -> String {
    let v = serde_json::from_str::<serde_json::Value>(s).expect("json content");
    let content = v["output"].as_array().expect("content as array");

    content.last().unwrap()["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string()
}

pub fn get_user_input(input_prompt: &str, quit_phrase: &str, print_prompt: bool) -> Option<String> {
    loop {
        if print_prompt {
            print!("\n{input_prompt}\nEnter {quit_phrase} to end> ");
        } else {
            print!("Enter {quit_phrase} to end> ");
        }

        std::io::stdout().flush().expect("successful stdout flush");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("successful stdin read");
        input = input.trim().to_string();

        if !input.is_empty() {
            return if input.to_lowercase() == quit_phrase {
                None
            } else {
                Some(input)
            };
        }
    }
}

pub fn print_llm_output(s: &str) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut color_spec = ColorSpec::new();
    stdout
        .set_color(color_spec.set_fg(Some(Color::Green)))
        .expect("set term color");
    writeln!(&mut stdout, "{}", s).expect("successful stdout write");
    color_spec.clear();
    stdout.set_color(&color_spec).expect("clear set color");
}

pub trait ResponseExt<T> {
    fn extract_output(self) -> String;
}

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long)]
    pub model: String,

    #[arg(short, long, default_value = "https://api.openai.com/v1")]
    api_base: String,

    #[arg(short, long)]
    pub temperature: Option<f32>,

    #[arg(short, long)]
    pub max_tokens: Option<u32>,

    #[arg(short, long, default_value = "info")]
    log_level: LogLevel,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum LogLevel {
    Info,
    Error,
}

impl Args {
    pub fn parse_args() -> Self {
        let args = Self::parse();
        args.validate();

        let log_level = match args.log_level {
            LogLevel::Info => Level::INFO,
            LogLevel::Error => Level::ERROR,
        };
        let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();
        tracing::subscriber::set_global_default(subscriber).expect("subscriber set succeeds");

        args
    }

    pub fn create_client(&self) -> Client<OpenAIConfig> {
        let config = OpenAIConfig::default().with_api_base(self.api_base.to_owned());
        Client::with_config(config)
    }

    fn validate(&self) {
        let Some(temperature) = self.temperature else {
            return;
        };
        if temperature < 0.0 || temperature > 2.0 {
            use clap::CommandFactory;
            use clap::error::ErrorKind;

            let mut cmd = Self::command();
            cmd.error(
                ErrorKind::InvalidValue,
                "temperature must be between 0.0 and 2.0",
            )
            .exit();
        }
    }
}

impl ResponseExt<Result<Response, OpenAIError>> for Result<Response, OpenAIError> {
    fn extract_output(self) -> String {
        error!("{:?}", self);
        match self {
            Ok(resp) => resp.output_text.unwrap(),
            Err(OpenAIError::JSONDeserialize(e, content)) => {
                error!("Response JSON deserializing failed; attempting only text extraction: {e}");
                extract_content_from_json(&content)
            }
            Err(e) => {
                error!("error occurred: {e}");
                "<an error occurred>".to_string()
            }
        }
    }
}
