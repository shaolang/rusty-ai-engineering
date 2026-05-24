use eval_chatbot::*;
use helpers::{Args, History, Result, input};

use openai_oxide::{
    OpenAI,
    types::{chat::Role, responses::ResponseCreateRequest},
};
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
        history.add_user_msg(user_input);
        let expanded_query = expand_query(&args, &client, &history).await;
        let recs = db.search::<Record>(&expanded_query, 3).await?;
        history.replace_system_prompt(system_prompt.prepare(recs));
        assistant_msg = remove_bracket_tags(llm_response(&args, &client, &history).await);

        user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));
    }

    Ok(())
}

async fn expand_query(args: &Args, client: &OpenAI, history: &History) -> String {
    let model = args
        .secondary_model
        .as_deref()
        .unwrap_or_else(|| args.model.as_ref());
    let convo = Conversation(history.exclude_system_prompt()).to_string();
    let input = format!(
        "Rewritee, in an expanded way, what the user means to say in their final prompt
         of the following conversation: {convo}"
    );

    let resp = client
        .responses()
        .create(
            ResponseCreateRequest::new(model)
                .temperature(0.0)
                .input(input),
        )
        .await;
    resp.expect("query expanded").output_text()
}

struct Conversation(History);

impl ToString for Conversation {
    fn to_string(&self) -> String {
        self.0
            .messages
            .iter()
            .map(|rec| {
                let role = match rec.role {
                    Role::Assistant => "Assistant",
                    Role::User => "User",
                    _ => "Unknown",
                };
                format!("{}: {}", role, rec.content.to_string())
            })
            .collect::<Vec<String>>()
            .join(" ")
    }
}
