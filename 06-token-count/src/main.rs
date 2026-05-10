fn main() {
    let encoding_model = tiktoken_rs::cl100k_base().expect("encoding model loaded");
    let content = std::fs::read_to_string("./resources/flamehamster.md").expect("file read");
    let tokenized_text = encoding_model.encode_with_special_tokens(&content);
    let num_tokens = tokenized_text.len();

    println!("Token count: {num_tokens}");
}
