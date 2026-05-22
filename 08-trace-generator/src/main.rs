use std::io::Write;

use eval_chatbot::{Record, init_chat_history, llm_response, setup, user_prompt};
use helpers::{Args, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let (client, db, _) = setup(&args, "target/07-rag-chatbot.db").await?;
    let mut writer = csv::Writer::from_path("target/traces.csv")?;

    for (i, q) in load_queries().iter().enumerate() {
        let mut history = init_chat_history();
        history.add_assistant_msg("How can I help you today?");

        let documentations = db.search::<Record>(&q.user_input, 3).await?;
        let user_query = user_prompt(&q.user_input, documentations);
        history.add_user_msg(user_query.clone());
        let ai_response = llm_response(&args, &client, &history).await;
        let trace = Trace {
            query_topic: q.topic.clone(),
            user_query,
            history: serde_json::to_string(&history).unwrap(),
            ai_response,
        };

        writer.serialize(&trace).expect("trace written");
        println!("query {} completed", i + 1);
        std::io::stdout().flush().expect("stdout flushed");
    }

    writer.flush().expect("csv writer flushed");

    Ok(())
}

fn load_queries() -> Vec<Query> {
    let rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path("resources/queries.csv")
        .expect("csv reader created");

    rdr.into_deserialize()
        .map(|r| r.expect("record read"))
        .collect()
}

#[derive(serde::Deserialize)]
struct Query {
    #[allow(dead_code)]
    topic: String,
    user_input: String,
}

#[derive(serde::Serialize)]
struct Trace {
    #[serde(rename = "Query Topic")]
    query_topic: String,

    #[serde(rename = "User Query")]
    user_query: String,

    #[serde(rename = "History")]
    history: String,

    #[serde(rename = "AI Response")]
    ai_response: String,
}
