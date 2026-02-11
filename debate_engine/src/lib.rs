pub mod debate;
mod llm;
pub mod simulation;
pub use debate::debate_runner::run_debate;
pub use simulation::engine::{Simulation, SimulationResult};
