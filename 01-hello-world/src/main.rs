use argh::FromArgs;
use openai_oxide::{ClientConfig, OpenAI, types::responses::ResponseCreateRequest};

#[tokio::main]
async fn main() -> Result<(), openai_oxide::OpenAIError> {
    let args: Args = argh::from_env();
    let client = new_client(&args)?;
    let response = client
        .responses()
        .create(
            ResponseCreateRequest::new(&args.model)
                .temperature(args.temperature)
                .input("Who was the first person to land on the Moon?"),
        )
        .await?;

    println!("{}", response.output_text());
    Ok(())
}

fn new_client(args: &Args) -> Result<OpenAI, openai_oxide::OpenAIError> {
    let config = ClientConfig::from_env()?.base_url(&args.base_url);
    Ok(OpenAI::with_config(config))
}

#[derive(Clone, FromArgs)]
/// "Hello, World" demo from chapter 1
struct Args {
    /// model to use
    #[argh(option)]
    model: String,

    /// url to connect to; defaults to https://api.openai.com/v1
    #[argh(option, default = "String::from(\"https://api.openai.com/v1\")")]
    base_url: String,

    /// temperature
    #[argh(option, default = "0.0")]
    temperature: f64,
}
