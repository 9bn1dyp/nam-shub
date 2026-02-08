use std::fmt;

// each individual message
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub id: u32,
    pub message: String,
}

// each exchange in debate
#[derive(Debug, Clone, PartialEq)]
pub struct Exchange {
    // message struct proposer message, opposer reply
    pub proposer: Message,
    pub opposer: Message,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DebateOutcome {
    #[default]
    Ongoing,
    ProposerWon,
    OpposerWon,
}

impl fmt::Display for DebateOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DebateOutcome::Ongoing => write!(f, "Ongoing"),
            DebateOutcome::ProposerWon => write!(f, "Proposer won"),
            DebateOutcome::OpposerWon => write!(f, "Opposer won"),
        }
    }
}

// full debate between both agents, both agents will ref this in their structs
#[derive(Debug, Clone, PartialEq)]
pub struct Debate {
    pub proposer_id: u32,
    pub opposer_id: u32,

    // max turns for each agent
    pub max_turns: usize,
    pub exchanges: Vec<Exchange>,

    // judges outcome of debate
    pub outcome: DebateOutcome,
}

impl Debate {
    pub fn new(proposer_id: u32, opposer_id: u32, max_turns: usize) -> Self {
        Self {
            proposer_id,
            opposer_id,
            max_turns,
            exchanges: Vec::new(),
            outcome: DebateOutcome::default(),
        }
    }

    // add exchange
    pub fn add_exchange(&mut self, exchange: Exchange) {
        self.exchanges.push(exchange);
    }

    // check if max exchange already reached
    pub fn is_complete(&self) -> bool {
        self.exchanges.len() >= self.max_turns
    }

    // set judge outcome
    pub fn set_outcome(&mut self, outcome: DebateOutcome) {
        self.outcome = outcome;
    }

    // format debate into transcript
    pub fn format_transcript(&self) -> String {
        // debate info
        let mut transcript = format!(
            "Debate: Agent {} (Proposer) vs Agent {} (Opposer)\n
             Max turns per agent: {}\n
             Status: {:?}\n\n",
            self.proposer_id, self.opposer_id, self.max_turns, self.outcome
        );

        // exchanges
        for (i, turn) in self.exchanges.iter().enumerate() {
            transcript.push_str(&format!(
                "
                Round {}\n
                Agent {} (Proposer) Message: {}\n
                Agent {} (Opposer) Reply: {}\n\n",
                i + 1,
                self.proposer_id,
                turn.proposer.message,
                self.opposer_id,
                turn.opposer.message,
            ));
        }

        // outcome
        transcript.push_str(&format!("Judge's verdict: {}", self.outcome));

        transcript
    }
}
