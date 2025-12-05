use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::responses::{CreateResponse, CreateResponseArgs};
use utils::{Args, Green, cprintln, get_output_text_from_response, parse_args, read_stdin};

#[tokio::main]
async fn main() {
    let args = parse_args();
    let client = create_client(&args);
    let mut history = "Assistant: How can I help you today? (Enter 'quit' to end)".to_string();
    cprintln(Green, &history);
    loop {
        let user_input = read_stdin(Some("     User: ".to_string()));
        if &user_input == "quit" {
            break;
        }
        history = format!("{history}\nUser: {user_input}");
        let req = create_response_request(&args, &history);
        let resp = client.responses().create(req).await;
        let resp_text = get_output_text_from_response(resp);
        cprintln(Green, &format!("Assistant: {resp_text}"));
        history = format!("{history}\nUser: {resp_text}");
    }
}

fn create_client(args: &Args) -> Client<OpenAIConfig> {
    let config = OpenAIConfig::new().with_api_base(&args.base_url);
    Client::with_config(config)
}

fn create_response_request(args: &Args, user_input: &str) -> CreateResponse {
    CreateResponseArgs::default()
        .input(user_input)
        .model(args.model.to_owned())
        .temperature(args.temperature.clone())
        .build()
        .expect("create response request succeeds")
}
