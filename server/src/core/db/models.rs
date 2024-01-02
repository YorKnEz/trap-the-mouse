#[derive(Debug)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub addr: String,
    pub highscore: Option<u32>,
    pub connected: u32,
}
