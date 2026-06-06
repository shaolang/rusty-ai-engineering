use openai_oxide::{
    OpenAI, OpenAIError,
    types::responses::{ResponseOutputContent, ResponseOutputItem},
};

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
                Some(t.trim())
            })
            .collect::<Vec<&str>>()
            .join("\n")
    }
}
