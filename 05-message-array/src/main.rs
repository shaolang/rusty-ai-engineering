use helpers::{Args, History, Result, create_openai_client, extract_texts, input};
use openai_oxide::types::responses::ResponseCreateRequest;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = create_openai_client(&args)?;

    let mut history = History::new("You are an AI assistant who always talks like a pirate".into());

    let assistant_msg = "Arrgh, how can I help you, matey?";
    history.add_assistant_msg(assistant_msg);

    println!("Assistant: {assistant_msg}");
    let mut user_input = input("\nUser: ");

    while &user_input != "exit" {
        history.add_user_msg(user_input);
        let req = ResponseCreateRequest::new(&args.model)
            .temperature(args.temperature)
            .input(&history);
        let response = client.responses().create(req).await?;
        let llm_output_text = extract_texts(&response.output, true);
        println!("Assistant: {llm_output_text}");
        history.add_assistant_msg(llm_output_text);

        user_input = input("\nUser: ");
    }

    Ok(())
}
