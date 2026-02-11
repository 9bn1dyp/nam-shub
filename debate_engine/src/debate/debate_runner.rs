use anyhow::Result;
use core::{Debate, Exchange, Message};
use genai::Client;
use genai::chat::ChatMessage;

use crate::llm::{judge_debate, send_message};

pub async fn run_debate(
    proposer_id: u32,
    opposer_id: u32,
    proposer_model: &str,
    opposer_model: &str,
    topic: &str,
    max_turns: usize,
    judge_model: &str,
) -> Result<Debate> {
    // create genai client
    let client = Client::default();
    // init new debate struct
    let mut debate = Debate::new(proposer_id, opposer_id, max_turns);

    // context
    let proposer_system = format!(
        "You are debating: '{}'. Your role is PROPOSITION. Be persuasive and logical.",
        topic
    );
    let opposer_system = format!(
        "You are debating: '{}'. Your role is OPPOSITION. Be persuasive and logical.",
        topic
    );

    // local history
    let mut proposer_history = vec![ChatMessage::system(&proposer_system)];
    let mut opposer_history = vec![ChatMessage::system(&opposer_system)];
    // local message id
    let mut message_id = 0;

    // 1 turn = 1 proposer message and 1 opposer response
    for turn in 0..max_turns {
        let (proposer_response, opposer_response) = run_round(
            &client,
            proposer_model,
            opposer_model,
            &mut proposer_history,
            &mut opposer_history,
            topic,
            turn,
        )
        .await?;

        // add exchange to the debate
        debate.add_exchange(Exchange {
            proposer: Message {
                id: message_id,
                message: proposer_response,
            },
            opposer: Message {
                id: message_id + 1,
                message: opposer_response,
            },
        });

        message_id += 2;
    }

    // have another model judge the outcome of the interaction
    let outcome = judge_debate(&client, judge_model, topic, &debate.exchanges).await?;
    debate.set_outcome(outcome);
    // return updated debate
    Ok(debate)
}

// priv func
async fn run_round(
    client: &Client,
    proposer_model: &str,
    opposer_model: &str,
    proposer_history: &mut Vec<ChatMessage>,
    opposer_history: &mut Vec<ChatMessage>,
    topic: &str,
    turn: usize,
) -> Result<(String, String)> {
    // more context
    let prompt = if turn == 0 {
        format!("Make your opening argument for: '{}'", topic)
    } else {
        "Continue your argument. Address opponent's points.".to_string()
    };

    // push proposer history
    proposer_history.push(ChatMessage::user(&prompt));
    let proposer_response = send_message(client, proposer_model, proposer_history).await?;
    proposer_history.push(ChatMessage::assistant(&proposer_response));

    // push opposer history
    opposer_history.push(ChatMessage::user(format!(
        "PROPOSITION said: '{}'\n\nRespond and defend your position.",
        proposer_response
    )));

    let opposer_response = send_message(client, opposer_model, opposer_history).await?;
    opposer_history.push(ChatMessage::assistant(&opposer_response));

    // return both responses
    Ok((proposer_response, opposer_response))
}
