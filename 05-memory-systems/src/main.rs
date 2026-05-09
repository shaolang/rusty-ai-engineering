use helpers::{Args, Result, create_openai_client, input};
use openai_oxide::types::responses::ResponseCreateRequest;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = create_openai_client(&args)?;
    let mut history = String::new();
    let assistant_message = "Assistant: How can I help you today?";
    history.push_str(&assistant_message);

    let mut user_input = input(&format!("{assistant_message}\nUser: "));
    while &user_input != "exit" {
        history.push_str(&format!("\nUser: {user_input}"));
        let req = ResponseCreateRequest::new(&args.model)
            .temperature(args.temperature)
            .input(history.clone());
        let response = client.responses().create(req).await?;
        let output_text = format!("\nAssistant: {}\n", response.output_text().trim());
        print!("{output_text}");
        history.push_str(&output_text);

        user_input = input("\nUser: ");
    }

    Ok(())
}
