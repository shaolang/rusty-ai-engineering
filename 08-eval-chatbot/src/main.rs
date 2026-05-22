use eval_chatbot::*;
use helpers::{Args, Result, input};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let (client, db, mut history) = setup(&args, "target/eval-chatbot.db").await?;
    let mut assistant_msg = "How can I help you today?".to_string();
    let mut user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));

    while &user_input != "exit" {
        history.add_assistant_msg(assistant_msg);
        let recs = db.search::<Record>(&user_input, 3).await?;
        history.add_user_msg(user_prompt(&user_input, recs));
        assistant_msg = llm_response(&args, &client, &history).await;

        user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));
    }

    Ok(())
}
