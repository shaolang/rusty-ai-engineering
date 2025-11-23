use async_openai::{
    Client,
    config::OpenAIConfig,
    types::responses::{CreateResponse, Input},
};
use common::{Args, ResponseExt, get_user_input, print_llm_output};

#[tokio::main]
async fn main() {
    let args = Args::parse_args();
    let client = args.create_client();
    let mut history = String::new();

    if let Some(user_input) = get_user_input("How can I help you today?", "quit", true) {
        let output = send(&user_input, &args, &client).await;
        print_llm_output(&output);
        history += &format!("Assistant: {}", output);
    }

    while let Some(user_input) = get_user_input("", "quit", false) {
        history += &format!("\nUser: {user_input}");
        let output = send(&history, &args, &client).await;
        print_llm_output(&output);
        history += &format!("\nAssistant: {}", output);
    }
}

async fn send(user_input: &str, args: &Args, client: &Client<OpenAIConfig>) -> String {
    let req = create_response_request(user_input, args);
    client.responses().create(req).await.extract_output()
}

fn create_response_request(user_input: &str, args: &Args) -> CreateResponse {
    CreateResponse {
        input: Input::Text(user_input.to_owned()),
        max_output_tokens: args.max_tokens.clone(),
        model: args.model.clone(),
        temperature: args.temperature.clone(),
        ..Default::default()
    }
}
