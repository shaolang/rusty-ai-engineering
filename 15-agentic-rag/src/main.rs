use std::path::Path;

use helpers::{Args, Embed, History, Result, VectorDb, create_openai_client, extract_texts, input};
use openai_oxide::{
    OpenAI, OpenAIError,
    types::responses::{Response, ResponseCreateRequest, ResponseInput, ResponseTool},
};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = create_openai_client(&args)?;
    let vectordb = populate_vectordb("target/agentic-rag.db").await?;
    let mut history = History::new(include_str!("./main-system-prompt.txt").into());
    let mut assistant_msg = "How can I help you today?".to_string();
    let mut user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));

    while &user_input != "exit" {
        history.add_assistant_msg(assistant_msg);
        history.add_user_msg(user_input);
        let mut resp = llm_response(&client, &args, &history, main_tool()).await?;

        loop {
            let func_calls = resp.function_calls();
            if func_calls.is_empty() {
                break;
            }

            for fn_call in func_calls {
                if &fn_call.name == "research-docs" {
                    research_docs(&client, &args, &history, &vectordb).await?;
                } else {
                    eprintln!("Agent invokes an unknown tool: {}", fn_call.name);
                }
            }

            resp = llm_response(&client, &args, &history, main_tool()).await?;
        }

        assistant_msg = extract_texts(&resp.output, true);
        user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));
    }

    Ok(())
}

fn main_tool() -> Vec<ResponseTool> {
    vec![ResponseTool::Function {
        name: "research-docs".to_string(),
        description: Some("Retrieves relevant documentation excerpts".to_string()),
        parameters: None,
        strict: Some(true),
    }]
}

async fn llm_response(
    client: &OpenAI,
    args: &Args,
    input: impl Into<ResponseInput>,
    tools: Vec<ResponseTool>,
) -> std::result::Result<Response, OpenAIError> {
    let req = ResponseCreateRequest::new(&args.model)
        .temperature(args.temperature)
        .tools(tools)
        .input(input);
    client.responses().create(req).await
}

async fn research_docs(
    client: &OpenAI,
    args: &Args,
    history: &History,
    vectordb: &VectorDb,
) -> Result<()> {
    let ConversationData {
        product_name,
        expanded_query,
    } = classify_and_expand(history, client, args).await?;
    let mut history =
        History::new(format!(include_str!("./research-system-prompt.txt"), expanded_query).into());
    let mut rag_chunks: Vec<Record> = Vec::new();

    for _ in 0..5 {
        let resp = llm_response(client, args, &history, rag_tools()).await?;
        if !resp.has_function_calls() {
            break;
        }
        for fn_call in resp.function_calls() {
            match fn_call.name.as_str() {
                "search-manual" => {
                    let query = fn_call.arguments["query"]
                        .as_str()
                        .expect("query argument exists");
                    let topk = fn_call.arguments["top-k"]
                        .as_u64()
                        .expect("top-k argument exists") as usize;
                    let mut records = search_manual(vectordb, query, topk, &product_name).await;
                    rag_chunks.append(&mut records);
                    history.add_function_call_output(
                        &fn_call.call_id,
                        serde_json::to_string(&rag_chunks).unwrap(),
                    );
                }
                "read-webpage" => {
                    let url = fn_call.arguments["url"]
                        .as_str()
                        .expect("url argument exists");
                    let text = read_webpage(url).await;
                    history.add_function_call_output(&fn_call.call_id, text);
                }
                tool_name => {
                    eprintln!("Unknown tool called: {tool_name}");
                }
            }
        }
    }

    Ok(())
}

async fn classify_and_expand(
    conversation: &History,
    client: &OpenAI,
    args: &Args,
) -> Result<ConversationData> {
    let prompt = format!(
        include_str!("classify-system-prompt.txt"),
        conversation.exclude_system_prompt().to_string()
    );
    let req = ResponseCreateRequest::new(&args.model)
        .temperature(args.temperature)
        .input(prompt);
    let resp = client.responses().parse::<ConversationData>(req).await?;

    Ok(resp.parsed.unwrap())
}

fn rag_tools() -> Vec<ResponseTool> {
    vec![
        ResponseTool::Function {
            name: "search-manual".to_string(),
            description: Some("Searches a software documentation manual and retrieves excerpts relevant to a user query".to_string()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "A user query for which we need to search the documentation manual"
                    },
                    "top-k": {
                        "type": "integer",
                        "description": "The number of excerpts to retrieve from the documentation"
                    }
                },
                "required": ["query", "top-k"]
            })),
            strict: Some(true),
        },
        ResponseTool::Function {
            name: "read-webpage".to_string(),
            description: Some("Reads the text of a web page at a given URL".to_string()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "the URL whose web page is to be read"
                    }
                },
                "required": ["url"]
            })),
            strict: Some(true),
        },
    ]
}

pub async fn read_webpage(url: &str) -> String {
    let resp = reqwest::get(url).await.expect("http get succeeded");
    let doc = scraper::Html::parse_document(&resp.text().await.unwrap());
    doc.root_element().text().collect::<Vec<&str>>().join("")
}

pub async fn search_manual(
    vectordb: &VectorDb,
    query: &str,
    topk: usize,
    manual_name: &str,
) -> Vec<Record> {
    let manual_names = [
        "flamehamster",
        "rumblechirp",
        "verbiage++",
        "guineapigment",
        "emrgency",
    ];
    if manual_names.contains(&manual_name) {
        vectordb
            .search_with_filter(query, topk, ("manual", manual_name))
            .await
    } else {
        vectordb.search(query, topk).await
    }
    .expect("query succeeded")
}

#[derive(Clone, Debug, schemars::JsonSchema, serde::Deserialize)]
struct ConversationData {
    product_name: String,
    expanded_query: String,
}

#[derive(Debug, Embed, serde::Serialize)]
pub struct Record {
    id: String,
    manual: String,
    #[embed]
    content: String,
}

pub async fn populate_vectordb(path: impl AsRef<Path>) -> Result<VectorDb> {
    let needs_init = !path.as_ref().exists();
    let db = VectorDb::try_connect(path).await?;

    if needs_init {
        db.create_table::<Record>().await?;
        let resources = std::path::PathBuf::from("resources/");
        let pattern = fancy_regex::Regex::new(r#"(?mis)^# .+?(?=^# |\Z)"#).unwrap();

        for (manual, manual_src) in [
            ("emgency", "emrgency.md"),
            ("flamehamster", "flamehamster.md"),
            ("guineapigment", "guineapigment.md"),
            ("rumblechirp", "rumblechirp.md"),
            ("verbiage++", "verbiagepp.md"),
        ] {
            let fpath = resources.join(manual_src);
            let content = std::fs::read_to_string(&fpath).expect("markdown read");
            let recs: Vec<Record> = pattern
                .find_iter(&content)
                .into_iter()
                .enumerate()
                .map(|(i, text)| Record {
                    id: format!("{manual}-{i}"),
                    manual: manual.to_string(),
                    content: text.unwrap().as_str().to_string(),
                })
                .collect();

            db.insert(recs).await?;
        }
    }

    Ok(db)
}
