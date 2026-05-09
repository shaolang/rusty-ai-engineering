use openai_oxide::{OpenAI, OpenAIError};

use crate::Args;

pub fn create_openai_client(args: &Args) -> Result<OpenAI, OpenAIError> {
    let config = openai_oxide::ClientConfig::from_env()?.base_url(&args.base_url);
    Ok(OpenAI::with_config(config))
}
