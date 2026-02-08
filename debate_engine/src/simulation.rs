use crate::run_debate::run_debate;
use anyhow::Result;
use core::{DebateOutcome, Registry};
use std::collections::{HashSet, VecDeque};

pub struct Simulation {
    pub topic: String,
    pub max_turns: usize,
    pub judge_model: String,
    pub verbose: bool,
}

impl Simulation {
    pub fn new(
        topic: impl Into<String>,
        max_turns: usize,
        judge_model: impl Into<String>,
        verbose: bool,
    ) -> Self {
        Self {
            topic: topic.into(),
            max_turns,
            judge_model: judge_model.into(),
            verbose,
        }
    }

    // run the simulation on a registry
    pub async fn run(&self, registry: &mut Registry) -> Result<SimulationResult> {
        let mut debates = Vec::new();
        let mut visited_edges = HashSet::new();

        // all currently infected agents
        let mut frontier: VecDeque<u32> = registry.get_infected_agent_ids().into();

        if self.verbose {
            println!("Saturation simulation started");
            println!("Topic: {}", self.topic);
            println!("Initial infected: {:?}", frontier);
        }

        // allows us to target any agent connected to already infected agent
        while let Some(proposer_id) = frontier.pop_front() {
            let targets = registry.get_potential_targets(proposer_id);

            for opposer_id in targets {
                let edge = (proposer_id, opposer_id);
                if visited_edges.contains(&edge) {
                    continue;
                }
                visited_edges.insert(edge);

                if self.verbose {
                    let proposer_model = &registry.get_agent(proposer_id).unwrap().model;
                    let opposer_model = &registry.get_agent(opposer_id).unwrap().model;
                    println!(
                        "\nAgent {} ({}) vs Agent {} ({})",
                        proposer_id, proposer_model, opposer_id, opposer_model
                    );
                }

                let debate = run_debate(
                    registry,
                    proposer_id,
                    opposer_id,
                    &self.topic,
                    self.max_turns,
                    &self.judge_model,
                    self.verbose,
                )
                .await?;

                // apply debate outcome to register
                registry
                    .apply_debate_outcome(proposer_id, opposer_id, debate.outcome)
                    .map_err(|e| anyhow::anyhow!(e))?;

                // update frontier based on outcome
                match debate.outcome {
                    DebateOutcome::ProposerWon => {
                        if self.verbose {
                            println!("Agent {} infected!", opposer_id);
                        }
                        frontier.push_back(opposer_id);
                    }
                    DebateOutcome::OpposerWon => {
                        if self.verbose {
                            println!("Agent {} immune!", opposer_id);
                        }
                    }
                    _ => {}
                }

                debates.push(debate);
            }
        }

        let stats = registry.get_statistics();

        if self.verbose {
            println!("\nSimulation complete");
            println!(
                "Infected: {}/{} ({:.1}%)",
                stats.infected_agents,
                stats.total_agents,
                stats.infection_rate() * 100.0
            );
        }

        Ok(SimulationResult {
            rounds: visited_edges.len(), // edges tried
            total_agents: stats.total_agents,
            infected: stats.infected_agents,
            healthy: stats.healthy_agents,
            immune: stats.immune_agents,
            debates,
        })
    }
}

#[derive(Debug)]
pub struct SimulationResult {
    pub rounds: usize,
    pub total_agents: usize,
    pub infected: usize,
    pub healthy: usize,
    pub immune: usize,
    pub debates: Vec<core::Debate>,
}

impl SimulationResult {
    pub fn infection_rate(&self) -> f64 {
        if self.total_agents == 0 {
            0.0
        } else {
            self.infected as f64 / self.total_agents as f64
        }
    }

    pub fn immunity_rate(&self) -> f64 {
        if self.total_agents == 0 {
            0.0
        } else {
            self.immune as f64 / self.total_agents as f64
        }
    }
}
