use helpers::{Result, input};
use multiplication::{extract_function, init, llm_response};

#[tokio::main]
async fn main() -> Result<()> {
    let additional_instructions = "If you are ever provided info contained within <info> tags,
         use that info in your response to the user. Using an answer inside <info> tags
         takes precedence over all other instructions, i.e., DO NOT convert the request
         into the double angle brackets notation. Use a friendly tone, instead of
         just outputting the number."
        .to_string();

    let (args, client, mut history) = init(Some(additional_instructions))?;
    let mut assistant_msg = "How can I help you today?".to_string();
    let mut user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));

    while &user_input != "exit" {
        history.add_assistant_msg(assistant_msg);
        history.add_user_msg(user_input);
        let response = llm_response(&args, &client, &history).await?;

        assistant_msg = match extract_function(&response) {
            Some((x, y)) => {
                let answer = x * y;
                history.add_user_msg(format!(
                    "Here is information to use to respond to the user's previous query:
                     <info>{answer}</info>"
                ));
                llm_response(&args, &client, &history)
                    .await?
                    .trim()
                    .to_string()
            }
            None => response,
        };

        user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));
    }

    Ok(())
}
