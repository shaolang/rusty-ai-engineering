use async_openai::{
    Client,
    config::OpenAIConfig,
    types::responses::{CreateResponse, CreateResponseArgs},
};
use utils::{Args, Green, OpenAIErrorExt, Red, cprintln, parse_args};

#[tokio::main]
async fn main() {
    let args = parse_args();
    let client = create_client(&args);
    let req = create_response_req(&args, "Who was the first person to land on the moon?");

    match client.responses().create(req).await {
        Err(e) => cprintln(
            Red,
            &format!("Error occurred: {:?}", e.try_extract_output()),
        ),
        Ok(resp) => cprintln(Green, &resp.output_text().unwrap_or("".to_string())),
    }
}

fn create_client(args: &Args) -> Client<OpenAIConfig> {
    let config = OpenAIConfig::default().with_api_base(&args.base_url);
    Client::with_config(config)
}

fn create_response_req(args: &Args, input: &str) -> CreateResponse {
    CreateResponseArgs::default()
        .input(input)
        .model(&args.model)
        .temperature(args.temperature)
        .build()
        .expect("create response request succeeds")
}
