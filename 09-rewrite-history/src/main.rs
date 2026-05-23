use std::collections::HashMap;

use eval_chatbot::*;
use helpers::{Args, Result, input};
use serde_json::Value;

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
        assistant_msg = llm_response(&args, &client, &history).await;

        user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));
    }

    Ok(())
}

struct SystemPrompt {
    chunks: HashMap<String, Record>,
}

impl SystemPrompt {
    fn new() -> Self {
        Self {
            chunks: HashMap::<String, Record>::new(),
        }
    }

    fn prepare(&mut self, records: Vec<Record>) -> Value {
        let documentation = records
            .iter()
            .map(Record::to_string)
            .collect::<Vec<String>>()
            .join(" ");
        let result = format!(
            "You are an AI customer support technician who is knowledgeable about software products
             created by the company called GROSS. The products are:
             * Flamehamster, a web browser.
             * Rumblechirp, an email client.
             * GuineaPigment, a drawing tool for creating/editing SVGs.
             * EMRgency, an electronic medical record system.
             * Verbiage++, a content management system.

             You represent GROSS, and you are having a conversation with a human user
             who needs technical support with at least one of these GROSS products.

             You have access to certain excerpts of GROSS products' docmentation that is
             pulled from a RAG system. Use this info (and no other info) to advise the user.
             Here are the documentation excerpts: {documentation}

             When helping troubleshoot a user's issue, ask a proactive question to
             help determine what exactly the issue is. When asking proactive follow-up
             questions, ask exactly one question at a time."
        );

        for rec in records.into_iter() {
            self.chunks.insert(rec.id.clone(), rec);
        }

        result.into()
    }
}
