use helpers::{Args, History, Result, input};
use multiplication::{extract_function, init, llm_response, multiply};
use openai_oxide::OpenAI;

#[tokio::main]
async fn main() -> Result<()> {
    let additional_instructions = Some(
        "
        If you ever want to read the contents of a web page, use this
        notation: <<read_webpage(url)>>. For example, if you want to know the
        text contained within the website at the url https://example_site.com, output
        this: <<read_webpage(https://example_site.com)>>"
            .to_string(),
    );
    let (args, client, mut history) = init(additional_instructions)?;
    let mut assistant_msg = "How can I help you today?".to_string();
    let mut user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));

    while &user_input != "exit" {
        history.add_assistant_msg(assistant_msg);
        history.add_user_msg(user_input);
        let llm_response = llm_response(&args, &client, &history).await?;

        assistant_msg = match extract_function(&llm_response) {
            Some(func) if func.starts_with("multiply") => {
                println!("\t\tCalculated by tool");
                feed_back(&args, &client, &mut history, format!("{}", multiply(&func))).await?
            }
            Some(func) if func.starts_with("read_webpage") => {
                let content = read_webpage(&func).await?;
                feed_back(&args, &client, &mut history, content).await?
            }
            _ => llm_response,
        };

        user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));
    }

    Ok(())
}

async fn read_webpage(func: &str) -> Result<String> {
    let pattern = fancy_regex::Regex::new(r#"\(\s*(\S+)\s*\)"#).unwrap();
    let url = pattern
        .captures(func)
        .unwrap()
        .unwrap()
        .get(1)
        .unwrap()
        .as_str();
    let client = reqwest::ClientBuilder::new()
        .user_agent(reqwest::header::USER_AGENT)
        .build()?;
    let resp = client.get(url).send().await?;
    let doc = scraper::Html::parse_document(&resp.text().await?);
    let content = doc.root_element().text().collect::<Vec<&str>>().join(" ");

    Ok(content)
}

async fn feed_back(
    args: &Args,
    client: &OpenAI,
    history: &mut History,
    info: impl AsRef<str>,
) -> Result<String> {
    history.add_user_msg(format!(
        "Here is information to use to respond to the user's previous query:
         <info>{}</info>",
        info.as_ref()
    ));
    Ok(llm_response(&args, &client, &history)
        .await?
        .trim()
        .to_string())
}
