use openai_oxide::types::{
    chat::Role,
    responses::{ResponseInput, ResponseInputItem},
};
use serde_json::Value;

#[derive(Clone, Debug, serde::Serialize)]
pub struct History {
    pub messages: Vec<Value>,
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
        let input_item: Value = serde_json::to_value(ResponseInputItem {
            role: Role::Developer,
            content: system_prompt,
        })
        .unwrap();
        let messages = vec![input_item];
        Self { messages }
    }

    add_msg!(add_assistant_msg, Role::Assistant);
    add_msg!(add_user_msg, Role::User);

    fn add_msg(&mut self, role: Role, content: Value) {
        let input_item = serde_json::to_value(ResponseInputItem { role, content }).unwrap();
        self.messages.push(input_item);
    }

    pub fn add_function_call_msg(&mut self, msg: impl Into<Value>) {
        self.messages.push(msg.into());
    }

    pub fn add_function_call_output(&mut self, call_id: &str, result: impl Into<Value>) {
        let input_item = serde_json::json!({
            "type": "function_call_output",
            "call_id": call_id,
            "output": result.into()
        });
        self.messages.push(input_item);
    }

    pub fn exclude_system_prompt(&self) -> History {
        let mut cloned = self.clone();
        cloned.messages.remove(0);
        cloned
    }

    pub fn replace_system_prompt(&mut self, content: Value) {
        let input_item = serde_json::to_value(ResponseInputItem {
            role: Role::Developer,
            content,
        })
        .unwrap();
        self.messages.remove(0);
        self.messages.insert(0, input_item);
    }
}

impl From<&History> for ResponseInput {
    fn from(history: &History) -> Self {
        Self::Items(history.messages.clone())
    }
}
