use crate::agent::{Agent, InfectionStatus};
use crate::debate::DebateOutcome;
use crate::topology::Topology;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Registry {
    // acts as counter for agent_id
    next_agent_id: u32,
    // all agents
    agents: HashMap<u32, Agent>,
    // see topology.rs
    pub topology: Option<Topology>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            next_agent_id: 0,
            agents: HashMap::new(),
            topology: None,
        }
    }

    // append internal counter on creation
    pub fn create_agent(&mut self, model: String) -> u32 {
        let id = self.next_agent_id;
        self.next_agent_id += 1;
        self.agents.insert(id, Agent::new(id, model));
        id
    }

    // get agent (read)
    pub fn get_agent(&self, id: u32) -> Option<&Agent> {
        self.agents.get(&id)
    }

    // get agent (write)
    pub fn get_agent_mut(&mut self, id: u32) -> Option<&mut Agent> {
        self.agents.get_mut(&id)
    }

    pub fn get_all_agent_ids(&self) -> Vec<u32> {
        self.agents.keys().copied().collect()
    }

    // gets all agents in vec (read)
    pub fn get_all_agents(&self) -> Vec<&Agent> {
        self.agents.values().collect()
    }

    // agent count not using counter
    // next_agent_id could be used since counter starts from 0
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    pub fn infected_count(&self) -> usize {
        self.agents.values().filter(|a| a.is_infected()).count()
    }

    pub fn healthy_count(&self) -> usize {
        self.agents.values().filter(|a| a.is_healthy()).count()
    }

    pub fn immune_count(&self) -> usize {
        self.agents.values().filter(|a| a.is_immune()).count()
    }

    pub fn get_infected_agent_ids(&self) -> Vec<u32> {
        self.agents
            .iter()
            .filter(|(_, agent)| agent.is_infected())
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn get_healthy_agent_ids(&self) -> Vec<u32> {
        self.agents
            .iter()
            .filter(|(_, agent)| agent.is_healthy())
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn get_immune_agent_ids(&self) -> Vec<u32> {
        self.agents
            .iter()
            .filter(|(_, agent)| agent.is_immune())
            .map(|(id, _)| *id)
            .collect()
    }

    // calls infect_init() for agents who start with the infection
    pub fn infect_patient_init(&mut self, agent_id: u32) -> Result<(), String> {
        if let Some(agent) = self.agents.get_mut(&agent_id) {
            agent.infection_status = crate::agent::InfectionStatus::Infected;
            agent.infected_by = None;
            Ok(())
        } else {
            Err(format!("Agent {} not found", agent_id))
        }
    }

    // apply debate outcome
    pub fn apply_debate_outcome(
        &mut self,
        proposer_id: u32,
        opposer_id: u32,
        outcome: DebateOutcome,
    ) -> Result<(), String> {
        let opposer = self
            .agents
            .get_mut(&opposer_id)
            .ok_or("opposer not found")?;

        // Apply outcome
        match outcome {
            DebateOutcome::ProposerWon => {
                opposer.infected_by = Some(proposer_id);
                opposer.infection_status = InfectionStatus::Infected;
            }
            DebateOutcome::OpposerWon => {
                opposer.infection_status = InfectionStatus::Immune;
            }
            DebateOutcome::Ongoing => {}
        }

        Ok(())
    }

    // validate debate agents
    pub fn can_debate(&self, proposer_id: u32, opposer_id: u32) -> Result<(), String> {
        // Check both agents exist
        let proposer = self
            .agents
            .get(&proposer_id)
            .ok_or_else(|| format!("proposer {} not found", proposer_id))?;
        let opposer = self
            .agents
            .get(&opposer_id)
            .ok_or_else(|| format!("opposer {} not found", opposer_id))?;

        // Check proposer is infected
        if !proposer.is_infected() {
            return Err(format!("proposer {} is not infected", proposer_id));
        }

        // Check opposer is healthy
        if !opposer.is_healthy() {
            return Err(format!("opposer {} is not healthy", opposer_id));
        }

        // Check topology exist
        let topology = match &self.topology {
            Some(t) => t,
            None => return Err(String::from("Topology does not exist")),
        };

        // Check connection exists
        if !topology.are_connected(proposer_id, opposer_id) {
            return Err(format!(
                "Agents {} and {} are not connected",
                proposer_id, opposer_id
            ));
        }

        Ok(())
    }

    // get healthy agents connected to a given id
    pub fn get_potential_targets(&self, infector_id: u32) -> Vec<u32> {
        let topology = match &self.topology {
            Some(t) => t,
            None => return Vec::new(),
        };

        topology
            .get_neighbors(infector_id)
            .into_iter()
            .filter(|id| self.agents.get(id).map(|a| a.is_healthy()).unwrap_or(false))
            .collect()
    }

    // registry stats, return struct has other methods
    pub fn get_statistics(&self) -> RegistryStatistics {
        RegistryStatistics {
            total_agents: self.agent_count(),
            infected_agents: self.infected_count(),
            healthy_agents: self.healthy_count(),
            immune_agents: self.immune_count(),
            total_connections: if let Some(topology) = &self.topology {
                topology.connection_count()
            } else {
                0
            },
        }
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

// registry stats and methods
#[derive(Debug, Clone)]
pub struct RegistryStatistics {
    pub total_agents: usize,
    pub infected_agents: usize,
    pub healthy_agents: usize,
    pub immune_agents: usize,
    pub total_connections: usize,
}

impl RegistryStatistics {
    pub fn infection_rate(&self) -> f64 {
        if self.total_agents == 0 {
            0.0
        } else {
            self.infected_agents as f64 / self.total_agents as f64
        }
    }

    pub fn immunity_rate(&self) -> f64 {
        if self.total_agents == 0 {
            0.0
        } else {
            self.immune_agents as f64 / self.total_agents as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = Registry::default();
        assert_eq!(registry.agent_count(), 0);
        assert_eq!(registry.infected_count(), 0);
    }

    #[test]
    fn test_create_agent() {
        let mut registry = Registry::default();
        let agent = registry.create_agent("model".to_string());

        assert_eq!(registry.agent_count(), 1);
        assert!(registry.get_agent(agent).is_some());

        let agent2 = registry.create_agent("model".to_string());

        assert_eq!(registry.agent_count(), 2);
        assert!(registry.get_agent(agent2).is_some());
    }

    #[test]
    fn test_patient_init() {
        let mut registry = Registry::default();
        let agent = registry.create_agent("model".to_string());

        registry.infect_patient_init(agent).unwrap();

        assert_eq!(registry.infected_count(), 1);
        assert_eq!(registry.healthy_count(), 0);
    }

    #[test]
    fn test_apply_debate_outcome_infection() {
        let mut registry = Registry::default();
        let agent_a = registry.create_agent("model".to_string());
        let agent_b = registry.create_agent("model".to_string());

        // add topology
        let mut topology = Topology::new();
        topology.add_connection(agent_a, agent_b);
        registry.topology = Some(topology);

        registry.infect_patient_init(agent_a).unwrap();

        registry
            .apply_debate_outcome(agent_a, agent_b, DebateOutcome::ProposerWon)
            .unwrap();

        assert!(registry.get_agent(agent_b).unwrap().is_infected());
        assert_eq!(
            registry.get_agent(agent_b).unwrap().infected_by,
            Some(agent_a)
        );
    }

    #[test]
    fn test_apply_debate_outcome_immunity() {
        let mut registry = Registry::default();
        let agent_a = registry.create_agent("model".to_string());
        let agent_b = registry.create_agent("model".to_string());

        // add topology
        let mut topology = Topology::new();
        topology.add_connection(agent_a, agent_b);
        registry.topology = Some(topology);

        registry.infect_patient_init(agent_a).unwrap();

        registry
            .apply_debate_outcome(agent_a, agent_b, DebateOutcome::OpposerWon)
            .unwrap();

        assert!(registry.get_agent(agent_b).unwrap().is_immune());
    }

    #[test]
    fn test_can_debate_validation() {
        let mut registry = Registry::default();
        let agent_a = registry.create_agent("model".to_string());
        let agent_b = registry.create_agent("model".to_string());

        // add topology
        let mut topology = Topology::new();
        topology.add_connection(agent_a, agent_b);
        registry.topology = Some(topology);

        // Should fail - proposer not infected
        assert!(registry.can_debate(agent_a, agent_b).is_err());

        registry.infect_patient_init(agent_a).unwrap();

        // Should succeed
        assert!(registry.can_debate(agent_a, agent_b).is_ok());
    }

    #[test]
    fn test_can_debate_not_connected() {
        let mut registry = Registry::default();
        let agent_a = registry.create_agent("model".to_string());
        let agent_b = registry.create_agent("model".to_string());

        registry.infect_patient_init(agent_a).unwrap();

        // Should fail - not connected
        assert!(registry.can_debate(agent_a, agent_b).is_err());
    }

    #[test]
    fn test_get_potential_targets() {
        let mut registry = Registry::default();
        let agent_a = registry.create_agent("model".to_string());
        let agent_b = registry.create_agent("model".to_string());
        let agent_c = registry.create_agent("model".to_string());

        // add topology
        let mut topology = Topology::new();
        topology.add_connection(agent_a, agent_b);
        topology.add_connection(agent_a, agent_c);
        registry.topology = Some(topology);

        registry.infect_patient_init(agent_a).unwrap();

        let targets = registry.get_potential_targets(agent_a);
        assert_eq!(targets.len(), 2);
        assert!(targets.contains(&agent_b));
        assert!(targets.contains(&agent_c));
    }

    #[test]
    fn test_statistics() {
        let mut registry = Registry::default();
        let agent_a = registry.create_agent("model".to_string());
        let _agent_b = registry.create_agent("model".to_string());
        let _agent_c = registry.create_agent("model".to_string());

        registry.infect_patient_init(agent_a).unwrap();

        let stats = registry.get_statistics();
        assert_eq!(stats.total_agents, 3);
        assert_eq!(stats.infected_agents, 1);
        assert_eq!(stats.healthy_agents, 2);
        assert_eq!(stats.immune_agents, 0);
    }
}
