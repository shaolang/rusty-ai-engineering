use helpers::{Args, History, Result, create_openai_client, extract_texts, input};
use openai_oxide::types::responses::ResponseCreateRequest;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = create_openai_client(&args)?;
    let documentation =
        std::fs::read_to_string("resources/flamehamster.md").expect("markdown read");
    let mut history = History::new(
        format!(
            "You are an AI customer support technician who is knowledgeable about software products
         created by the company called GROSS. One such product is a web browser called
         Flamehamster. You are to answer user queries below solely on the following documentation:
         {documentation}"
        )
        .into(),
    );

    let assistant_msg = "How can I help you today?";
    history.add_assistant_msg(assistant_msg);

    let mut user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));
    while &user_input != "exit" {
        history.add_user_msg(user_input);
        let req = ResponseCreateRequest::new(&args.model)
            .temperature(args.temperature)
            .input(&history);
        let response = client.responses().create(req).await?;
        let llm_output_text = extract_texts(&response.output, true);
        println!("Assistant: {llm_output_text}");
        history.add_assistant_msg(llm_output_text);

        user_input = input("\nUser: ");
    }

    Ok(())
}
