use serde::{Deserialize, Serialize};
use utils::{Green, Red, cprint, cprintln};

#[tokio::main]
async fn main() {
    let table = eval_chatbot::populate_vector_db("gross").await;
    let openai = eval_chatbot::start_open_ai_client();
    let trace_file = std::fs::File::create("trace.csv").unwrap();
    let mut writer = csv::Writer::from_writer(trace_file);

    for query in load_queries("resources/queries.csv") {
        cprint(Red, "Tracing ");
        cprint(Green, &format!("{:15} {:·<80}", query.topic, query.query));
        openai.reset();
        let documentation = eval_chatbot::extract_chunk_texts(
            table
                .get_relevant_records(&query.query, "vectorized_text", 3)
                .await,
        );
        let response = openai
            .send(eval_chatbot::augment_user_prompt(
                &query.query,
                documentation,
            ))
            .await;
        let rec = OutRecord {
            topic: query.topic.clone(),
            query: query.query.clone(),
            history: serde_json::to_string(&openai.history).unwrap(),
            response,
        };
        writer.serialize(rec).unwrap();
        writer.flush().unwrap();
        cprintln(Green, "done");
    }
}

#[derive(Deserialize)]
struct Query {
    topic: String,
    query: String,
}

#[derive(Serialize)]
struct OutRecord {
    topic: String,
    query: String,
    history: String,
    response: String,
}

fn load_queries(file_name: &str) -> Vec<Query> {
    let file = std::fs::File::open(file_name).expect("file opened");
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);
    reader.deserialize().filter_map(|r| r.ok()).collect()
}

fn exclude_final_response(mut entire_history: serde_json::Value) -> serde_json::Value {
    entire_history.as_array_mut().unwrap().pop().unwrap()
}
