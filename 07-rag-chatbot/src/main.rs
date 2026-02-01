use std::fs;

use arrow_array::cast::as_string_array;
use arrow_array::{Array, RecordBatch};
use async_openai::types::responses::{CreateResponse, CreateResponseArgs};
use async_openai::{Client, config::OpenAIConfig};
use futures::TryStreamExt;
use lancedb::Table;
use lancedb::query::{ExecutableQuery, QueryBase};
use rag_chatbot as rag;
use utils::{Args, Green, History, Red, cprintln, get_output, parse_args, read_stdin};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args();
    let mut embedder = rag::TextEmbedder::new();
    let md_text = fs::read_to_string("resources/flamehamster.md").expect("loaded markdown");
    let doc = embedder.chunk_markdown_text_by_h1_header(md_text, "flamehamster");
    let vector_db = rag::VectorDb::connect("data/starter").await;
    let table = vector_db.create_table("flamehamster", doc).await;

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
        let search_results = get_relevant_chunks(&table, &mut embedder, &user_input, 3).await;
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
    embedder: &mut rag::TextEmbedder,
    query: impl AsRef<str>,
    top_k: usize,
) -> Vec<RecordBatch> {
    let query_vector = embedder.to_embedding(query);
    table
        .query()
        .nearest_to(query_vector.as_slice())
        .expect("created lancedb query")
        .limit(top_k)
        .refine_factor(5)
        .nprobes(10)
        .column("vector")
        .execute()
        .await
        .expect("ran query against lancedb")
        .try_collect()
        .await
        .unwrap()
}

fn combine_batches_to_string(batches: &[RecordBatch]) -> String {
    batches.iter()
        .flat_map(|batch| {
            let column = batch
                .column_by_name("chunk_text")
                .expect("retrieved chunk_text from lancedb query result");
            let string_array = as_string_array(column);
            (0..string_array.len())
                .map(|i| string_array.value(i))
                .collect::<Vec<&str>>()
        })
        .collect::<Vec<&str>>()
        .join("\n")
}
