use utils::{Green, History, cprintln, read_stdin};

#[tokio::main]
async fn main() {
    let table = eval_chatbot::populate_vector_db("gross").await;
    let openai = eval_chatbot::OpenAI::new(&reduce_hallucinations::system_prompt(None));

    loop {
        cprintln(Green, "Assistant: How can I help you today");
        let user_input = read_stdin(Some("    User: ".to_string()));
        if &user_input == "quit" {
            break;
        }
        let expanded_user_input = expand_query(&openai, &user_input).await;
        let rag_chunks = eval_chatbot::extract_chunk_texts(
            table
                .get_relevant_records(&expanded_user_input, "vectorized_text", 3)
                .await,
        );
        openai
            .history
            .replace_first_system_prompt(&reduce_hallucinations::system_prompt(Some(rag_chunks)));
        let response = reduce_hallucinations::remove_bracket_tags(&openai.send(&user_input).await);
        cprintln(Green, &format!("Assistant: {response}"));
    }
}

async fn expand_query(openai: &eval_chatbot::OpenAI, user_input: &str) -> String {
    openai.history.replace_first_system_prompt(""); // exclude system prompt when expanding query
    let req = format!(
        "Rewrite, in an expanded way, what the user means to say in their final prompt of
         the following conversation: {user_input}"
    );
    let response = openai.send(&req).await;

    // req and reponse shouldn't be in the history, 'cos response is meant for rag retrieval
    // in main loop above
    openai.history.remove_last();
    openai.history.remove_last();

    response
}
