use helpers::{Args, Result, create_openai_client, input};
use openai_oxide::types::responses::ResponseCreateRequest;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = Args::parse();
    let client = create_openai_client(&args)?;
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
