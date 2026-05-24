use std::collections::HashMap;

use eval_chatbot::Record;
use serde_json::Value;

pub fn remove_bracket_tags(text: String) -> String {
    let regex = fancy_regex::Regex::new(r#"\[\[.*?\]\]\s*(\r?\n)?"#).expect("valid regex");
    regex.replace_all(&text, "").to_string()
}

pub struct SystemPrompt {
    chunks: HashMap<String, Record>,
}

impl SystemPrompt {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::<String, Record>::new(),
        }
    }

    pub fn prepare(&mut self, records: Vec<Record>) -> Value {
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
