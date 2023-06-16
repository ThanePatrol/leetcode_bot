#[derive(Debug)]
pub struct Leetcode {
    pub name: String,
    pub url: String,
    pub have_solved: bool
}

impl Leetcode {
    pub fn new(name: String, url: String) -> Self {
        Self {
            name,
            url,
            have_solved: false,
        }
    }
}