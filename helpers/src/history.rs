use openai_oxide::types::{
    chat::Role,
    responses::{ResponseInput, ResponseInputItem},
};
use serde_json::Value;

#[derive(Clone, Debug, serde::Serialize)]
pub struct History {
    pub messages: Vec<ResponseInputItem>,
}

macro_rules! add_msg {
    ($name:ident,$role:expr) => {
        pub fn $name(&mut self, msg: impl Into<Value>) {
            self.add_msg($role, msg.into());
        }
    };
}

impl History {
    pub fn new(system_prompt: Value) -> Self {
        let input_item = ResponseInputItem {
            role: Role::Developer,
            content: system_prompt,
        };
        let messages = vec![input_item];
        Self { messages }
    }

    add_msg!(add_assistant_msg, Role::Assistant);
    add_msg!(add_user_msg, Role::User);

    fn add_msg(&mut self, role: Role, content: Value) {
        let input_item = ResponseInputItem { role, content };
        self.messages.push(input_item);
    }

    pub fn exclude_system_prompt(&self) -> History {
        let mut cloned = self.clone();
        cloned.messages.remove(0);
        cloned
    }

    pub fn replace_system_prompt(&mut self, content: Value) {
        let input_item = ResponseInputItem {
            role: Role::Developer,
            content,
        };
        self.messages.remove(0);
        self.messages.insert(0, input_item);
    }
}

impl From<&History> for ResponseInput {
    fn from(history: &History) -> Self {
        Self::Messages(history.messages.clone())
    }
}
