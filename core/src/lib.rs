pub mod agent;
pub mod debate;
pub mod registry;
pub mod topology;

pub use agent::{Agent, InfectionStatus};
pub use debate::{Debate, DebateOutcome, Exchange, Message};
pub use registry::{Registry, RegistryStatistics};
pub use topology::{Topology, TopologyBuilder};
