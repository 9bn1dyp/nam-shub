pub mod agent;
pub mod debate;
pub mod registry;
pub mod topology;

pub use agent::{Agent, InfectionStatus};
pub use debate::{Debate, DebateOutcome};
pub use registry::{Registry, RegistryStatistics};
pub use topology::{Topology, TopologyBuilder};

#[cfg(test)]
mod tests {
    use super::*;
}
