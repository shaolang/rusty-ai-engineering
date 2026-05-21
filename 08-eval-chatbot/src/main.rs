use eval_chatbot::*;
use helpers::{Result, extract_texts, input};
use openai_oxide::types::responses::ResponseCreateRequest;

#[tokio::main]
async fn main() -> Result<()> {
    let (args, client, db, mut history) = setup("target/eval-chatbot.db").await?;
    let mut assistant_msg = "How can I help you today?".to_string();
    let mut user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));

    while &user_input != "exit" {
        history.add_assistant_msg(assistant_msg);
        let recs = db.search::<Record>(&user_input, 3).await?;
        history.add_user_msg(user_prompt(&user_input, recs));

        let response = client
            .responses()
            .create(
                ResponseCreateRequest::new(&args.model)
                    .temperature(args.temperature)
                    .input(&history),
            )
            .await
            .expect("response request sent");
        assistant_msg = extract_texts(&response.output, true);

        user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));
    }

    Ok(())
}
