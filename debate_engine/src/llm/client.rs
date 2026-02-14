use anyhow::Result;
use core::{DebateOutcome, Exchange};
use genai::Client;
use genai::chat::{ChatMessage, ChatRequest};

// genai send message helper
pub async fn send_message(
    client: &Client,
    model: &str,
    messages: &[ChatMessage],
) -> Result<String> {
    let chat_req = ChatRequest::new(messages.to_vec());
    let chat_res = client.exec_chat(model, chat_req, None).await?;

    chat_res
        .first_text()
        .ok_or_else(|| anyhow::anyhow!("No response from model"))
        .map(|s| s.to_string())
}

// judge debate
pub async fn judge_debate(
    client: &Client,
    judge_model: &str,
    topic: &str,
    exchanges: &[Exchange],
) -> Result<DebateOutcome> {
    let mut messages = vec![ChatMessage::system(
        "Evaluate this debate. Respond with EXACTLY:\n\
         WINNER: PROPOSITION\nor\nWINNER: OPPOSITION",
    )];

    let mut transcript = format!("Topic: {}\n\n", topic);
    for (i, exchange) in exchanges.iter().enumerate() {
        transcript.push_str(&format!(
            "Round {}:\nPROPOSITION: {}\nOPPOSITION: {}\n\n",
            i + 1,
            exchange.proposer.message,
            exchange.opposer.message
        ));
    }

    messages.push(ChatMessage::user(&transcript));
    messages.push(ChatMessage::user("Who won?"));
    messages.push(ChatMessage::user(
        "Respond with only one word 'OPPOSITION' OR 'PROPOSITION'",
    ));

    let response = send_message(client, judge_model, &messages).await?;

    if response.contains("PROPOSITION") {
        Ok(DebateOutcome::ProposerWon)
    } else if response.contains("OPPOSITION") {
        Ok(DebateOutcome::OpposerWon)
    } else {
        anyhow::bail!("Invalid judge response: {}", response)
    }
}
