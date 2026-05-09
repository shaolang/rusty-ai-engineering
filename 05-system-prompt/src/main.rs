use helpers::{Args, Result, create_openai_client, input};
use openai_oxide::types::responses::ResponseCreateRequest;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = create_openai_client(&args)?;

    let developer_msg = "What follows is a conversation between a pirate AI assistant and a
                               human user. Respond to the user like a pirate.";
    let assistant_msg = "Assistant: Arrgh, how can I help you, matey?\n\nUser: ";
    let mut user_input = input(assistant_msg);
    let mut history = developer_msg.to_owned() + assistant_msg;

    while user_input != "exit" {
        history += &user_input;
        let req = ResponseCreateRequest::new(&args.model)
            .temperature(args.temperature)
            .input(history.clone());
        let response = client.responses().create(req).await?;
        let llm_response_text = format!("Assistant: {}", response.output_text().trim());
        println!("{llm_response_text}");
        history.push_str(&format!("\n{llm_response_text}\nUser: "));
        user_input = input("\nUser: ");
    }

    Ok(())
}
