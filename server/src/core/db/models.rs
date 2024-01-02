#[derive(Debug)]
pub struct User {
    pub name: String,
    pub addr: String,
    pub notify_addr: String,
    pub highscore: Option<u32>,
}
