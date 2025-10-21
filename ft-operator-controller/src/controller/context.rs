use kube::Client;
use std::sync::Arc;

use ft_operator_common::state::State;

// Context struct to hold the kube client and the state
#[derive(Clone)]
pub struct Context {
    pub client: Client,
    pub state: Option<Arc<State>>,
}

impl Context {
    pub fn new(client: Client) -> Self {
        Self { client, state: None }
    }

    pub fn with_state(mut self, state: Arc<State>) -> Self {
        self.state = Some(state);
        self
    }
}