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
