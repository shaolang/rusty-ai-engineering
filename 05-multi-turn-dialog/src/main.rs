use helpers::{Args, Result, create_openai_client, input};
use openai_oxide::types::responses::ResponseCreateRequest;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = create_openai_client(&args)?;
    let mut user_input = input("Assistant: How can I help you today?\n\nUser: ");

    while &user_input != "exit" {
        let req = ResponseCreateRequest::new(&args.model)
            .temperature(args.temperature)
            .input(user_input);
        let response = client.responses().create(req).await?;
        println!("{}\n\n", response.output_text());

        user_input = input("User: ");
    }

    Ok(())
}
