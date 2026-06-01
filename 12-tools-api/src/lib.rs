use helpers::{Args, History};
use openai_oxide::types::responses::{ResponseCreateRequest, ResponseTool};

pub fn new_request(
    args: &Args,
    history: &History,
    tools: &[ResponseTool],
) -> ResponseCreateRequest {
    ResponseCreateRequest::new(&args.model)
        .temperature(args.temperature)
        .input(history)
        .tools(tools.to_owned())
}

pub async fn read_webpage(url: impl AsRef<str>) -> String {
    let client = reqwest::ClientBuilder::new()
        .user_agent(reqwest::header::USER_AGENT)
        .build()
        .unwrap();
    let resp = client.get(url.as_ref()).send().await.expect("page read");
    resp.text().await.unwrap()
}

pub fn tools() -> Vec<ResponseTool> {
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
pub enum Tool {
    #[serde(alias = "multiply")]
    Multiply {
        first_number: i32,
        second_number: i32,
    },
    #[serde(alias = "read-webpage")]
    ReadWebpage { url: String },
}
