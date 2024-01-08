mod models;

pub use models::*;

pub use rusqlite::{params, Connection, Result};

pub static DB_NAME: &str = "db.db";

pub fn init_db() -> Result<()> {
    let conn = Connection::open(DB_NAME)?;

    let _ = conn.execute("DROP TABLE user", ());

    conn.execute(
        "
CREATE TABLE user (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL, 
    addr TEXT NOT NULL,
    highscore INTEGER, 
    connected INTEGER,
    UNIQUE (name, addr)
)",
        (),
    )?;

    Ok(())
}

pub trait UserOps {
    const GET_USER_BY_ID: &'static str;
    const GET_USER_BY_KEY: &'static str;
    const ADD_USER: &'static str;
    const CHANGE_USER_NAME: &'static str;
    const REMOVE_USER: &'static str;
    const TOGGLE_CONNECTED: &'static str;

    fn get_user_by_id(&self, id: u32) -> Result<User>;
    fn get_user_by_key(&self, name: &str, addr: &str) -> Result<User>;
    fn add_user(&self, name: &str, addr: &str) -> Result<()>;
    fn change_user_name(&self, id: u32, name: &str) -> Result<()>;
    fn remove_user(&self, id: u32) -> Result<()>;
    fn toggle_connected(&self, id: u32) -> Result<()>;
    fn is_connected(&self, id: u32) -> Result<Option<User>>;
}

impl UserOps for Connection {
    const GET_USER_BY_ID: &'static str = "SELECT * FROM user WHERE id = ?1";
    const GET_USER_BY_KEY: &'static str = "SELECT * FROM user WHERE (name, addr) = (?1, ?2)";
    const ADD_USER: &'static str = "INSERT INTO user (name, addr, connected) VALUES(?1, ?2, 1)";
    const CHANGE_USER_NAME: &'static str = "UPDATE user SET name = ?2 WHERE id = ?1";
    const REMOVE_USER: &'static str = "DELETE FROM user WHERE id = ?1";
    const TOGGLE_CONNECTED: &'static str =
        "UPDATE user SET connected = NOT connected WHERE id = (?1)";

    fn get_user_by_id(&self, id: u32) -> Result<User> {
        let mut stmt = self.prepare(Self::GET_USER_BY_ID)?;

        let user = stmt.query_row([id], |row| {
            Ok(User {
                id: row.get(0)?,
                name: row.get(1)?,
                addr: row.get(2)?,
                highscore: row.get(3)?,
                connected: row.get(4)?,
            })
        })?;

        Ok(user)
    }

    fn get_user_by_key(&self, name: &str, addr: &str) -> Result<User> {
        let mut stmt = self.prepare(Self::GET_USER_BY_KEY)?;

        let user = stmt.query_row([name, addr], |row| {
            Ok(User {
                id: row.get(0)?,
                name: row.get(1)?,
                addr: row.get(2)?,
                highscore: row.get(3)?,
                connected: row.get(4)?,
            })
        })?;

        Ok(user)
    }

    fn add_user(&self, name: &str, addr: &str) -> Result<()> {
        let mut stmt = self.prepare(Self::ADD_USER)?;

        stmt.execute(params![name, addr])?;

        Ok(())
    }

    fn change_user_name(&self, id: u32, name: &str) -> Result<()> {
        let mut stmt = self.prepare(Self::CHANGE_USER_NAME)?;

        stmt.execute(params![id, name])?;

        Ok(())
    }

    fn remove_user(&self, id: u32) -> Result<()> {
        let mut stmt = self.prepare(Self::REMOVE_USER)?;

        stmt.execute(params![id])?;

        Ok(())
    }

    fn toggle_connected(&self, id: u32) -> Result<()> {
        let mut stmt = self.prepare(Self::TOGGLE_CONNECTED)?;

        stmt.execute(params![id])?;

        Ok(())
    }

    fn is_connected(&self, id: u32) -> Result<Option<User>> {
        let user = self.get_user_by_id(id)?;

        if user.connected == 1 {
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
}
