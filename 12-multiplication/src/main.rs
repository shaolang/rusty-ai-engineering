use helpers::{Args, History, Result, create_openai_client, extract_texts, input};
use openai_oxide::{OpenAI, types::responses::ResponseCreateRequest};

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

fn init() -> Result<(Args, OpenAI, History)> {
    let args = Args::parse();
    let client = create_openai_client(&args)?;
    let history = History::new(
        "You are a helpful AI assistant. If you ever need o multiply two numbers, DO NOT attempt
         to answer with your internal knowledge. Instead, output a special notation with double
         angle brackets like this: <<multiply(first_number, second_number)>>.
         For example, if a user asks you to multiply 50 by 2,
         your output should be: <<multiply(50, 2)>>. A second example: a user asks you how many
         apples there are in five baskets and each basket contains twelve apples. Your output
         should be: <<multiply(5, 12)>>."
            .into(),
    );

    Ok((args, client, history))
}

async fn llm_response(args: &Args, client: &OpenAI, history: &History) -> Result<String> {
    let response = client
        .responses()
        .create(
            ResponseCreateRequest::new(&args.model)
                .temperature(args.temperature)
                .input(history),
        )
        .await?;

    Ok(extract_texts(&response.output, true))
}

fn extract_function(text: &str) -> Option<(i32, i32)> {
    let pattern =
        fancy_regex::Regex::new(r#"(?is)<<\s*([A-Za-z_]\w*)\s*\((\d+)\s*,\s*(\d+)\)\s*>>"#)
            .expect("valid regex");
    match pattern.captures(text) {
        Ok(Some(cs)) => {
            let func = cs.get(1).unwrap();
            if func.as_str() == "multiply" {
                let arg1: i32 = cs.get(2).unwrap().as_str().parse().unwrap();
                let arg2: i32 = cs.get(3).unwrap().as_str().parse().unwrap();
                Some((arg1, arg2))
            } else {
                None
            }
        }
        _ => None,
    }
}
