use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::responses::{CreateResponse, CreateResponseArgs};
use utils::{
    Args, Green, History, Red, cprintln, get_output_from_response, parse_args, read_stdin,
};

#[tokio::main]
async fn main() {
    let args = parse_args();
    let client = create_client(&args);
    let history = History::new();
    history.add_system_input("You are an AI assistant who always talk like a pirate.");
    cprintln(Green, "Assistant: Arrgh, how can I help you, matey?");
    loop {
        let user_input = read_stdin(Some("     User: ".to_string()));
        if &user_input == "quit" {
            break;
        }
        history.add_user_input(&user_input);
        let req = create_response_request(&args, &history);
        let resp_result = client.responses().create(req).await;
        match get_output_from_response(resp_result) {
            Ok((text, outputs)) => {
                cprintln(Green, &format!("Assistant: {text}"));
                history.add_assistant_outputs(&outputs);
            }
            Err(e) => cprintln(Red, &format!("Error occurred: {e:?}")),
        }
    }
}

fn create_client(args: &Args) -> Client<OpenAIConfig> {
    let config = OpenAIConfig::new().with_api_base(&args.base_url);
    Client::with_config(config)
}

fn create_response_request(args: &Args, history: &History) -> CreateResponse {
    CreateResponseArgs::default()
        .input(history.as_input_params())
        .model(args.model.to_owned())
        .temperature(args.temperature)
        .build()
        .expect("create response request succeeds")
}
