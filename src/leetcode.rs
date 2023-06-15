#[derive(Debug)]
pub struct Leetcode {
    name: String,
    url: String,
    have_solved: bool
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