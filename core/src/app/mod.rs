use log::{debug, error, warn};

use self::actions::Actions;
use self::state::AppState;
use crate::app::actions::Action;
use crate::inputs::key::Key;
use crate::io::IoEvent;
use crate::node::execution::PushExecutor;
use crate::node::source::Source;
use crate::pushr::push::state::PushState;
use crate::pushr::push::random::CodeGenerator;
use crate::pushr::push::vector::{BoolVector, IntVector};
use crate::pushr::push::item::Item;
use crate::pushr::push::io::PushMessage;
use crate::pushr::push::instructions::{InstructionCache, InstructionSet};

pub mod actions;
pub mod state;
pub mod ui;

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

/// The main application, containing the state
pub struct App {
    /// We could dispatch an IO event
    io_tx: tokio::sync::mpsc::Sender<IoEvent>,
    /// Contextual actions
    actions: Actions,
    /// State
    is_loading: bool,
    state: AppState,
    /// Push State
    executor: PushExecutor,

}

impl App {
    pub fn new(io_tx: tokio::sync::mpsc::Sender<IoEvent>, bin: String, code: String) -> Self {
        let actions = vec![Action::Quit].into();
        let is_loading = false;
        let state = AppState::default();
        let mut executor = PushExecutor::new();
        executor.initialize();

        // Load program code
        let program = Source::read_debug_code(code);
        executor.load(program);
        // Inject interpreter binary
        executor.push_state.name_bindings.insert("BIN".to_string(), Item::id(bin)); 

        Self {
            io_tx,
            actions,
            is_loading,
            state,
            executor,
        }
    }

    /// Handle a user action
    pub async fn do_action(&mut self, key: Key) -> AppReturn {
        if let Some(action) = self.actions.find(key) {
            debug!("Run action [{:?}]", action);
            match action {
                Action::Quit => AppReturn::Exit,
                Action::Sleep => {
                    if let Some(duration) = self.state.duration().cloned() {
                        // Sleep is an I/O action, we dispatch on the IO channel that's run on another thread
                        self.dispatch(IoEvent::Sleep(duration)).await
                    }
                    AppReturn::Continue
                }
                // IncrementDelay and DecrementDelay is handled in the UI thread
                Action::IncrementDelay => {
                    self.state.increment_delay();
                    AppReturn::Continue
                }
                // Note, that we clamp the duration, so we stay >= 0
                Action::DecrementDelay => {
                    self.state.decrement_delay();
                    AppReturn::Continue
                }
                Action::Step => {
                    self.executor.step();
                    AppReturn::Continue
                }
                Action::RandomInput => {
                    if let Some(random_input) = CodeGenerator::random_bool_vector(1024, 0.05) {
                        let input_msg = PushMessage::new(IntVector::new(vec![]), random_input);
                        self.executor.push_state.input_stack.push(input_msg);
                    }
                    AppReturn::Continue
                }
            }
        } else {
            warn!("No action accociated to {}", key);
            AppReturn::Continue
        }
    }

    /// We could update the app or dispatch event on tick
    pub async fn update_on_tick(&mut self) -> AppReturn {
        // here we just increment a counter
        self.state.incr_tick();
        self.executor.step();
        AppReturn::Continue
    }

    /// Send a network event to the IO thread
    pub async fn dispatch(&mut self, action: IoEvent) {
        // `is_loading` will be set to false again after the async action has finished in io/handler.rs
        self.is_loading = true;
        if let Err(e) = self.io_tx.send(action).await {
            self.is_loading = false;
            error!("Error from dispatch {}", e);
        };
    }

    pub fn actions(&self) -> &Actions {
        &self.actions
    }

    pub fn app_state(&self) -> &AppState {
        &self.state
    }

    pub fn push_state(&self) -> &PushState {
        &self.executor.push_state
    }

    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    pub fn initialized(&mut self) {
        // Update contextual actions
        self.actions = vec![
            Action::Quit,
            Action::Sleep,
            Action::IncrementDelay,
            Action::DecrementDelay,
        ]
        .into();
        self.state = AppState::initialized()
    }

    pub fn loaded(&mut self) {
        self.is_loading = false;
    }

    pub fn slept(&mut self) {
        self.state.incr_sleep();
    }
}
