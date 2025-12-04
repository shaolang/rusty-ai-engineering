use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::responses::{CreateResponse, CreateResponseArgs};
use utils::{Args, Green, cprintln, get_output_text_from_response, parse_args, read_stdin};

#[tokio::main]
async fn main() {
    let args = parse_args();

    let prompt = "Enter a phrase and I'll translate it into Spanish!\n> ";
    let user_input = read_stdin(Some(prompt.to_string()));
    let input = format!("{}: {}.\n\n{}",
        "Translate the following phrase into Spanish",
        user_input,
        "Answer with just the translate and no other introductory text.");

    let client = create_client(&args);
    let req = create_response_request(&args, &input);
    let resp = client.responses().create(req).await;
    let resp_text = get_output_text_from_response(resp);
    cprintln(Green, &resp_text);
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
