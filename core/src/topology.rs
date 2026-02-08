use rand::Rng;
use std::collections::{HashMap, HashSet};

// network topology of agents
#[derive(Debug, Clone)]
pub struct Topology {
    connections: HashMap<u32, HashSet<u32>>,
}

impl Topology {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    pub fn add_connection(&mut self, agent_a: u32, agent_b: u32) {
        // cant connect with self
        if agent_a == agent_b {
            return;
        }

        // avoid duplicate conn
        if self.are_connected(agent_a, agent_b) {
            return;
        }

        self.connections.entry(agent_a).or_default().insert(agent_b);
        self.connections.entry(agent_b).or_default().insert(agent_a);
    }

    // remove connection
    pub fn remove_connection(&mut self, agent_a: u32, agent_b: u32) {
        if let Some(neighbors) = self.connections.get_mut(&agent_a) {
            neighbors.remove(&agent_b);
        }
        if let Some(neighbors) = self.connections.get_mut(&agent_b) {
            neighbors.remove(&agent_a);
        }
    }

    // check if connected
    pub fn are_connected(&self, agent_a: u32, agent_b: u32) -> bool {
        self.connections
            .get(&agent_a)
            .map(|neighbors| neighbors.contains(&agent_b))
            .unwrap_or(false)
    }

    // retrieve all neighbors for an agent
    pub fn get_neighbors(&self, agent_id: u32) -> Vec<u32> {
        self.connections
            .get(&agent_id)
            .map(|neighbors| neighbors.iter().copied().collect())
            .unwrap_or_default()
    }

    // get len of degrees for an agent
    pub fn get_degree(&self, agent_id: u32) -> usize {
        self.connections
            .get(&agent_id)
            .map(|neighbors| neighbors.len())
            .unwrap_or(0)
    }

    // retrieve connections
    pub fn get_all_connections(&self) -> Vec<(u32, u32)> {
        let mut temp_con = Vec::new();
        for (&agent_a, neighbors) in &self.connections {
            for &agent_b in neighbors {
                // use < to avoid duplication connections
                if agent_a < agent_b {
                    temp_con.push((agent_a, agent_b));
                }
            }
        }
        temp_con
    }

    // connection count
    pub fn connection_count(&self) -> usize {
        self.get_all_connections().len()
    }

    // get all ids in hashmap
    pub fn get_all_agent_ids(&self) -> HashSet<u32> {
        self.connections.keys().copied().collect()
    }
}

impl Default for Topology {
    fn default() -> Self {
        Self::new()
    }
}

// preset topology helper
pub struct TopologyBuilder;

impl TopologyBuilder {
    // agent linked to ever other agent
    pub fn fully_connected(agent_ids: &[u32]) -> Topology {
        let mut topology = Topology::new();

        for i in 0..agent_ids.len() {
            for j in (i + 1)..agent_ids.len() {
                topology.add_connection(agent_ids[i], agent_ids[j]);
            }
        }

        topology
    }

    // ring network
    pub fn ring(agent_ids: &[u32]) -> Topology {
        let mut topology = Topology::new();

        for i in 0..agent_ids.len() {
            let next = (i + 1) % agent_ids.len();
            topology.add_connection(agent_ids[i], agent_ids[next]);
        }

        topology
    }

    // star network 1 centralised agent
    pub fn star(center: u32, periphery: &[u32]) -> Topology {
        let mut topology = Topology::new();

        for &agent in periphery {
            topology.add_connection(center, agent);
        }

        topology
    }

    // random network with random_bool
    pub fn random(agent_ids: &[u32], connection_probability: f64) -> Topology {
        let mut topology = Topology::new();
        let mut rng = rand::rng();

        for i in 0..agent_ids.len() {
            for j in (i + 1)..agent_ids.len() {
                if rng.random_bool(connection_probability) {
                    topology.add_connection(agent_ids[i], agent_ids[j]);
                }
            }
        }

        topology
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_connection() {
        let mut topology = Topology::new();
        let agent_a = 1;
        let agent_b = 2;

        topology.add_connection(agent_a, agent_b);

        assert!(topology.are_connected(agent_a, agent_b));
        assert!(topology.are_connected(agent_b, agent_a)); // test both directions as well
        assert_eq!(topology.get_degree(agent_a), 1);
        assert_eq!(topology.get_degree(agent_b), 1);
    }

    #[test]
    fn test_get_neighbors() {
        let mut topology = Topology::new();
        let agent_a = 1;
        let agent_b = 2;
        let agent_c = 3;

        topology.add_connection(agent_a, agent_b);
        topology.add_connection(agent_a, agent_c);

        let neighbors = topology.get_neighbors(agent_a);
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&agent_b));
        assert!(neighbors.contains(&agent_c));
    }

    #[test]
    fn test_fully_connected() {
        let agents: Vec<u32> = (0..4).collect();
        let topology = TopologyBuilder::fully_connected(&agents);

        // each agent should have 3 conn
        for agent in &agents {
            assert_eq!(topology.get_degree(*agent), 3);
        }
    }

    #[test]
    fn test_random_topology() {
        let agent_ids = vec![1, 2, 3, 4, 5];

        let max_probability = TopologyBuilder::random(&agent_ids, 1.0);
        assert_eq!(max_probability.connection_count(), 10); // max 10 connections from sample of 5

        let low_probability = TopologyBuilder::random(&agent_ids, 0.0);
        assert_eq!(low_probability.connection_count(), 0);
    }
}
