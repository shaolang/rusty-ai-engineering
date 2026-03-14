use arrow_array::RecordBatch;
use async_openai::types::responses::{CreateResponse, CreateResponseArgs};
use utils::get_output;
use vector_db::{VectorDb, VectorDbRecord, VectorTable};

pub fn start_open_ai_client() -> OpenAI {
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

pub async fn populate_vector_db(table_name: &str) -> VectorTable {
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
    db.open(table_name).await.unwrap()
}

pub struct OpenAI {
    client: async_openai::Client<async_openai::config::OpenAIConfig>,
    pub history: utils::History,
    model: String,
    system_input: String,
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
    pub fn new(system_input: &str) -> Self {
        let args = utils::parse_args();
        let config = async_openai::config::OpenAIConfig::new().with_api_base(args.base_url);
        let history = utils::History::new();
        history.add_system_input(system_input);

        Self {
            client: async_openai::Client::with_config(config),
            history,
            model: args.model,
            system_input: system_input.to_string(),
            temperature: args.temperature,
        }
    }

    pub fn reset(&self) {
        self.history.clear();
        self.history.add_system_input(&self.system_input);
    }

    pub async fn send(&self, user_input: impl AsRef<str>) -> String {
        self.history.add_user_input(user_input.as_ref());
        let req = self.__create_response(self.history.as_input_params());
        let response = self.client.responses().create(req).await;

        match get_output(response) {
            Ok((text, output_items)) => {
                self.history.add_assistant_outputs(&output_items);
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

pub fn augment_user_prompt(user_input: impl AsRef<str>, documentation: impl AsRef<str>) -> String {
    format!(
        "Here are excerpts from the official GROSS product documentation: {}.
        Use whatever info from the documentation exercepts (and no other info) to answer
        the following query: {}",
        documentation.as_ref(),
        user_input.as_ref()
    )
}

pub fn extract_chunk_texts(chunks: Vec<RecordBatch>) -> String {
    Chunk::from_record_batches(chunks)
        .iter()
        .map(|chunk| chunk.chunk_text.clone())
        .collect::<Vec<String>>()
        .join("\n")
}

fn create_chunks_from(manual: &str) -> Vec<Chunk> {
    let text = std::fs::read_to_string(format!("resources/{manual}.md")).expect("md file read");
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
    fancy_regex::RegexBuilder::new(r#"(?m)^# .+?(?=^# |\Z)"#)
        .dot_matches_new_line(true)
        .multi_line(true)
        .build()
        .expect("well-formed regex")
        .find_iter(md_text.as_ref())
        .map(|m| m.unwrap().as_str().to_string())
        .collect()
}
