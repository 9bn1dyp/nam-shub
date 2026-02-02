use crate::debate::Debate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum InfectionStatus {
    #[default]
    Healthy, // hasnt debated
    Infected, // lost debate
    Immune,   // won debate
}

#[derive(Debug, Clone)]
pub struct Agent<'debate> {
    pub id: u32,
    // ai model agent uses todo!
    pub model: String,
    pub infection_status: InfectionStatus,
    pub debate_history: Vec<&'debate Debate>,
    pub infected_by: Option<u32>,
}

impl<'debate> Agent<'debate> {
    pub fn new(id: u32, model: String) -> Self {
        Self {
            id,
            model,
            infection_status: InfectionStatus::default(),
            debate_history: Vec::new(),
            infected_by: None,
        }
    }

    pub fn is_infected(&self) -> bool {
        self.infection_status == InfectionStatus::Infected
    }

    pub fn is_healthy(&self) -> bool {
        self.infection_status == InfectionStatus::Healthy
    }

    pub fn is_immune(&self) -> bool {
        self.infection_status == InfectionStatus::Immune
    }

    // self infect called when debate is lost
    fn infect(&mut self, infected_by: u32) {
        self.infection_status = InfectionStatus::Infected;
        self.infected_by = Some(infected_by);
    }

    // self immune called when debate is won
    fn immune(&mut self) {
        self.infection_status = InfectionStatus::Immune;
    }

    // log previous debate to history
    pub fn add_debate(&mut self, debate: &'debate Debate) {
        self.debate_history.push(debate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new(0, "model".to_string());

        assert!(!agent.is_infected());
        assert!(agent.is_healthy());
        assert!(!agent.is_immune());
        assert_eq!(agent.id, 0);
        assert_eq!(agent.model, "model");
    }

    #[test]
    fn test_agent_infection() {
        let mut agent = Agent::new(0, "model".to_string());
        let infector_id = 1;

        agent.infect(infector_id);

        assert!(agent.is_infected());
        assert!(!agent.is_healthy());
        assert_eq!(agent.infected_by, Some(infector_id));
    }

    #[test]
    fn test_agent_immunity() {
        let mut agent = Agent::new(0, "model".to_string());

        agent.immune();

        assert!(agent.is_immune());
        assert!(!agent.is_healthy());
        assert!(!agent.is_infected());
    }

    #[test]
    fn test_infection_status_default() {
        assert_eq!(InfectionStatus::default(), InfectionStatus::Healthy);
    }

    #[test]
    fn test_debate_history() {
        let mut agent = Agent::new(0, "model".to_string());
        let debate = crate::debate::Debate::new(0, 1, 2);

        agent.add_debate(&debate);

        assert_eq!(agent.debate_history.len(), 1);
        assert_eq!(agent.debate_history[0], &debate);
    }
}
