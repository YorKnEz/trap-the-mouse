mod models;

pub use models::*;

pub use rusqlite::{params, Connection, Result};

pub static DB_NAME: &str = "db.db";

pub fn init_db() -> Result<()> {
    let conn = Connection::open(DB_NAME)?;

    let _ = conn.execute("DROP TABLE user", ());

    conn.execute(
        "CREATE TABLE user (
    name TEXT NOT NULL, 
    addr TEXT NOT NULL, 
    notify_addr TEXT NOT NULL,
    highscore INTEGER, 
    PRIMARY KEY (name, addr)
)",
        (),
    )?;

    Ok(())
}

pub trait UserOps {
    const GET_USER_BY_NAME: &'static str;
    const ADD_USER: &'static str;
    const REMOVE_USER: &'static str;

    fn get_user_by_key(&self, name: &str, addr: &str) -> Result<User>;
    fn add_user(&self, name: &str, addr: &str, notify_addr: &str) -> Result<()>;
    fn remove_user(&self, name: &str, addr: &str) -> Result<()>;
}

impl UserOps for Connection {
    const GET_USER_BY_NAME: &'static str = "SELECT * FROM user WHERE (name, addr) = (?1, ?2)";
    const ADD_USER: &'static str = "INSERT INTO user (name, addr, notify_addr) VALUES(?1, ?2, ?3)";
    const REMOVE_USER: &'static str = "DELETE FROM user WHERE (name, addr) = (?1, ?2)";

    fn get_user_by_key(&self, name: &str, addr: &str) -> Result<User> {
        let mut stmt = self.prepare(Self::GET_USER_BY_NAME)?;

        let user = stmt.query_row([name, addr], |row| {
            Ok(User {
                name: row.get(0)?,
                addr: row.get(1)?,
                notify_addr: row.get(2)?,
                highscore: row.get(3)?,
            })
        })?;

        Ok(user)
    }

    fn add_user(&self, name: &str, addr: &str, notify_addr: &str) -> Result<()> {
        let mut stmt = self.prepare(Self::ADD_USER)?;

        stmt.execute(params![name, addr, notify_addr])?;

        Ok(())
    }

    fn remove_user(&self, name: &str, addr: &str) -> Result<()> {
        let mut stmt = self.prepare(Self::REMOVE_USER)?;

        stmt.execute(params![name, addr])?;

        Ok(())
    }
}
