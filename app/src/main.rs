use anyhow::Result;
use core::{InfectionStatus, Registry, TopologyBuilder};
use debate_engine::Simulation;
use dotenv::dotenv;
use visualizer::visualize_graph;

/// Debate Simulation - AI Agent Network
///
/// This simulation models how AI agents with different LLM's, try to infect each other via debate.
/// - Agents are connected in a network topology
/// - "Infected" agents debate "healthy" agents
/// - Debates are judged by an LLM to determine the outcome
/// - Winners become "immune", losers become "infected" and can spread further
///
/// Requirements:
/// - Create a `.env` file with your API keys (see .env.example)
/// - Set OPENAI_API_KEY or other provider keys for the genai crate

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // ============================================================================
    // Step 1: Create Registry (Agent Manager)
    // ============================================================================
    let mut registry = Registry::new();

    // ============================================================================
    // Step 2: Define Network Topology
    // ============================================================================
    // The topology determines which agents can debate each other.
    // Available topology types:
    //
    // - Fully Connected: Every agent connected to every other agent
    //   TopologyBuilder::fully_connected(&[0, 1, 2, 3, 4, 5])
    //
    // - Ring: Agents form a circular chain
    //   TopologyBuilder::ring(&[0, 1, 2, 3, 4, 5])
    //
    // - Star: One central agent connected to all peripheral agents
    //   TopologyBuilder::star(2, &[0, 1, 3, 4, 5])
    //
    // - Random: Probabilistic connections between agents
    //   TopologyBuilder::random(&[0, 1, 2, 3, 4, 5], 0.5)

    let agent_ids = vec![0, 1, 2, 3, 4, 5];
    registry.topology = Some(TopologyBuilder::star(2, &agent_ids));

    // ============================================================================
    // Step 3: Create Agents with AI Models
    // ============================================================================
    // Each agent uses an LLM to debate. You can mix different models to see
    // how model capabilities affect debate outcomes.
    //
    // Supported models (via genai crate):
    // - OpenAI: "gpt-5.2-chat-latest", "gpt-3.5-turbo", etc.
    // - Anthropic: "claude-3-opus-20240229", etc.
    // - Other providers supported by genai crate

    let agent_models = vec![
        "gpt-3.5-turbo",       // Agent 0
        "gpt-3.5-turbo",       // Agent 1
        "gpt-5.2-chat-latest", // Agent 2 (center - stronger model)
        "gpt-3.5-turbo",       // Agent 3
        "gpt-3.5-turbo",       // Agent 4
        "gpt-3.5-turbo",       // Agent 5
    ];

    for model in agent_models {
        registry.create_agent(model.to_string());
    }

    // ============================================================================
    // Step 4: Set Initial Infection (Patient Zero)
    // ============================================================================
    // The "infected" agent is the initial proposer in the debate topic.
    // This agent will attempt to convince connected healthy agents.
    // Once a healthy agents becomes infected, it will start infecting as well.

    let patient_zero = 2;
    registry.infect_patient_init(patient_zero)?;

    // ============================================================================
    // Step 5: Configure and Run Simulation
    // ============================================================================
    // Simulation parameters:
    // - topic: The debate proposition
    // - max_turns: Number of back and forth exchanges per debate
    // - judge_model: Judge LLM that evaluates who won each debate
    //
    // Simulation methods
    // - .with_parallelism(usize): Optionally set batch size to run asynchronously
    // - .run(&mut Registry): Run the simulation

    let topic = "Does pineapple belong on pizza";
    let turns_per_debate = 2;
    let judge_model = "gpt-5.2-chat-latest";
    let max_parallel_debates = 5;

    let sim = Simulation::new(topic, turns_per_debate, judge_model)
        .with_parallelism(max_parallel_debates);

    let result = sim.run(&mut registry).await?;

    // ============================================================================
    // Step 6: Display Results
    // ============================================================================
    println!("Simulation Statistics:");
    println!("  Total Agents:    {}", result.total_agents);
    println!("  Total Debates:   {}", result.rounds);
    println!(
        "  Infected count:  {} ({:.1}%)",
        result.infected,
        result.infection_rate() * 100.0
    );
    println!(
        "  Immune count: {} ({:.1}%)",
        result.immune,
        result.immunity_rate() * 100.0
    );
    println!(
        "  Healthy count: {} ({:.1}%)",
        result.immune,
        result.healthy_rate() * 100.0
    );

    for id in registry.get_all_agent_ids() {
        let agent = registry.get_agent(id).unwrap();

        let status_display = match agent.infection_status {
            InfectionStatus::Healthy => " Healthy",
            InfectionStatus::Infected => " Infected",
            InfectionStatus::Immune => " Immune",
        };

        let infected_by = agent
            .infected_by
            .map(|pid| format!("Agent {}", pid))
            .unwrap_or_else(|| "-".to_string());

        let model_display = agent.model.clone();

        println!(
            "{} {} {} {}",
            id, status_display, model_display, infected_by
        );
    }

    // ============================================================================
    // Step 7: Generate graph
    // ============================================================================

    visualize_graph(&registry)?;

    Ok(())
}
