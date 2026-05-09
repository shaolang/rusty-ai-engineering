use argh::FromArgs;
use openai_oxide::{
    ClientConfig, OpenAI, error::OpenAIError, types::responses::ResponseCreateRequest,
};

#[tokio::main]
async fn main() -> Result<(), OpenAIError> {
    let args: Args = argh::from_env();
    let client = create_client(&args.base_url)?;
    let user_input = input("Ahoy! Got questions? Spit 'em out, ye scallywag!");
    let req = ResponseCreateRequest::new(&args.model)
        .temperature(args.temperature)
        .input(format!(
            "Respond to the following like a pirate: {user_input}"
        ));
    let response = client.responses().create(req).await?;
    println!("{}", response.output_text());

    Ok(())
}

fn create_client(base_url: &str) -> Result<OpenAI, OpenAIError> {
    let config = ClientConfig::from_env()?.base_url(base_url);

    Ok(OpenAI::with_config(config))
}

fn input(prompt: &str) -> String {
    use std::io::Read;

    println!("{prompt}");
    let mut s = String::new();
    std::io::stdin().read_line(&mut s).expect("read from stdin");
    s.trim().to_string()
}

#[derive(Clone, FromArgs)]
/// Rusty AI Engineering Demos
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
