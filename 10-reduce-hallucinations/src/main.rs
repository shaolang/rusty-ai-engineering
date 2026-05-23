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
        assistant_msg = remove_bracket_tags(llm_response(&args, &client, &history).await);

        user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));
    }

    Ok(())
}

fn remove_bracket_tags(text: String) -> String {
    let regex = fancy_regex::Regex::new(r#"\[\[.*?\]\]\s*(\r?\n)?"#).expect("valid regex");
    regex.replace_all(&text, "").to_string()
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
        let result = format!(include_str!("system-prompt.txt"), documentation);

        for rec in records.into_iter() {
            self.chunks.insert(rec.id.clone(), rec);
        }

        result.into()
    }
}
