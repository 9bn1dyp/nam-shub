use anyhow::Result;
use debate_engine::{Registry, Simulation, TopologyBuilder};
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // create topology and registry
    let mut registry = Registry::new();
    registry.topology = Some(TopologyBuilder::star(2, &[0, 1, 2, 3, 4]));

    // create agents
    registry.create_agent("gpt-3.5-turbo".to_string());
    registry.create_agent("gpt-3.5-turbo".to_string());
    registry.create_agent("gpt-5.1-chat-latest".to_string());
    registry.create_agent("gpt-3.5-turbo".to_string());
    registry.create_agent("gpt-3.5-turbo".to_string());

    print!("{:?}", registry.topology);

    // infect initial patient
    registry.infect_patient_init(2).unwrap();

    // start sim
    let sim = Simulation::new(
        "AI will ultimately benefit humanity more than harm it",
        2,                     // turns per agent
        "gpt-5.1-chat-latest", // judge model
        true,                  // false to skip outputs
    );

    let result = sim.run(&mut registry).await?;

    println!(
        "\n📊 Final: {:.1}% infected",
        result.infection_rate() * 100.0
    );

    Ok(())
}
