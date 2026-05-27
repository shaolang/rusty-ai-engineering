use helpers::{Args, History, Result, create_openai_client, extract_texts};
use openai_oxide::{OpenAI, types::responses::ResponseCreateRequest};

pub fn init(additional_instructions: Option<String>) -> Result<(Args, OpenAI, History)> {
    let args = Args::parse();
    let client = create_openai_client(&args)?;
    let additional_instructions = additional_instructions.unwrap_or("".to_string());
    let history = History::new(
        format!("You are a helpful AI assistant. If you ever need to multiply two numbers, DO NOT attempt
         to answer with your internal knowledge. Instead, output a special notation with double
         angle brackets like this: <<multiply(first_number, second_number)>>.
         For example, if a user asks you to multiply 50 by 2,
         your output should be: <<multiply(50, 2)>>. A second example: a user asks you how many
         apples there are in five baskets and each basket contains twelve apples. Your output
         should be: <<multiply(5, 12)>>. {additional_instructions}").into()
    );

    Ok((args, client, history))
}

pub async fn llm_response(args: &Args, client: &OpenAI, history: &History) -> Result<String> {
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

pub fn extract_function(text: &str) -> Option<String> {
    let pattern =
        fancy_regex::Regex::new(r#"(?is)<<\s*([A-Za-z_]\w*\s*\(.+\))\s*>>"#).expect("valid regex");
    match pattern.captures(text) {
        Ok(Some(cs)) => Some(cs.get(1).unwrap().as_str().to_string()),
        _ => None,
    }
}

pub fn multiply(text: &str) -> i32 {
    let pattern = fancy_regex::Regex::new(r#"\(\s*(\d+)\s*,\s*(\d+)\s*\)"#).unwrap();
    let matches = pattern.captures(text).unwrap().unwrap();
    let x: i32 = matches.get(1).unwrap().as_str().parse().unwrap();
    let y: i32 = matches.get(2).unwrap().as_str().parse().unwrap();

    x * y
}
