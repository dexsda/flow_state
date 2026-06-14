use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ChecklistType {
    None,
    RoundRobin,
    Todo,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Checklist {
    pub checklist_type: ChecklistType,
    pub checklist: VecDeque<String>,
}

impl Default for Checklist {
    fn default() -> Self {
        Checklist {
            checklist_type: ChecklistType::None,
            checklist: std::collections::VecDeque::default(),
        }
    }
}
