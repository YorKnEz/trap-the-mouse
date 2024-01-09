// mod ping;

mod connect;
mod disconnect;

mod close_lobby;
mod create_lobby;
mod get_lobbies;
mod get_lobby_state;
mod join_lobby;
mod leave_lobby;

mod become_role;
mod change_name;
mod make_host;

mod make_move;
mod start_game;

// pub use ping::ping_cmd;

pub use connect::connect_cmd;
pub use disconnect::disconnect_cmd;

pub use close_lobby::close_lobby_cmd;
pub use create_lobby::create_lobby_cmd;
pub use get_lobbies::get_lobbies_cmd;
pub use get_lobby_state::get_lobby_state;
pub use join_lobby::join_lobby_cmd;
pub use leave_lobby::leave_lobby_cmd;

pub use become_role::become_role_cmd;
pub use change_name::change_name_cmd;
pub use make_host::make_host_cmd;

pub use make_move::make_move_cmd;
pub use start_game::start_game_cmd;
