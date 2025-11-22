use async_openai::{
    error::OpenAIError,
    types::responses::{CreateResponse, Input},
};
use common::{error, get_user_input, print_llm_output};

#[tokio::main]
async fn main() {
    let args = common::Args::parse_args();
    let client = args.create_client();

    while let Some(input) = get_user_input("Enter phrase for checking", "quit") {
        let req = create_response_request(&args, &input);
        match client.responses().create(req).await {
            Ok(resp) => print_llm_output(&resp.output_text.unwrap()),
            Err(OpenAIError::JSONDeserialize(e, content)) => {
                error!("Error occurred: {e}. Attempting to extract only content from output");
                print_llm_output(&common::extract_content_from_json(&content));
            }
            Err(e) => error!("Error occurred: {e}"),
        }
    }
}

fn create_response_request(args: &common::Args, user_input: &str) -> CreateResponse {
    let input = format!(
        "Check the TEXT below for grammatical errors. If there are errors, simply rewrite
         the text so that it's correct. If there are no errors, just output the original text.
         Here's the TEXT: {user_input}"
    );

    CreateResponse {
        input: Input::Text(input),
        model: args.model.to_owned(),
        max_output_tokens: args.max_tokens.clone(),
        temperature: args.temperature.clone(),
        ..Default::default()
    }
}
