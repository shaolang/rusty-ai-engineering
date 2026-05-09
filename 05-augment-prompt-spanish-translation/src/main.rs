use helpers::{Args, Result, create_openai_client, input};
use openai_oxide::types::responses::ResponseCreateRequest;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = create_openai_client(&args)?;
    let user_input = input("Enter a phrase, and I'll translate it into Spanish!\n");
    let req = ResponseCreateRequest::new(&args.model)
        .temperature(args.temperature)
        .input(format!(
            "Translate the following into Spanish: {user_input}.\n\n
             Answer with just the translation and no other introductory text."
        ));
    let response = client.responses().create(req).await?;
    println!("{}", response.output_text());

    Ok(())
}
