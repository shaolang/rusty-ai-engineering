use helpers::{Args, Result, create_openai_client, input};
use openai_oxide::types::responses::ResponseCreateRequest;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = create_openai_client(&args)?;
    let user_input = input("Enter a text for grammatical check.\n");
    let req = ResponseCreateRequest::new(&args.model)
        .temperature(args.temperature)
        .input(format!(
            "Check the TEXT below for grammatical errors. If there are errors, simply
             rewrite the text so that it's correct. If there are no errors, just output
             the original text. Here is the TEXT: {user_input}"
        ));
    let response = client.responses().create(req).await?;
    println!("{}", response.output_text());

    Ok(())
}
