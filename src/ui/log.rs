use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogMessage {
    pub text: String,
    pub color: [u8; 3], // RGB
    pub turn: u64,
    pub count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameLog {
    pub messages: Vec<LogMessage>,
    pub max_messages: usize,
    pub history: Vec<LogMessage>,
    pub max_history: usize,
    pub turn_message_count: u32,
    pub needs_more: bool,
    pub current_turn: u64,
}

impl GameLog {
    pub fn new(max_messages: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_messages,
            history: Vec::new(),
            max_history: 1000,
            turn_message_count: 0,
            needs_more: false,
            current_turn: 0,
        }
    }

    pub fn add<S: Into<String>>(&mut self, text: S, turn: u64) {
        self.add_colored(text, [255, 255, 255], turn);
    }

    pub fn add_colored<S: Into<String>>(&mut self, text: S, color: [u8; 3], turn: u64) {
        let text_str = text.into();
        if text_str.is_empty() {
            return;
        }

        //
        if !self.messages.is_empty() && self.messages.last().unwrap().turn != turn {
            self.turn_message_count = 0;
            self.needs_more = false;
        }
        self.turn_message_count += 1;
        if self.turn_message_count >= 8 {
            self.needs_more = true;
        }

        //
        if let Some(last) = self.messages.last_mut() {
            if last.text == text_str && last.color == color {
                last.count += 1;
                last.turn = turn;
                                  //
                if let Some(last_hist) = self.history.last_mut() {
                    if last_hist.text == text_str && last_hist.color == color {
                        last_hist.count += 1;
                        last_hist.turn = turn;
                    }
                }
                return;
            }
        }

        let msg = LogMessage {
            text: text_str,
            color,
            turn,
            count: 1,
        };

        self.messages.push(msg.clone());
        self.history.push(msg);

        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    pub fn clear_turn_count(&mut self) {
        self.turn_message_count = 0;
        self.needs_more = false;
    }
}
