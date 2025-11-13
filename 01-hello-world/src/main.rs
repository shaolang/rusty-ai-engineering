use clap::Parser;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    args.validate();

    chat(&args).await;
}

// open ai
async fn chat(args: &Args) {
    use async_openai::{Client, config::OpenAIConfig, types::CreateCompletionRequestArgs};

    let config = OpenAIConfig::default().with_api_base(args.api_base.to_string());

    let client = Client::with_config(config);
    let request = CreateCompletionRequestArgs::default()
        .model(&args.model)
        .prompt(&args.construct_prompt())
        .temperature(args.temperature)
        .max_tokens(args.max_tokens)
        .build()
        .expect("valid creation request");
    let response = client.completions().create(request).await.unwrap();

    println!("{}", response.choices.first().unwrap().text);
}

// cli

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    model: String,

    /// Valid range: >= 0.0 and <= 2.0
    #[arg(short, long, default_value_t = 0.0)]
    temperature: f32,

    #[arg(long, default_value_t = 500)]
    max_tokens: u32,

    #[arg(short, long, default_value = "https://api.openai.com")]
    api_base: String,

    input: Vec<String>,
}

impl Args {
    fn validate(&self) {
        if self.temperature < 0.0 || self.temperature > 2.0 {
            use clap::CommandFactory;
            use clap::error::ErrorKind;

            let mut cmd = Args::command();
            cmd.error(
                ErrorKind::ValueValidation,
                "Temperature must be between 0.0 and 2.0",
            )
            .exit();
        }
    }

    fn construct_prompt(&self) -> String {
        self.input.join(" ")
    }
}
