use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::responses::{CreateResponse, CreateResponseArgs};
use utils::{Args, Green, Red, cprintln, get_output_from_response, parse_args, read_stdin};

#[tokio::main]
async fn main() {
    let args = parse_args();
    let client = create_client(&args);

    cprintln(
        Green,
        "Assistant: How can I help you today? (Enter 'quit' to end)",
    );
    loop {
        let user_input = read_stdin(Some("     User: ".to_string()));
        if &user_input == "quit" {
            break;
        }
        let req = create_response_request(&args, &user_input);
        let resp = client.responses().create(req).await;
        match get_output_from_response(resp) {
            Ok((text, _)) => cprintln(Green, &format!("Assistant: {text}")),
            Err(e) => cprintln(Red, &format!("An error occurred: {e:?}")),
        }
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
        .temperature(args.temperature)
        .build()
        .expect("create response request succeeds")
}
