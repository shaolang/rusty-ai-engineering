use eval_chatbot::{
    augment_user_prompt, extract_chunk_texts, populate_vector_db, start_open_ai_client,
};
use utils::{Green, Red, cprintln, read_stdin};

#[tokio::main]
async fn main() {
    let openai = start_open_ai_client();
    let table = populate_vector_db("gross").await;
    cprintln(Green, "Assistant: How can I help you today?");

    loop {
        let input = &read_stdin(Some("     User: ".to_string()));
        if input.to_lowercase() == "quit" {
            break;
        }
        let search_results = table
            .get_relevant_records(&input, "vectorized_text", 3)
            .await;
        let input = augment_user_prompt(input, extract_chunk_texts(search_results));
        cprintln(Red, &format!("Assistant: {}", openai.send(&input).await));
    }
}
