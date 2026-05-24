use eval_chatbot::*;
use helpers::{Args, Result, input};

use reduce_hallucinations::{SystemPrompt, remove_bracket_tags};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let (client, db, mut history) = setup(&args, "target/eval-chatbot.db").await?;
    let mut system_prompt = SystemPrompt::new();
    history.replace_system_prompt(system_prompt.prepare(vec![]));

    let mut assistant_msg = "How can I help you today?".to_string();
    let mut user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));

    while &user_input != "exit" {
        history.add_assistant_msg(assistant_msg);
        let recs = db.search::<Record>(&user_input, 3).await?;
        history.add_user_msg(user_input);
        history.replace_system_prompt(system_prompt.prepare(recs));
        assistant_msg = remove_bracket_tags(llm_response(&args, &client, &history).await);

        user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));
    }

    Ok(())
}
