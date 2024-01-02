use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("internal error")]
    InternalAnyhow(#[from] anyhow::Error),

    #[error("internal error")]
    InternalIo(#[from] std::io::Error),

    #[error("internal error")]
    InternalR2d2(#[from] r2d2::Error),

    #[error("internal error")]
    InternalRusqlite(#[from] rusqlite::Error),

    #[error("internal error")]
    InternalAddrParseError(#[from] std::net::AddrParseError),

    #[error("{message}")]
    API { message: String },

    #[error("you are not connected")]
    APINotConnected,
}
