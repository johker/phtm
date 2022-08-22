// Control Module
// Control of different stages of the node lifecycle
//

#[derive(Debug, PartialEq)]
pub enum State {
    Waiting { waiting_time: usize },
    Computing,
    Replicating,
    Done,
    Failure(String),
}

#[derive(Debug, Clone, Copy)]
pub enum Event {
    NothingHappened,
    TemporalMemoryInitialized,
}

impl State {
    pub fn next(self, event: Event) -> State {
        match (self, event) {
            (State::Waiting { waiting_time }, Event::NothingHappened) => State::Waiting {
                waiting_time: waiting_time + 1,
            },
            (s, e) => State::Failure(
                format!("Wrong state, event combintation: {:#?} {:#?}", s, e).to_string(),
            ),
        }
    }

    pub fn run(&self) {
        match *self {
            State::Waiting { waiting_time } => {
                println!("We waited for {}", waiting_time);
            }
            State::Computing {} => {
                // Compute update on input
            }
            State::Replicating {} => {}
            State::Done | State::Failure(_) => {}
        }
    }
}
