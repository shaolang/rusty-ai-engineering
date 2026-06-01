use helpers::{Args, History, Result, create_openai_client, extract_texts, input};
use openai_oxide::types::responses::{Response, ResponseCreateRequest, ResponseTool};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = create_openai_client(&args)?;
    let tools = tools();
    let mut history = History::new(
        "
        You are a helpful AI assistant. If you ever need to multiply two numbers, DO NOT attempt
        to answer with your internal knowledge. Instead, use your multiply tool."
            .into(),
    );
    let mut assistant_msg = "How can I help you today?".to_string();
    let mut user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));

    while &user_input != "exit" {
        history.add_assistant_msg(assistant_msg);
        history.add_user_msg(user_input);
        let mut resp = client
            .responses()
            .create(new_request(&args, &history, &tools))
            .await?;
        if let Some((fn_call, call_id, tool)) = find_function_call(&resp) {
            let result = match tool {
                Tool::Multiply {
                    first_number,
                    second_number,
                } => serde_json::json!(first_number * second_number),
                Tool::ReadWebpage { url } => serde_json::Value::String(read_webpage(url).await),
            };
            history.add_function_call_msg(fn_call);
            history.add_function_call_output(&call_id, format!("{result}"));
            let req = new_request(&args, &history, &tools);
            resp = client.responses().create(req).await?;
        }

        assistant_msg = extract_texts(&resp.output, true);
        user_input = input(format!("Assistant: {}\n\nUser: ", assistant_msg.trim()));
    }

    Ok(())
}

fn new_request(args: &Args, history: &History, tools: &[ResponseTool]) -> ResponseCreateRequest {
    ResponseCreateRequest::new(&args.model)
        .temperature(args.temperature)
        .input(history)
        .tools(tools.to_owned())
}

fn find_function_call(resp: &Response) -> Option<(Value, String, Tool)> {
    resp.function_calls().pop().map(|r| {
        let json = serde_json::json!({r.name.clone(): r.arguments});
        let tool: Tool = serde_json::from_value(json).unwrap();
        let args = serde_json::to_string(&r.arguments).unwrap();
        let fn_call = serde_json::json!({
            "type": "function_call",
            "call_id": r.call_id,
            "name": r.name,
            "arguments": args,
        });
        (fn_call, r.call_id, tool)
    })
}

async fn read_webpage(url: impl AsRef<str>) -> String {
    let client = reqwest::ClientBuilder::new().user_agent(reqwest::header::USER_AGENT).build().unwrap();
    let resp = client.get(url.as_ref())
        .send()
        .await
        .expect("page read");
    resp.text().await.unwrap()
}

fn tools() -> Vec<ResponseTool> {
    vec![
        ResponseTool::Function {
            name: "multiply".to_string(),
            description: Some("multiply two numbers to get a product.".to_string()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "first_number": {"type": "integer"},
                    "second_number": {"type": "integer"},
                },
                "required": ["first_number", "second_number"]})),
            strict: Some(false),
        },
        ResponseTool::Function {
            name: "read-webpage".to_string(),
            description: Some("access a webpage and obtains its text.".to_string()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "the URL of the webpage"
                    }
                },
                "required": ["url"]
            })),
            strict: Some(false),
        },
    ]
}

#[derive(Debug, serde::Deserialize)]
enum Tool {
    #[serde(alias = "multiply")]
    Multiply {
        first_number: i32,
        second_number: i32,
    },
    #[serde(alias = "read-webpage")]
    ReadWebpage { url: String },
}
