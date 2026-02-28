use std::fs;

use arrow_array::cast::as_largestring_array;
use arrow_array::{Array, RecordBatch};
use async_openai::types::responses::{CreateResponse, CreateResponseArgs};
use async_openai::{Client, config::OpenAIConfig};
use futures::TryStreamExt;
use lancedb::Table;
use lancedb::query::{ExecutableQuery, QueryBase};
use pcre2::bytes::RegexBuilder;
use utils::{Args, Green, History, Red, cprintln, get_output, parse_args, read_stdin};
use vector_db::{VectorDb, VectorDbRecord};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args();
    let md_text = fs::read_to_string("resources/flamehamster.md").expect("loaded markdown");
    let mut db = VectorDb::try_new("data/starter").await?;
    let chunks: Vec<Chunk> = split_markdown_by_h1(md_text)
        .into_iter()
        .enumerate()
        .map(|(i, manual)| Chunk {
            chunk_id: format!("{i}"),
            manual,
        })
        .collect();
    db.create_table_with_data("flamehamster", chunks)
        .await
        .expect("lance table created");
    let table = db.open("flamehamster").await.expect("table exists");
    let mut model = db.model.lock().expect("embedding model held");

    let client = create_client(&args);
    let history = History::new();
    history.add_system_input(
        "You are an AI customer support technician who is knowledgeable about
         software products created by the company GROSS. One such product is a
         web browser called Flamehamster.",
    );

    cprintln(Green, "Assistant: How can I help you today?");
    loop {
        let user_input = read_stdin(Some("     User: ".to_string()));
        if &user_input == "quit" {
            break;
        }
        let search_results = get_relevant_chunks(&table, &mut model, &user_input, 3).await;
        let documentation = combine_batches_to_string(search_results.as_slice());
        let user_input = format!(
            "Here are excerpts from the official Flamehamster web browser: {documentation}.
             Use whatever info from the above documentation excerpts (and no other info) to
             answer the following query: {user_input}"
        );
        history.add_user_input(&user_input);

        let req = create_response_request(&args, &history);
        let resp_result = client.responses().create(req).await;
        match get_output(resp_result) {
            Ok((text, outputs)) => {
                cprintln(Green, &format!("Assistant: {text}"));
                history.add_assistant_outputs(&outputs);
            }
            Err(e) => cprintln(Red, &format!("Error occurred: {e:?}")),
        }
    }

    Ok(())
}

#[derive(VectorDbRecord)]
struct Chunk {
    chunk_id: String,
    #[vector = "manual_vector"]
    manual: String,
}

fn create_client(args: &Args) -> Client<OpenAIConfig> {
    let config = OpenAIConfig::new().with_api_base(&args.base_url);
    Client::with_config(config)
}

fn create_response_request(args: &Args, history: &History) -> CreateResponse {
    CreateResponseArgs::default()
        .input(history.as_input_params())
        .model(args.model.to_owned())
        .temperature(args.temperature)
        .build()
        .expect("create response request succeeds")
}

async fn get_relevant_chunks(
    table: &Table,
    model: &mut fastembed::TextEmbedding,
    query: impl AsRef<str>,
    top_k: usize,
) -> Vec<RecordBatch> {
    let query_vector = model
        .embed([query.as_ref()], None)
        .map(|vv| vv[0].to_owned())
        .expect("embedding created");
    table
        .query()
        .nearest_to(query_vector.as_slice())
        .expect("created lancedb query")
        .limit(top_k)
        .refine_factor(5)
        .nprobes(10)
        .column("manual_vector")
        .execute()
        .await
        .expect("ran query against lancedb")
        .try_collect()
        .await
        .unwrap()
}

fn combine_batches_to_string(batches: &[RecordBatch]) -> String {
    batches
        .iter()
        .flat_map(|batch| {
            let column = batch
                .column_by_name("manual")
                .expect("retrieved manual from lancedb query result");
            let string_array = as_largestring_array(column);
            (0..string_array.len())
                .map(|i| string_array.value(i))
                .collect::<Vec<&str>>()
        })
        .collect::<Vec<&str>>()
        .join("\n")
}

fn split_markdown_by_h1(md_text: impl AsRef<str>) -> Vec<String> {
    RegexBuilder::new()
        .dotall(true)
        .crlf(true)
        .build(r#"(?m)^# .+?(?=^# |\Z)"#)
        .expect("well-formed regex")
        .find_iter(md_text.as_ref().as_bytes())
        .filter_map(|m| m.map(|m| m.as_bytes()).ok())
        .map(|m| String::from_utf8_lossy(m).trim().to_string())
        .collect()
}
