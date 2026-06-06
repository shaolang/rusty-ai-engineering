use helpers::{Args, History, Result, create_openai_client, extract_texts, input};
use openai_oxide::{
    OpenAI,
    types::responses::{FunctionCall, Response, ResponseCreateRequest, ResponseTool},
};
use podcast_agent::{create_audio, read_webpage, search_web};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = create_openai_client(&args)?;
    let mut history = History::new(
        "
        You are an AI assistant. You have access to several specialized tools. Here are your tools:

        <tools>
        * With the search-web tool, you have the ability to search the web based on a query and
          retrieve URLs of web pages relevant to that query. This is especially useful for
          searching for current information and information you don't possess in your internal
          knowledge.
        * With the read-webpage tool, you have the ability to read the text from a web page of any
          given URL. This is a useful tool to use in conjunction with the search-web tool.
          That is, the search-web tool retrieves URLs, and the read-webpage tool can read
          the text contained at those web paegs.
        * With the create-audio tool, you can convert a podcast script text into an audio wav
          podcast.
        </tools>"
            .into(),
    );
    let mut assistant_msg = "How can I help you today?".to_string();
    let mut user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));

    while &user_input != "exit" {
        history.add_assistant_msg(assistant_msg);
        history.add_user_msg(user_input);
        let mut resp = llm_response(&args, &client, &history).await?;
        let mut fn_calls = func_calls(&resp);

        while !fn_calls.is_empty() {
            loop {
                let Some((fcall, tool)) = fn_calls.pop_front() else {
                    break;
                };
                let result = match tool {
                    Tool::ReadWebpage { url } => read_webpage(url).await,
                    Tool::SearchWeb { query } => {
                        serde_json::to_string(&search_web(query).await).unwrap()
                    }
                    Tool::CreateAudio { script } => {
                        tokio::task::spawn_blocking(|| create_audio(script))
                            .await
                            .unwrap()
                    }
                };
                history.add_function_call_msg(&fcall);
                history.add_function_call_output(&fcall.call_id, result.as_str());
            }

            resp = llm_response(&args, &client, &history).await?;
            fn_calls = func_calls(&resp);
        }

        assistant_msg = extract_texts(&resp.output, true);
        user_input = input(format!("Assistant: {}\n\nUser: ", assistant_msg.trim()));
    }

    Ok(())
}

async fn llm_response(args: &Args, client: &OpenAI, history: &History) -> Result<Response> {
    let req = ResponseCreateRequest::new(&args.model)
        .temperature(args.temperature)
        .tools(tools())
        .input(history);

    Ok(client.responses().create(req).await.unwrap())
}

fn tools() -> Vec<ResponseTool> {
    let search_web_tool = ResponseTool::Function {
        name: "search-web".to_string(),
        description: Some(
            "Search the web based on a query and retrieve URLs of web pages relevant to that query"
                .to_string(),
        ),
        parameters: Some(serde_json::json!({
            "type": "object",
            "properties": {
                "query": {"type": "string"},
            },
            "required": ["query"]})),
        strict: Some(true),
    };

    let read_webpage_tool = ResponseTool::Function {
        name: "read-webpage".to_string(),
        description: Some("Read the text from a page web of the given URL.".to_string()),
        parameters: Some(serde_json::json!({
            "type": "object",
            "properties": {
                "url": {"type": "string"},
            },
            "required": ["url"]})),
        strict: Some(true),
    };

    let create_audio_tool = ResponseTool::Function {
        name: "create-audio".to_string(),
        description: Some("Create a wav file from the given script.".to_string()),
        parameters: Some(serde_json::json!({
            "type": "object",
            "properties": {
                "script": {"type": "string"},
            },
            "required": ["script"]})),
        strict: Some(true),
    };

    vec![search_web_tool, read_webpage_tool, create_audio_tool]
}

fn func_calls(resp: &Response) -> std::collections::VecDeque<(FunctionCall, Tool)> {
    resp.function_calls()
        .into_iter()
        .map(|fcall| {
            let json = serde_json::json!({fcall.name.clone(): fcall.arguments});
            let tool: Tool = serde_json::from_value(json).unwrap();
            (fcall, tool)
        })
        .collect()
}

#[derive(Debug, serde::Deserialize)]
enum Tool {
    #[serde(alias = "read-webpage")]
    ReadWebpage { url: String },

    #[serde(alias = "search-web")]
    SearchWeb { query: String },

    #[serde(alias = "create-audio")]
    CreateAudio { script: String },
}
