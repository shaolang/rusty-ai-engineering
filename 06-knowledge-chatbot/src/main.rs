use async_openai::{
    Client,
    config::OpenAIConfig,
    types::responses::{CreateResponse, CreateResponseArgs},
};
use utils::{Args, Green, History, Red, cprintln, get_output, parse_args, read_stdin};

#[tokio::main]
async fn main() {
    let args = parse_args();
    let client = create_client(&args);
    let history = History::new();
    history.add_system_input(&format!(
        r#"
        You are an AI customer support technician who is knowledgeable about software products \
        created by the company called GROSS. One such product is a web browswer called \
        Flamehamster. You are to answer user query below solely on the following documentation: \
        {}
    "#,
        load_documentation()
    ));

    cprintln(
        Green,
        "Assistant: How can I help you today? (Enter 'quit' to end)",
    );
    loop {
        let user_input = read_stdin(Some("     User: ".to_string()));
        if &user_input == "quit" {
            break;
        }
        history.add_user_input(&user_input);
        let req = create_response_request(&args, &history);
        let resp_result = client.responses().create(req).await;
        match get_output(resp_result) {
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

fn load_documentation() -> String {
    std::fs::read_to_string("resources/flamehamster.md").expect("flamehamster.md read successfully")
}

fn create_response_request(args: &Args, history: &History) -> CreateResponse {
    CreateResponseArgs::default()
        .input(history.as_input_params())
        .model(args.model.to_owned())
        .temperature(args.temperature)
        .build()
        .expect("create response request succeeds")
}
