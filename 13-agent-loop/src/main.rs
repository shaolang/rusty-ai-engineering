use helpers::{Args, History, Result, create_openai_client, extract_texts, input};
use openai_oxide::types::responses::Response;
use serde_json::Value;
use tools_api::{Tool, new_request, read_webpage, tools};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = create_openai_client(&args)?;
    let tools = tools();
    let mut history = History::new(
        "
        You are a helpful AI assistant. If you ever need to multiply two numbers, DO NOT attempt
        to answer with your internal knowledge. Instead, use your multiply tool."
            .into(),
    );
    let mut assistant_msg = "How can I help you today?".to_string();
    let mut user_input = input(format!("Assistant: {assistant_msg}\n\nUser: "));

    while &user_input != "exit" {
        history.add_assistant_msg(assistant_msg);
        history.add_user_msg(user_input);
        let mut resp = client
            .responses()
            .create(new_request(&args, &history, &tools))
            .await?;

        let mut tool_calls = get_function_calls(&resp);
        loop {
            let Some((fn_call, call_id, tool)) = tool_calls.pop_front() else {
                break;
            };

            let result = match tool {
                Tool::Multiply {
                    first_number,
                    second_number,
                } => serde_json::json!(first_number * second_number),
                Tool::ReadWebpage { url } => serde_json::Value::String(read_webpage(url).await),
            };
            history.add_function_call_msg(fn_call);
            history.add_function_call_output(&call_id, format!("{result}"));
            let req = new_request(&args, &history, &tools);
            resp = client.responses().create(req).await?;
            tool_calls.append(&mut get_function_calls(&resp));
        }

        assistant_msg = extract_texts(&resp.output, true);
        user_input = input(format!("Assistant: {}\n\nUser: ", assistant_msg.trim()));
    }

    Ok(())
}

fn get_function_calls(resp: &Response) -> std::collections::VecDeque<(Value, String, Tool)> {
    resp.function_calls()
        .iter()
        .map(|r| {
            let json = serde_json::json!({r.name.clone(): r.arguments});
            let tool: Tool = serde_json::from_value(json).unwrap();
            let args = serde_json::to_string(&r.arguments).unwrap();
            let fn_call = serde_json::json!({
                "type": "function_call",
                "call_id": r.call_id,
                "name": r.name,
                "arguments": args,
            });
            (fn_call, r.call_id.clone(), tool)
        })
        .collect()
}
