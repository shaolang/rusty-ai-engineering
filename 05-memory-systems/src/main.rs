use helpers::{Args, History, Result, create_openai_client, input};
use openai_oxide::types::responses::ResponseCreateRequest;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = create_openai_client(&args)?;
    let mut history = History::new();
    let assistant_message = "Assistant: How can I help you today?\n\nUser: ";
    history.add_assistant_msg(assistant_message);

    let mut user_input = input(assistant_message);
    while &user_input != "exit" {
        history.add_user_msg(user_input);
        let req = ResponseCreateRequest::new(&args.model)
            .temperature(args.temperature)
            .input(history.clone());
        let response = client.responses().create(req).await?;
        let output_text = response.output_text();
        println!("Assistant: {output_text}");
        history.add_assistant_msg(output_text);

        user_input = input("\nUser: ");
    }

    Ok(())
}
