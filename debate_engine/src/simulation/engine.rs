use crate::run_debate;
use anyhow::Result;
use core::{Debate, DebateOutcome, Registry};
use std::collections::{HashSet, VecDeque};
use tokio::task::JoinSet;

/// High-level simulation orchestrator
pub struct Simulation {
    pub topic: String,
    pub max_turns: usize,
    pub judge_model: String,
    pub max_parallel_debates: usize,
}

impl Simulation {
    pub fn new(topic: impl Into<String>, max_turns: usize, judge_model: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            max_turns,
            judge_model: judge_model.into(),
            max_parallel_debates: 4,
        }
    }

    pub fn with_parallelism(mut self, max_parallel: usize) -> Self {
        self.max_parallel_debates = max_parallel;
        self
    }

    // sim loop
    pub async fn run(&self, registry: &mut Registry) -> Result<SimulationResult> {
        // get all infected agents in the registry
        let mut infected_deque: VecDeque<u32> = registry.get_infected_agent_ids().into();

        let mut all_debates = Vec::new();

        // loop suntil
        while !infected_deque.is_empty() {
            // create a batch of infected--healthy edges, len constrainted by max_parallel
            let batch = self.build_debate_batch(registry, &infected_deque);

            // if batch is empty, check for every id they still have targets else remove
            if batch.is_empty() {
                infected_deque.retain(|&id| !registry.get_potential_targets(id).is_empty());
                continue;
            }

            // run the batch async
            let debates = self.run_debate_batch(registry, &batch).await?;

            self.apply_batch_results(registry, debates, &mut infected_deque, &mut all_debates)?;
        }

        // finalize results
        Ok(self.finalize(registry, all_debates))
    }

    // create batch of debates to be ran async
    fn build_debate_batch(
        &self,
        registry: &Registry,
        infected_deque: &VecDeque<u32>,
    ) -> Vec<(u32, u32)> {
        let mut batch = Vec::new();
        let mut used_opposers = HashSet::new();

        // for infected agents in infected_deque
        for &proposer_id in infected_deque {
            // for targets connected to infected agent
            for opposer_id in registry.get_potential_targets(proposer_id) {
                // create edge for each target
                let edge = (proposer_id, opposer_id);

                // skip if already used previously
                if used_opposers.contains(&opposer_id) {
                    continue;
                }

                // add to current batch, current used list
                batch.push(edge);
                used_opposers.insert(opposer_id);

                // if reaches constraint return batch
                if batch.len() >= self.max_parallel_debates {
                    return batch;
                }
            }
        }

        batch
    }

    async fn run_debate_batch(
        &self,
        registry: &Registry,
        pairs: &[(u32, u32)],
    ) -> Result<Vec<Debate>> {
        // container for async tasks
        let mut tasks = JoinSet::new();

        // iterate over edges in batch
        for &(proposer_id, opposer_id) in pairs {
            let max_turns = self.max_turns;

            // clone none copy types
            let topic = self.topic.clone();
            let judge_model = self.judge_model.clone();

            // get agents model from registry
            let proposer_model = registry.get_agent(proposer_id).unwrap().model.clone();
            let opposer_model = registry.get_agent(opposer_id).unwrap().model.clone();

            // spawn task for each edge
            tasks.spawn(async move {
                run_debate(
                    proposer_id,
                    opposer_id,
                    &proposer_model,
                    &opposer_model,
                    &topic,
                    max_turns,
                    &judge_model,
                )
                .await
            });
        }

        // result vec for return
        let mut results = Vec::new();
        // wait for any task to finish then
        while let Some(result) = tasks.join_next().await {
            // result is Result<Result<Debate, anyhow::Error>, JoinError> here
            results.push(result??);
        }

        Ok(results)
    }

    fn apply_batch_results(
        &self,
        registry: &mut Registry,
        debates: Vec<Debate>,
        infected_deque: &mut VecDeque<u32>,
        all_debates: &mut Vec<Debate>,
    ) -> Result<()> {
        for debate in debates.into_iter() {
            // update registry of outcome, this changes infectionstatus on agents
            registry.apply_debate_outcome(debate.proposer_id, debate.opposer_id, debate.outcome)?;

            // if agent lost debate (becomes infected) and not already in infected_deque, add to fontier
            if debate.outcome == DebateOutcome::ProposerWon
                && !infected_deque.contains(&debate.opposer_id)
            {
                infected_deque.push_back(debate.opposer_id);
            }

            // push debate
            all_debates.push(debate);
        }

        Ok(())
    }

    fn finalize(&self, registry: &Registry, debates: Vec<Debate>) -> SimulationResult {
        let stats = registry.get_statistics();

        SimulationResult {
            rounds: debates.len(),
            total_agents: stats.total_agents,
            infected: stats.infected_agents,
            healthy: stats.healthy_agents,
            immune: stats.immune_agents,
            debates,
        }
    }
}

/// Returned to callers (app crates)
#[derive(Debug)]
pub struct SimulationResult {
    pub rounds: usize,
    pub total_agents: usize,
    pub infected: usize,
    pub healthy: usize,
    pub immune: usize,
    pub debates: Vec<Debate>,
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

    pub fn healthy_rate(&self) -> f64 {
        if self.total_agents == 0 {
            0.0
        } else {
            self.healthy as f64 / self.total_agents as f64
        }
    }
}
