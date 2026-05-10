use openai_oxide::{
    OpenAI, OpenAIError,
    types::{
        beta::Role,
        responses::{ResponseInput, ResponseInputItem, ResponseOutputContent, ResponseOutputItem},
    },
};
use serde_json::Value;

use crate::Args;

pub fn create_openai_client(args: &Args) -> Result<OpenAI, OpenAIError> {
    let config = openai_oxide::ClientConfig::from_env()?.base_url(&args.base_url);
    Ok(OpenAI::with_config(config))
}

pub fn extract_texts(output: &[ResponseOutputItem], last_only: bool) -> String {
    let texts: Vec<String> = output.iter().map(|o| o.extract_texts()).collect::<Vec<_>>();

    if last_only {
        texts.last().expect("text responses exist").clone()
    } else {
        texts.join("\n")
    }
}

#[derive(Clone, Debug)]
pub struct History {
    messages: Vec<ResponseInputItem>,
}

macro_rules! add_msg {
    ($name:ident,$role:expr) => {
        pub fn $name(&mut self, msg: impl Into<Value>) {
            self.add_msg($role, msg.into());
        }
    };
}

impl History {
    pub fn new() -> Self {
        let messages = vec![];
        Self { messages }
    }

    add_msg!(add_developer_msg, Role::Developer);
    add_msg!(add_assistant_msg, Role::Assistant);
    add_msg!(add_user_msg, Role::User);

    fn add_msg(&mut self, role: Role, content: Value) {
        let input_item = ResponseInputItem { role, content };
        self.messages.push(input_item);
    }
}

impl From<&History> for ResponseInput {
    fn from(history: &History) -> Self {
        Self::Messages(history.messages.clone())
    }
}

trait ResponseOutputItemExt {
    fn extract_texts(&self) -> String;
}

impl ResponseOutputItemExt for ResponseOutputItem {
    fn extract_texts(&self) -> String {
        let ResponseOutputItem {
            content: Some(contents),
            ..
        } = self
        else {
            return "".to_string();
        };
        contents
            .iter()
            .filter_map(|c| {
                let ResponseOutputContent { text: Some(t), .. } = c else {
                    return None;
                };
                Some(t.as_str())
            })
            .collect::<Vec<&str>>()
            .join("\n")
    }
}
