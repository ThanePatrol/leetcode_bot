use crate::leetcode::Difficulty::{Easy, Hard, Medium};

#[derive(Debug, Clone)]
pub struct Leetcode {
    pub db_id: u32,
    pub name: String,
    pub url: String,
    pub number: u32,
    pub categories: Vec<String>,
    pub difficulty: Difficulty,
    pub have_solved: bool
}

impl Leetcode {
    pub fn new(name: String, url: String) -> Self {
        Self {
            db_id: 0,
            name,
            url,
            number: 0,
            categories: vec![],
            difficulty: Difficulty::Easy,
            have_solved: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard
}

impl Difficulty {
    pub fn new(input: String) -> Self {
        match input.as_str() {
            "Easy" => Easy,
            "Medium" => Medium,
            _ => Hard,
        }
    }
}