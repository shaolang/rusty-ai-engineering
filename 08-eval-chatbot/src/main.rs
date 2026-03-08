use arrow_array::RecordBatch;
use async_openai::types::responses::{CreateResponse, CreateResponseArgs};
use utils::{Green, Red, cprintln, get_output, read_stdin};
use vector_db::{VectorDb, VectorDbRecord};

#[tokio::main]
async fn main() {
    let openai = start_open_ai_client();
    let table = populate_vector_db("gross")
        .await
        .open("gross")
        .await
        .unwrap();
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

fn start_open_ai_client() -> OpenAI {
    let system_input = "
        You are an AI customer support technician who is knowledgeable about
        software products created by the company called GROSS. The products are:
        * Flamehamster, a web browser.
        * Rumblechirp, an email client.
        * GuineaPigment, a drawing tool for creating/editing SVGs.
        * EMRgency, an electronic medical record system.
        * Verbiage++, a content management system.";
    OpenAI::new(system_input)
}

async fn populate_vector_db(table_name: &str) -> VectorDb {
    let mut db = VectorDb::try_new("data/gross")
        .await
        .expect("vector db created");
    let chunks: Vec<Chunk> = [
        "emrgency",
        "flamehamster",
        "guineapigment",
        "rumblechirp",
        "verbiagepp",
    ]
    .into_iter()
    .flat_map(create_chunks_from)
    .collect();
    db.create_table_with_data(table_name, chunks)
        .await
        .expect("chunks inserted");
    db
}

struct OpenAI {
    client: async_openai::Client<async_openai::config::OpenAIConfig>,
    history: utils::History,
    model: String,
    temperature: f32,
}

#[derive(VectorDbRecord)]
struct Chunk {
    id: String,
    manual: String,
    #[vector = "vectorized_text"]
    chunk_text: String,
}

impl OpenAI {
    fn new(system_input: &str) -> Self {
        let args = utils::parse_args();
        let config = async_openai::config::OpenAIConfig::new().with_api_base(args.base_url);
        let history = utils::History::new();
        history.add_system_input(system_input);

        Self {
            client: async_openai::Client::with_config(config),
            history,
            model: args.model,
            temperature: args.temperature,
        }
    }

    async fn send(&self, user_input: impl AsRef<str>) -> String {
        self.history.add_user_input(user_input.as_ref());
        let req = self.__create_response(self.history.as_input_params());
        let response = self.client.responses().create(req).await;

        match get_output(response) {
            Ok((text, output_items)) => {
                self.history.add_assistant_outputs(&output_items);
                println!("{:#?}", self.history);
                text
            }
            Err(err) => format!("An error occurred: {err:?}"),
        }
    }

    fn __create_response(
        &self,
        input: async_openai::types::responses::InputParam,
    ) -> CreateResponse {
        CreateResponseArgs::default()
            .model(&self.model)
            .temperature(self.temperature)
            .input(input)
            .build()
            .expect("create response request")
    }
}

fn augment_user_prompt(user_input: impl AsRef<str>, documentation: impl AsRef<str>) -> String {
    format!(
        "Here are excerpts from the official GROSS product documentation: {}.
        Use whatever info from the documentation exercepts (and no other info) to answer
        the following query: {}",
        documentation.as_ref(),
        user_input.as_ref()
    )
}

fn extract_chunk_texts(chunks: Vec<RecordBatch>) -> String {
    Chunk::from_record_batches(chunks)
        .iter()
        .map(|chunk| chunk.chunk_text.clone())
        .collect::<Vec<String>>()
        .join("\n")
}

fn create_chunks_from(manual: &str) -> Vec<Chunk> {
    let text = std::fs::read_to_string(&format!("resources/{manual}.md")).expect("md file read");
    split_markdown_by_h1(&text)
        .iter()
        .enumerate()
        .map(|(i, s)| Chunk {
            id: format!("{manual}-chunk-{i}"),
            manual: manual.to_string(),
            chunk_text: s.to_owned(),
        })
        .collect()
}

fn split_markdown_by_h1(md_text: impl AsRef<str>) -> Vec<String> {
    pcre2::bytes::RegexBuilder::new()
        .dotall(true)
        .crlf(true)
        .build(r#"(?m)^# .+?(?=^# |\Z)"#)
        .expect("well-formed regex")
        .find_iter(md_text.as_ref().as_bytes())
        .filter_map(|m| m.map(|m| m.as_bytes()).ok())
        .map(|m| String::from_utf8_lossy(m).trim().to_string())
        .collect()
}
