use std::{cell::RefCell, io::Write};

use argh::FromArgs;
use async_openai::error::OpenAIError;
use async_openai::types::responses::{
    InputContent, InputItem, InputMessage, InputParam, InputRole, InputTextContent, Item,
    MessageItem, OutputItem, OutputMessage, OutputMessageContent, OutputTextContent, Response,
};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};
pub use termcolor::Color::{Cyan, Green, Red};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn cprint(color: Color, text: &str) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut color_spec = ColorSpec::new();
    stdout
        .set_color(color_spec.set_fg(Some(color)))
        .expect("set color succeeds");
    write!(&mut stdout, "{text}").expect("write w/ color to stdout succeeds");
    color_spec.clear();
    stdout.set_color(&color_spec).expect("reset color succeeds");
    stdout.flush().unwrap();
}

pub fn cprintln(color: Color, text: &str) {
    cprint(color, &format!("{text}\n"));
}

pub fn get_output(
    response: Result<Response, OpenAIError>,
) -> Result<(String, Vec<OutputItem>), OpenAIError> {
    match response {
        Ok(resp) => {
            let text = resp.output_text().unwrap_or("".to_string());
            Ok((text, resp.output))
        }
        Err(e) => match e.try_extract_output() {
            Ok((text, output)) => Ok((text, vec![output])),
            Err(err) => {
                cprintln(Red, &format!("Error occurred: {:?}", err));
                Err(err)
            }
        },
    }
}

pub fn get_output_text(response: Result<Response, OpenAIError>) -> Result<String, OpenAIError> {
    get_output(response).map(|(text, _)| text)
}

pub fn parse_args() -> Args {
    let args: Args = argh::from_env();

    if args.temperature < 0.0 || args.temperature > 2.0 {
        cprintln(Red, "Temperature must be between 0.0 and 2.0");
        std::process::exit(1);
    }

    args
}

pub fn read_stdin(prompt: Option<String>) -> String {
    if let Some(text) = prompt {
        print!("{text}");
        std::io::stdout().flush().expect("flush output succeeds");
    }

    let mut s = String::new();
    std::io::stdin()
        .read_line(&mut s)
        .expect("read stdin succeeds");

    s.trim().to_string()
}

pub trait OpenAIErrorExt {
    fn try_extract_output(self) -> Result<(String, OutputItem), OpenAIError>;
    fn try_extract_response(self) -> Result<Response, OpenAIError>;
}

#[derive(FromArgs)]
/// CLI to run examples from A Common Sense Guide to AI Engineering
pub struct Args {
    /// base url to connect to
    #[argh(option, default = "String::from(\"https://api.openai.com/v1\")")]
    pub base_url: String,

    /// controls "creativity" in output; must be between 0.0 and 2.0 (both inclusive) where
    /// lower is less "creativity"
    #[argh(option, default = "0.0")]
    pub temperature: f32,

    /// llm model to use
    #[argh(option)]
    pub model: String,
}

impl OpenAIErrorExt for OpenAIError {
    fn try_extract_output(self) -> Result<(String, OutputItem), OpenAIError> {
        let resp = self.try_extract_response()?;
        let output = resp.output.last().expect("output exists");
        if let async_openai::types::responses::OutputItem::Message(msg) = output {
            let content = msg.content.last().expect("content exists");
            if let async_openai::types::responses::OutputMessageContent::OutputText(text) = content
            {
                Ok((text.text.clone(), output.to_owned()))
            } else {
                Ok(("".to_string(), output.to_owned()))
            }
        } else {
            Ok(("".to_string(), output.to_owned()))
        }
    }

    fn try_extract_response(self) -> Result<Response, OpenAIError> {
        let OpenAIError::JSONDeserialize(err, raw) = self else {
            return Err(self);
        };
        let Ok(content) = serde_json::from_str::<ContentOnly>(&raw) else {
            return Err(OpenAIError::JSONDeserialize(err, raw));
        };
        Ok(content.into())
    }
}

#[derive(Debug, Deserialize)]
struct ContentOnly {
    id: String,
    output: Vec<Output>,
}

#[derive(Debug, Deserialize)]
struct Output {
    id: String,
    #[serde(default)]
    content: Vec<Content>,
}

#[derive(Debug, Deserialize)]
struct Content {
    text: String,
}

#[derive(Default, Debug)]
pub struct History {
    items: RefCell<Vec<InputItem>>,
}

#[derive(Serialize)]
struct HistoryItem {
    role: String,
    content: String,
}

impl From<ContentOnly> for Response {
    fn from(value: ContentOnly) -> Self {
        Self {
            id: value.id,
            output: value
                .output
                .into_iter()
                .filter(|o| !o.content.is_empty())
                .map(|o| async_openai::types::responses::OutputItem::Message(o.into()))
                .collect(),
            background: Default::default(),
            billing: Default::default(),
            conversation: Default::default(),
            created_at: Default::default(),
            error: Default::default(),
            incomplete_details: Default::default(),
            instructions: Default::default(),
            max_output_tokens: Default::default(),
            metadata: Default::default(),
            model: Default::default(),
            object: Default::default(),
            parallel_tool_calls: Default::default(),
            previous_response_id: Default::default(),
            prompt: Default::default(),
            prompt_cache_key: Default::default(),
            prompt_cache_retention: Default::default(),
            reasoning: Default::default(),
            safety_identifier: Default::default(),
            service_tier: Default::default(),
            status: async_openai::types::responses::Status::Completed,
            temperature: Default::default(),
            text: Default::default(),
            tool_choice: Default::default(),
            tools: Default::default(),
            top_logprobs: Default::default(),
            top_p: Default::default(),
            truncation: Default::default(),
            usage: Default::default(),
        }
    }
}

impl From<Output> for async_openai::types::responses::OutputMessage {
    fn from(output: Output) -> Self {
        Self {
            id: output.id.to_owned(),
            role: async_openai::types::responses::AssistantRole::Assistant,
            status: async_openai::types::responses::OutputStatus::Completed,
            content: output.content.into_iter().map(|o| o.into()).collect(),
        }
    }
}

impl From<Content> for async_openai::types::responses::OutputMessageContent {
    fn from(content: Content) -> Self {
        let text_content = async_openai::types::responses::OutputTextContent {
            annotations: vec![],
            logprobs: None,
            text: content.text.to_string(),
        };
        async_openai::types::responses::OutputMessageContent::OutputText(text_content)
    }
}

impl History {
    pub fn new() -> Self {
        Self {
            items: RefCell::new(vec![]),
        }
    }

    pub fn add_user_input(&self, text: &str) {
        self.add_input(InputRole::User, text);
    }

    pub fn add_system_input(&self, text: &str) {
        self.add_input(InputRole::System, text);
    }

    pub fn add_assistant_outputs(&self, outputs: &[OutputItem]) {
        let mut items: Vec<InputItem> = outputs
            .iter()
            .filter_map(|m| {
                if let OutputItem::Message(msg) = m {
                    Some(InputItem::Item(Item::Message(MessageItem::Output(
                        msg.to_owned(),
                    ))))
                } else {
                    None
                }
            })
            .collect();
        let mut history = self.items.borrow_mut();
        history.append(&mut items);
    }

    fn add_input(&self, role: InputRole, text: &str) {
        let mut items = self.items.borrow_mut();
        let input_msg = InputMessage {
            content: vec![text.into()],
            role,
            status: None,
        };
        items.push(InputItem::Item(Item::Message(MessageItem::Input(
            input_msg,
        ))));
    }

    pub fn as_input_params(&self) -> InputParam {
        let items = self.items.borrow().clone();
        InputParam::Items(items)
    }

    pub fn clear(&self) {
        let mut items = self.items.borrow_mut();
        items.clear();
    }

    pub fn replace_first_system_prompt(&self, text: &str) {
        self.add_system_input(text);
        let mut items = self.items.borrow_mut();
        items.swap_remove(0);
    }

    pub fn remove_last(&self) {
        let mut items = self.items.borrow_mut();
        items.pop().unwrap();
    }
}

impl Serialize for History {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.items.borrow().len()))?;
        for e in self.items.borrow().iter() {
            let e: Result<HistoryItem, _> = e.try_into();
            if let Ok(e) = e {
                seq.serialize_element(&e)?;
            }
        }

        serde::ser::SerializeSeq::end(seq)
    }
}

impl TryFrom<&InputItem> for HistoryItem {
    type Error = ();

    fn try_from(item: &InputItem) -> Result<Self, Self::Error> {
        match item {
            InputItem::Item(Item::Message(MessageItem::Input(InputMessage {
                content,
                role,
                ..
            }))) => {
                if let InputContent::InputText(InputTextContent { text }) = content.first().unwrap()
                {
                    let role = match role {
                        InputRole::User => "user",
                        InputRole::Developer => "developer",
                        InputRole::System => "system",
                    }
                    .to_string();
                    Ok(HistoryItem {
                        role,
                        content: text.to_string(),
                    })
                } else {
                    Err(())
                }
            }
            InputItem::Item(Item::Message(MessageItem::Output(OutputMessage {
                content, ..
            }))) => {
                if let OutputMessageContent::OutputText(OutputTextContent { text, .. }) =
                    content.first().unwrap()
                {
                    Ok(HistoryItem {
                        role: "assistant".to_string(),
                        content: text.to_string(),
                    })
                } else {
                    Err(())
                }
            }
            _ => Err(()),
        }
    }
}
