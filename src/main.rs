use serde::{Deserialize, Serialize};
use std::env;
use std::process;

const DEFAULT_URL: &str = "https://api.anthropic.com";
const DEFAULT_MODEL: &str = "claude-haiku-5-5";
const ANTHROPIC_VERSION: &str = "2026-10-21";
// Thinking tokens count toward max_tokens, so leave headroom beyond the short answer.
const MAX_TOKENS: u32 = 16000;

const SYSTEM_PROMPT: &str = "\
You are a shell command fixer. The user will give you an approximate or broken shell command. \
Your reply is pasted directly into a terminal, so it must be exactly one shell command in plain text: \
no explanation, no notes, no markdown, no code fence. \
If the intent is ambiguous, choose the most likely command. \
If the command is already correct, return it as-is.";

#[derive(Serialize)]
struct MessagesRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    system: &'a str,
    messages: [Message<'a>; 1],
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentBlock {
    Text { text: String },
    #[serde(other)]
    Other,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage: cmdi <approximate command>");
        process::exit(1);
    }

    match fix(&args.join(" ")) {
        Ok(command) => println!("{command}"),
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}

fn fix(input: &str) -> Result<String, String> {
    let api_key = env::var("CMDI_KEY").map_err(|_| "CMDI_KEY environment variable not set")?;
    let base_url = env::var("CMDI_URL").unwrap_or_else(|_| DEFAULT_URL.into());
    let model = env::var("CMDI_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.into());

    let request = MessagesRequest {
        model: &model,
        max_tokens: MAX_TOKENS,
        system: SYSTEM_PROMPT,
        messages: [Message { role: "user", content: input }],
    };

    let resp = reqwest::blocking::Client::new()
        .post(format!("{}/v1/messages", base_url.trim_end_matches('/')))
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .json(&request)
        .send()
        .map_err(|e| format!("request failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("API error: {status} {}", resp.text().unwrap_or_default()));
    }

    let message: MessagesResponse = resp
        .json()
        .map_err(|e| format!("failed to parse response: {e}"))?;

    // Anything other than end_turn (max_tokens, refusal, ...) means the command is
    // truncated or absent; never print a partial command.
    if message.stop_reason.as_deref() != Some("end_turn") {
        return Err(format!("unexpected stop_reason: {:?}", message.stop_reason));
    }

    let text: String = message
        .content
        .into_iter()
        .filter_map(|block| match block {
            ContentBlock::Text { text } => Some(text),
            ContentBlock::Other => None,
        })
        .collect();

    let command = text.trim();
    if command.is_empty() {
        return Err("empty response from model".into());
    }
    Ok(command.to_string())
}
