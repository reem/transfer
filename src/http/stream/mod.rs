use parser::{StreamIdentifier};

use self::state::State;

mod state;

pub struct Stream {
    id: StreamIdentifier,
    state: State
}

