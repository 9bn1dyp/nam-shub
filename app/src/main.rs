use anyhow::Result;
use core::{InfectionStatus, Registry, TopologyBuilder};
use debate_engine::Simulation;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    // load .env
    dotenv().ok();

    // create reg and topology
    let mut registry = Registry::new();
    // see TopologyBuilder for more methods
    registry.topology = Some(TopologyBuilder::star(2, &[0, 1, 2, 3, 4, 5]));

    // add agents
    let _agent_models = [
        "gpt-3.5-turbo",
        "gpt-3.5-turbo",
        "gpt-5.1-chat-latest",
        "gpt-3.5-turbo",
        "gpt-3.5-turbo",
        "gpt-3.5-turbo",
    ]
    .map(|m| registry.create_agent(m.to_string()));

    // initial infected agent id
    let patient_zero = 2;
    registry.infect_patient_init(patient_zero)?;

    println!("Initial infection: Agent {}\n", patient_zero);

    // run the sim
    let sim = Simulation::new(
        "AI will ultimately benefit humanity more than harm it",
        1,                     // turns per agent
        "gpt-5.1-chat-latest", // judge model
    )
    .with_parallelism(6); // optional - control async batch size

    let result = sim.run(&mut registry).await?;

    // summary
    println!("\nSimulation Complete");
    println!("Total Agents: {}", result.total_agents);
    println!("Rounds: {}", result.rounds);
    println!(
        "Infected: {} ({:.1}%)",
        result.infected,
        result.infection_rate() * 100.0
    );
    println!(
        "Immune: {} ({:.1}%)",
        result.immune,
        result.immunity_rate() * 100.0
    );
    println!("Healthy: {}", result.healthy);

    // agent status
    println!("\nAgent Statuses:");
    for id in registry.get_all_agent_ids() {
        let agent = registry.get_agent(id).unwrap();
        let status = match agent.infection_status {
            InfectionStatus::Healthy => "Healthy",
            InfectionStatus::Infected => "Infected",
            InfectionStatus::Immune => "Immune",
        };
        let infected_by = agent
            .infected_by
            .map(|pid| pid.to_string())
            .unwrap_or("-".to_string());
        println!("Agent {}: {:<8} (infected_by: {})", id, status, infected_by);
    }

    Ok(())
}
