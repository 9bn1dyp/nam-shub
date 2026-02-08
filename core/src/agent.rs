#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InfectionStatus {
    #[default]
    Healthy, // hasnt debated
    Infected, // lost debate
    Immune,   // won debate
}

#[derive(Debug, Clone)]
pub struct Agent {
    pub id: u32,
    // ai model agent uses todo!
    pub model: String,
    pub infection_status: InfectionStatus,
    pub infected_by: Option<u32>,
}

impl Agent {
    pub fn new(id: u32, model: String) -> Self {
        Self {
            id,
            model,
            infection_status: InfectionStatus::default(),
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
    fn test_infection_status_default() {
        assert_eq!(InfectionStatus::default(), InfectionStatus::Healthy);
    }
}
