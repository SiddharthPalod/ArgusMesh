use std::collections::VecDeque;
use super::envelope::{Envelope, Priority};

pub struct PriorityQueue {
    critical: VecDeque<Envelope>,
    high: VecDeque<Envelope>,
    normal: VecDeque<Envelope>,
    low: VecDeque<Envelope>,
    cursor: u8,
}

impl PriorityQueue {
    pub fn new() -> Self {
        Self {
            critical: VecDeque::new(),
            high: VecDeque::new(),
            normal: VecDeque::new(),
            low: VecDeque::new(),
            cursor: 0,
        }
    }

    pub fn push(&mut self, env: Envelope) {
        match env.priority {
            Priority::Critical => self.critical.push_back(env),
            Priority::High => self.high.push_back(env),
            Priority::Normal => self.normal.push_back(env),
            Priority::Low => self.low.push_back(env),
        }
    }

    pub fn pop(&mut self) -> Option<Envelope> {
        self.cursor = (self.cursor + 1) % 11;

        let pick = match self.cursor {
            0..=4 => self.critical.pop_front(),
            5..=7 => self.high.pop_front(),
            8..=9 => self.normal.pop_front(),
            _ => self.low.pop_front(),
        };

        pick.or_else(|| {
            self.critical.pop_front()
                .or_else(|| self.high.pop_front())
                .or_else(|| self.normal.pop_front())
                .or_else(|| self.low.pop_front())
        })
    }

    pub fn is_empty(&self) -> bool {
        self.critical.is_empty()
            && self.high.is_empty()
            && self.normal.is_empty()
            && self.low.is_empty()
    }
}