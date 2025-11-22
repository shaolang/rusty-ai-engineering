use async_openai::{
    Client,
    config::OpenAIConfig,
    error::OpenAIError,
    types::responses::{CreateResponse, Input},
};
use clap::Parser;

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse_args();
    let client = args.create_client();

    while let Some(user_input) = get_user_input() {
        let req = create_response_requuest(&args, &user_input);
        match client.responses().create(req).await {
            Ok(content) => println!("{}", content.output_text.expect("output text exists")),
            Err(OpenAIError::JSONDeserialize(e, content)) => {
                log::error!("Json deserializing error: {e}");
                log::error!("Attempting to extract output from json");
                println!("{}", extract_last_output(&content));
            }
            Err(e) => log::error!("Error occurred: {e}"),
        }
    }

    println!("Bye");
}

fn get_user_input() -> Option<String> {
    loop {
        println!("Enter a phrase, and I'll translate it into Spanish!");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("read stdio successfully");
        input = input.trim().to_string();

        if !input.is_empty() {
            return if input.to_lowercase() == "quit" {
                None
            } else {
                Some(input)
            };
        }
    }
}

fn create_response_requuest(args: &Args, user_input: &str) -> CreateResponse {
    let input = format!("Translate the following phrase into Spanish: {user_input}");

    CreateResponse {
        input: Input::Text(input),
        model: args.model.to_owned(),
        max_output_tokens: args.max_tokens.clone(),
        temperature: args.temperature.clone(),
        ..Default::default()
    }
}

fn extract_last_output(content: &str) -> String {
    let v = serde_json::from_str::<serde_json::Value>(content).unwrap();
    let output = v["output"].as_array().expect("output array");

    output.last().unwrap()["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string()
}

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    model: String,

    #[arg(short, long, default_value = "https://api.openai.com/v1")]
    api_base: String,

    #[arg(short, long)]
    temperature: Option<f32>,

    #[arg(short, long)]
    max_tokens: Option<u32>,
}

impl Args {
    fn parse_args() -> Self {
        let args = Self::parse();
        args.validate();
        args
    }

    fn validate(&self) {
        let Some(temperature) = self.temperature else {
            return;
        };

        if temperature < 0.0 || temperature > 2.0 {
            use clap::CommandFactory;
            use clap::error::ErrorKind;

            let mut cmd = Args::command();
            cmd.error(
                ErrorKind::InvalidValue,
                "Temperature must be between 0.0 and 2.0",
            )
            .exit();
        }
    }

    fn create_client(&self) -> Client<OpenAIConfig> {
        let config = OpenAIConfig::default().with_api_base(&self.api_base);
        Client::with_config(config)
    }
}
