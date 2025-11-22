use std::io::Write;

use async_openai::{config::OpenAIConfig, error::OpenAIError, types::responses::{CreateResponse, Input, Response}, Client, Responses};
use clap::Parser;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    args.validate();

    let client = args.create_client();
    let responses = Responses::new(&client);
    let mut buffer = String::with_capacity(500);

    loop {
        prompt_user_input(&mut buffer);

        match buffer.trim().to_lowercase().as_str() {
            "" => println!("Please enter your query, or \"quit\" to end the session."),
            "quit" => break,
            _ => {
                let req = CreateResponse {
                    input: Input::Text(buffer.clone()),
                    model: args.model.clone(),
                    max_output_tokens: Some(args.max_tokens),
                    temperature: Some(args.temperature),
                    ..Default::default()
                };
                let output_text = try_extract_content(responses.create(req).await);

                println!("{}\n\n", output_text);
            }
        }
    }

    println!("Bye!");
}

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    model: String,

    #[arg(short, long, default_value="https://api.openai.com/v1")]
    api_base: String,

    #[arg(long, default_value_t=500)]
    max_tokens: u32,

    /// Valid range: >= 0.0 and <= 2.0
    #[arg(short, long, default_value_t=0.0)]
    temperature: f32,
}

impl Args {
    fn validate(&self) {
        if self.temperature < 0.0 || self.temperature > 2.0 {
            use clap::CommandFactory;
            use clap::error::ErrorKind;

            let mut cmd = Args::command();
            cmd.error(ErrorKind::InvalidValue, "Temperature must be between 0.0 and 2.0").exit();
        }
    }

    fn create_client(&self) -> Client<OpenAIConfig> {
        let config = OpenAIConfig::default().with_api_base(&self.api_base);
        Client::with_config(config)
    }
}

fn try_extract_content(result: Result<Response, OpenAIError>) -> String {
    match result {
        Ok(resp) => resp.output_text.expect("valid output text"),
        Err(OpenAIError::JSONDeserialize(_, string)) => {
            println!("{}", string);
            let resp: serde_json::Value = serde_json::from_str(&string).expect("json response");
            resp["output"].as_array().unwrap().last().unwrap()["text"].as_str().unwrap().to_string()
        }
        Err(_) => "<an error occurred>".to_string(),
    }
}

fn prompt_user_input(buffer: &mut String) {
    println!("I'm the Quickstart Guide chatbot! Ask me anything (Enter \"quit\" to end the session):");
    print!("> ");
    std::io::stdout().flush().expect("flush stdout");
    buffer.clear();
    std::io::stdin().read_line(buffer).expect("stdin read success");
}
