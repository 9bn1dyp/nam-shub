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
    // message struct attackers message, defenders reply
    pub attacker: Message,
    pub defender: Message,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DebateOutcome {
    #[default]
    Ongoing,
    AttackerWon,
    DefenderWon,
}

impl fmt::Display for DebateOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DebateOutcome::Ongoing => write!(f, "Ongoing"),
            DebateOutcome::AttackerWon => write!(f, "Attacker won"),
            DebateOutcome::DefenderWon => write!(f, "Defender won"),
        }
    }
}

// full debate between both agents, both agents will ref this in their structs
#[derive(Debug, Clone, PartialEq)]
pub struct Debate {
    pub attacker_id: u32,
    pub defender_id: u32,

    // max turns for each agent
    pub max_turns: usize,
    pub exchanges: Vec<Exchange>,

    // judges outcome of debate
    pub outcome: DebateOutcome,
}

impl Debate {
    pub fn new(attacker_id: u32, defender_id: u32, max_turns: usize) -> Self {
        Self {
            attacker_id,
            defender_id,
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
        self.exchanges.len() - 1 == self.max_turns
    }

    // set judge outcome
    pub fn set_outcome(&mut self, outcome: DebateOutcome) {
        self.outcome = outcome;
    }

    // format debate into transcript
    pub fn format_transcript(&self) -> String {
        // debate info
        let mut transcript = format!(
            "Debate: Agent {} (Attacker) vs Agent {} (Defender)\n
             Max turns per agent: {}\n
             Status: {:?}\n\n",
            self.attacker_id, self.defender_id, self.max_turns, self.outcome
        );

        // exchanges
        for (i, turn) in self.exchanges.iter().enumerate() {
            transcript.push_str(&format!(
                "
                Round {}\n
                Agent {} (Attacker) Message: {}\n
                Agent {} (Defender) Reply: {}\n\n",
                i + 1,
                self.attacker_id,
                turn.attacker.message,
                self.defender_id,
                turn.defender.message,
            ));
        }

        // outcome
        transcript.push_str(&format!("Judge's verdict: {}", self.outcome));

        transcript
    }
}
