use anyhow::Result;
use core::Registry;
use core::agent::{Agent, InfectionStatus};
use eframe::{App, run_native};
use egui::{Color32, Context};
use egui_graphs::{Graph, default_edge_transform};
use petgraph::Directed;
use petgraph::stable_graph::StableGraph;

// graph type to use agent as the payload
type AgentGraph = Graph<Agent, (), Directed, u32>;

pub fn visualize_graph(registry: &Registry) -> Result<()> {
    struct GraphApp {
        g: AgentGraph,
    }

    impl GraphApp {
        fn new(_cc: &eframe::CreationContext<'_>, registry: &Registry) -> Self {
            let pet_graph = build_graph_from_registry(registry);

            // map agent data to visual properties
            let g = egui_graphs::to_graph_custom(
                &pet_graph,
                |n| {
                    // apply the default settings
                    egui_graphs::default_node_transform(n);

                    // get payload data
                    let agent = n.payload();

                    // match infection for color
                    let color = match agent.infection_status {
                        InfectionStatus::Healthy => Color32::LIGHT_GRAY,
                        InfectionStatus::Infected => Color32::DARK_RED,
                        InfectionStatus::Immune => Color32::LIGHT_BLUE,
                    };

                    // set label
                    n.set_label(agent.id.to_string());
                    // set color
                    n.set_color(color);
                },
                default_edge_transform,
            );

            Self { g }
        }
    }

    impl App for GraphApp {
        fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Agent Network Visualization");
                // specify type
                ui.add(&mut egui_graphs::GraphView::<
                    Agent,
                    (),
                    petgraph::Directed,
                    u32,
                    egui_graphs::DefaultNodeShape,
                    egui_graphs::DefaultEdgeShape,
                >::new(&mut self.g));
            });
        }
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 500.0])
            .with_resizable(true),
        ..Default::default()
    };
    run_native(
        "Debate Simulation - Agent Network",
        native_options,
        Box::new(|cc| Ok(Box::new(GraphApp::new(cc, registry)))),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {}", e))
}

fn build_graph_from_registry(registry: &Registry) -> StableGraph<Agent, ()> {
    use std::collections::HashMap;
    let mut g = StableGraph::new();

    // get all agents from the registry
    let agents = registry.get_all_agents();
    let mut node_map = HashMap::new();

    // add nodes with agent payload
    for agent in agents {
        let id = agent.id;
        let idx = g.add_node(agent.clone());
        node_map.insert(id, idx);
    }

    // add connections
    if let Some(topology) = &registry.topology {
        for (from_id, to_id) in topology.get_all_connections() {
            if let (Some(&f), Some(&t)) = (node_map.get(&from_id), node_map.get(&to_id)) {
                g.add_edge(f, t, ());
            }
        }
    }

    g
}
