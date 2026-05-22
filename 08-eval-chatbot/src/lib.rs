use std::path::Path;

use helpers::{Args, Embed, History, Result, VectorDb, create_openai_client, extract_texts};
use openai_oxide::{OpenAI, types::responses::ResponseCreateRequest};

pub async fn setup(args: &Args, db_fname: impl AsRef<Path>) -> Result<(OpenAI, VectorDb, History)> {
    let client = create_openai_client(&args)?;
    let db = populate_vectordb(db_fname).await?;
    let history = init_chat_history();

    Ok((client, db, history))
}

pub async fn populate_vectordb(path: impl AsRef<Path>) -> Result<VectorDb> {
    let needs_init = !path.as_ref().exists();
    let db = VectorDb::try_connect(path).await?;

    if needs_init {
        db.create_table::<Record>().await?;
        let resources = std::path::PathBuf::from("resources/");
        let pattern = fancy_regex::Regex::new(r#"(?mis)^# .+?(?=^# |\Z)"#).unwrap();

        for manual in [
            "emrgency.md",
            "flamehamster.md",
            "guineapigment.md",
            "rumblechirp.md",
            "verbiagepp.md",
        ] {
            let fpath = resources.join(manual);
            let name = fpath.file_stem().unwrap().to_str().unwrap();
            let content = std::fs::read_to_string(&fpath).expect("markdown read");
            let recs: Vec<Record> = pattern
                .find_iter(&content)
                .into_iter()
                .enumerate()
                .map(|(i, text)| Record {
                    id: format!("{name}-{i}"),
                    chunk_text: text.unwrap().as_str().to_string(),
                })
                .collect();

            db.insert(recs).await?;
        }
    }

    Ok(db)
}

pub fn init_chat_history() -> History {
    let mut history = History::new();
    history.add_developer_msg(
        "You are an AI customer support techician who is knowledgeable about software products
         created by the company called GROSS. The products are:
         * Flamehamster, a web browser.
         * Rumblechirp, an email client.
         * GuineaPigment, a drawing tool for creating/editing SVGs.
         * EMRgency, an electronic medical record system.
         * Verbiage++, a content management system.",
    );

    history
}

pub fn user_prompt(user_input: &str, records: Vec<Record>) -> String {
    let doc = records
        .iter()
        .map(Record::to_string)
        .collect::<Vec<String>>()
        .join(" ");
    format!(
        "Here are excerpts from the official GROSS product documentation: {doc}. \
         Use whatever info from the above documentation excerpts (and no other info) to answer \
         the following query: {user_input}"
    )
}

pub async fn llm_response(args: &Args, client: &OpenAI, history: &History) -> String {
    let response = client
        .responses()
        .create(
            ResponseCreateRequest::new(&args.model)
                .temperature(args.temperature)
                .input(history),
        )
        .await
        .expect("response request sent");
    extract_texts(&response.output, true)
}

#[derive(Embed)]
pub struct Record {
    id: String,
    #[embed]
    chunk_text: String,
}

impl ToString for Record {
    fn to_string(&self) -> String {
        self.chunk_text.clone()
    }
}
