use niri_ipc::{Action, Request, socket};

use tracing::{error, info};

use zibi_core::direction::Direction;

pub fn connect() -> socket::Socket {
    match socket::Socket::connect() {
        Ok(socket) => {
            info!("Connected to niri socket");
            socket
        }
        Err(e) => {
            error!(error = ?e, "Cannot connect to niri socket");
            panic!("Cannot connect to niri socket");
        }
    }
}

pub fn send_direction(socket: &mut socket::Socket, dir: Direction) {
    info!("Direction: {:?}", dir);

    let action = match dir {
        Direction::Up => Request::Action(Action::FocusWorkspaceUp {}),
        Direction::Down => Request::Action(Action::FocusWorkspaceDown {}),
        Direction::Left => Request::Action(Action::FocusColumnLeft {}),
        Direction::Right => Request::Action(Action::FocusColumnRight {}),
    };

    if let Err(err) = socket.send(action) {
        error!("error when sending request to niri socket, {err}");
    }
}
