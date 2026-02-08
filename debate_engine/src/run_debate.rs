use anyhow::Result;
use genai::Client;
use genai::chat::{ChatMessage, ChatRequest};

use core::{Debate, DebateOutcome, Exchange, Message, Registry};

// run a debate between two agents
pub async fn run_debate(
    registry: &Registry,
    proposer_id: u32,
    opposer_id: u32,
    topic: &str,
    max_turns: usize,
    judge_model: &str,
    verbose: bool,
) -> Result<Debate> {
    // check if debate is possible
    registry
        .can_debate(proposer_id, opposer_id)
        .map_err(|e| anyhow::anyhow!(e))?;

    // genai client, api framework
    let client = Client::default();

    // get agent models
    let proposer_model = &registry
        .get_agent(proposer_id)
        .ok_or_else(|| anyhow::anyhow!("proposer {} not found", proposer_id))?
        .model;
    let opposer_model = &registry
        .get_agent(opposer_id)
        .ok_or_else(|| anyhow::anyhow!("opposer {} not found", opposer_id))?
        .model;

    let mut debate = Debate::new(proposer_id, opposer_id, max_turns);
    let mut message_id = 0u32;

    if verbose {
        println!("\n--- Debate: {} vs {} ---", proposer_model, opposer_model);
        println!("Topic: {}\n", topic);
    }

    // prompts
    let proposer_system = format!(
        "You are debating: '{}'. Your role is PROPOSITION. Be persuasive and logical.",
        topic
    );
    let opposer_system = format!(
        "You are debating: '{}'. Your role is OPPOSITION. Be persuasive and logical.",
        topic
    );

    let mut proposer_history = vec![ChatMessage::system(&proposer_system)];
    let mut opposer_history = vec![ChatMessage::system(&opposer_system)];

    // debate rounds
    for turn in 0..max_turns {
        if verbose {
            println!("--- Round {} ---", turn + 1);
        }

        // proposers turn
        let prompt = if turn == 0 {
            format!("Make your opening argument for: '{}'", topic)
        } else {
            "Continue your argument. Address opponent's points.".to_string()
        };
        proposer_history.push(ChatMessage::user(&prompt));

        let proposer_response = send_message(&client, proposer_model, &proposer_history).await?;
        if verbose {
            println!("PROPOSITION: {}", proposer_response);
        }
        proposer_history.push(ChatMessage::assistant(&proposer_response));

        // opposers turn
        opposer_history.push(ChatMessage::user(format!(
            "PROPOSITION said: '{}'\n\nRespond and defend your position.",
            proposer_response
        )));

        let opposer_response = send_message(&client, opposer_model, &opposer_history).await?;
        if verbose {
            println!("OPPOSITION: {}\n", opposer_response);
        }
        opposer_history.push(ChatMessage::assistant(&opposer_response));

        // record exchange
        debate.add_exchange(Exchange {
            proposer: Message {
                id: message_id,
                message: proposer_response.clone(),
            },
            opposer: Message {
                id: message_id + 1,
                message: opposer_response,
            },
        });
        message_id += 2;
    }

    // get judge outcome
    let outcome = judge_debate(&client, judge_model, topic, &debate.exchanges, verbose).await?;
    debate.set_outcome(outcome);

    if verbose {
        println!("Decision: {}\n", outcome);
    }

    Ok(debate)
}

// send message to a model
async fn send_message(client: &Client, model: &str, messages: &[ChatMessage]) -> Result<String> {
    let chat_req = ChatRequest::new(messages.to_vec());
    let chat_res = client.exec_chat(model, chat_req, None).await?;

    chat_res
        .first_text()
        .ok_or_else(|| anyhow::anyhow!("No response from model"))
        .map(|s| s.to_string())
}

// judge who won the debate
async fn judge_debate(
    client: &Client,
    judge_model: &str,
    topic: &str,
    exchanges: &[Exchange],
    verbose: bool,
) -> Result<DebateOutcome> {
    if verbose {
        println!("=== Judging ===");
    }

    let mut messages = vec![ChatMessage::system(
        "Evaluate this debate. Respond with EXACTLY:\n\
         WINNER: PROPOSITION\nor\nWINNER: OPPOSITION",
    )];

    // build transcript
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

    let response = send_message(client, judge_model, &messages).await?;

    let outcome = if response.contains("PROPOSITION") {
        DebateOutcome::ProposerWon
    } else if response.contains("OPPOSITION") {
        DebateOutcome::OpposerWon
    } else {
        anyhow::bail!("Invalid judge response: {}", response)
    };

    Ok(outcome)
}
