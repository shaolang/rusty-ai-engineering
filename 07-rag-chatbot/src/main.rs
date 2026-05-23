use helpers::{Args, Embed, History, Result, VectorDb, create_openai_client, extract_texts, input};
use openai_oxide::types::responses::ResponseCreateRequest;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = create_openai_client(&args)?;
    let mut history = new_history();

    let mut assistant_msg = "How can I help you today?".to_string();
    history.add_assistant_msg(assistant_msg.to_string());

    let db = prep_vectordb("target/07-rag-chatbot.db").await?;
    let mut user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));

    while &user_input != "exit" {
        let documentation = db
            .search(&user_input, 3)
            .await?
            .iter()
            .map(|rec: &Record| rec.chunk_text.as_str())
            .collect::<Vec<&str>>()
            .join(" ");
        history.add_assistant_msg(assistant_msg);
        history.add_user_msg(format!(
            "Here are excerpts from the official Flamehamster web browser documentation: \
                 {documentation}. Use whatever info from the above documentation excerpts \
                 (and no other info) to answer the following query: {user_input}"
        ));
        let req = ResponseCreateRequest::new(&args.model)
            .temperature(args.temperature)
            .input(&history);
        let resp = client.responses().create(req).await?;
        assistant_msg = extract_texts(&resp.output, true);

        user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));
    }

    Ok(())
}

fn new_history() -> History {
    let system_prompt = "You are an AI customer support technician who is knowledgeable about \
         software product created by the company called GROSS. One such product \
         is a web browser called Flamehamster.";
    History::new(system_prompt.into())
}

async fn prep_vectordb(db_path: impl AsRef<std::path::Path>) -> Result<VectorDb> {
    let need_init = !db_path.as_ref().exists();
    let db = VectorDb::try_connect(db_path).await?;

    if need_init {
        let records = create_records_from_documentation();

        db.create_table::<Record>().await?;
        db.insert(records).await?;
    }

    Ok(db)
}

fn create_records_from_documentation() -> Vec<Record> {
    let pattern = fancy_regex::Regex::new(r#"(?mis)^# .+?(?=^# |\Z)"#).unwrap();
    let doc = std::fs::read_to_string("resources/flamehamster.md").unwrap();

    pattern
        .find_iter(&doc)
        .enumerate()
        .map(|(i, s)| Record {
            id: format!("chunk-{i}"),
            manual: "flamehamster".to_string(),
            chunk_text: s.unwrap().as_str().to_string(),
        })
        .collect()
}

#[derive(Debug, Embed)]
struct Record {
    id: String,
    #[embed]
    chunk_text: String,
    manual: String,
}
