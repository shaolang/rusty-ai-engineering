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
        let input = Input::Text(format!(
            "Respond to the following like a pirate: {user_input}"
        ));

        let req = CreateResponse {
            input: input,
            max_output_tokens: Some(args.max_tokens),
            model: args.model.clone(),
            temperature: Some(args.temperature),
            ..Default::default()
        };
        match client.responses().create(req).await {
            Ok(resp) => {
                println!(
                    "{}",
                    resp.output_text.unwrap_or("no response?!".to_string())
                );
            }
            Err(OpenAIError::JSONDeserialize(e, content)) => {
                log::error!("An error occurred when deserializing content: {e}");
                log::error!("Deserializing just the content only");
                let v = serde_json::from_str::<serde_json::Value>(&content).unwrap();
                let output = v["output"].as_array().expect("an array output");
                let content = output.last().unwrap()["content"][0]["text"]
                    .as_str()
                    .unwrap();
                println!("{content}");
            }
            Err(e) => log::error!("Error occurred: {e}"),
        }
    }

    println!("Bye!");
}

fn get_user_input() -> Option<String> {
    loop {
        println!("Ahoy! Got questions? Spit 'em out, ye scallywag!");

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("successfully read from stdin");
        input = input.trim().to_string();

        if !input.is_empty() {
            return if &input.to_lowercase() == "quit" {
                None
            } else {
                Some(input)
            };
        }
    }
}

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    model: String,

    #[arg(short, long, default_value = "https://api.openai.com/v1")]
    api_base: String,

    #[arg(long, default_value_t = 500)]
    max_tokens: u32,

    /// valid range: >= 0.0 and <= 2.0
    #[arg(short, long, default_value_t = 0.0)]
    temperature: f32,
}

impl Args {
    fn parse_args() -> Self {
        let args = Self::parse();
        args.validate();
        args
    }

    fn validate(&self) {
        if self.temperature < 0.0 || self.temperature > 2.0 {
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
