use helpers::{Result, input};
use multiplication::{extract_function, init, llm_response};

#[tokio::main]
async fn main() -> Result<()> {
    let (args, client, mut history) = init()?;
    let mut assistant_msg = "How can I help you today?".to_string();
    let mut user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));

    while &user_input != "exit" {
        history.add_assistant_msg(assistant_msg);
        history.add_user_msg(user_input);
        let llm_response = llm_response(&args, &client, &history).await?;

        assistant_msg = match extract_function(&llm_response) {
            Some((x, y)) => {
                println!("\t\tCalculated by tool");
                format!("{}", x * y)
            }
            None => llm_response,
        };

        user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));
    }

    Ok(())
}
