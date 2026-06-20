use std::collections::VecDeque;

use agentic_workflow::initiate_podcast;
use helpers::{Args, History, create_openai_client, extract_texts, input};
use openai_oxide::{
    OpenAI, OpenAIError,
    types::responses::{Response, ResponseCreateRequest, ResponseTool},
};

#[tokio::main]
async fn main() -> helpers::Result<()> {
    let args = Args::parse();
    let client = create_openai_client(&args).unwrap();
    let mut history = History::new(
        r#"You are a podcast producer, creating news-based and explainer podcasts for people on any
         topic they choose..

         Here is the plan you should follow step by step to create a podcast:
         <olan>
         1. When the user describes te podcast they want, do not proceed to the next step until
            you've obtained the following information:
            * The topic of the podcast.
            * How long the podcast should be. (For example, five minutes long.)
            However, do not ask the user about the podcast style. Assume that the podcast style
            is a single host reporting news and insight.
         2. Next, create a simple summary describing the type of podcast the user wants. Ensure
            that this summary includes the desired time length of the podcast. For example,
            the summary might be: "A 3-minute podcast on the latest news, insights, and updates
            on the field of quantum physics"
         3. You have access to an initiate-podcast tool. Your next step is to call the
            initiate-podcast tool, passing along your summary to it.</plan>"#
            .into(),
    );
    let mut assistant_msg = "How can I help you today?".to_string();
    let mut user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));

    while &user_input != "exit" {
        history.add_assistant_msg(assistant_msg);
        history.add_user_msg(user_input);
        let mut response = llm_response(&client, &args, &history).await?;

        loop {
            let mut func_calls = VecDeque::from(response.function_calls());
            if func_calls.is_empty() {
                break;
            }

            while let Some(fcall) = func_calls.pop_front() {
                history.add_function_call_msg(&fcall);
                if &fcall.name == "initiate-podcast" {
                    let result = initiate_podcast(
                        fcall.arguments["podcast_description"].to_string(),
                        &client,
                        &args,
                    )
                    .await;
                    history.add_function_call_output(&fcall.call_id, result);
                }
            }

            response = llm_response(&client, &args, &history).await?;
        }

        assistant_msg = extract_texts(&response.output, true);
        user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));
    }

    Ok(())
}

async fn llm_response(
    client: &OpenAI,
    args: &Args,
    history: &History,
) -> Result<Response, OpenAIError> {
    let req = ResponseCreateRequest::new(&args.model)
        .temperature(args.temperature)
        .tools(tools())
        .input(history);

    client.responses().create(req).await
}

fn tools() -> Vec<ResponseTool> {
    vec![ResponseTool::Function {
        name: "initiate-podcast".to_string(),
        description: Some("Generates an audio podcast as a file called podcast.wav".to_string()),
        parameters: Some(serde_json::json!({
            "type": "object",
            "properties": {
                "podcast_description": {
                    "type": "string",
                }
            },
            "required": ["podcast_description"]
        })),
        strict: Some(true),
    }]
}
