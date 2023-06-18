use std::any::{Any, TypeId};
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

    pub fn serialize_categories(&self) -> String {
        serde_json::ser::to_string(&self.categories)
            .expect("Error serializing category")
    }
}

#[derive(Debug, Clone)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard
}

impl Difficulty {
    /// used for creating a new enum or deserializing from db
    pub fn new(input: String) -> Self {
        match input.as_str() {
            "Easy" => Easy,
            "Medium" => Medium,
            _ => Hard,
        }
    }

    pub fn serialize_to_str(&self) -> String {
        let str = match self {
            Easy => "Easy",
            Medium => "Medium",
            Hard => "Hard"
        };
        serde_json::ser::to_string(str)
            .expect("Error serializing enum to str")
    }
}