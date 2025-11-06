use std::ops::Deref;

use dioxus::fullstack::{FullstackContext, extract::FromRef};
use fern_server::Server;


#[derive(Clone)]
pub struct AppState {
  pub server : Server
}

impl From<Server> for AppState {
    fn from(server: Server) -> Self {
        Self {
          server
        }
    }
}

impl FromRef<FullstackContext> for AppState {
    fn from_ref(state: &FullstackContext) -> Self {
        state.extension::<AppState>().unwrap()
    }
}