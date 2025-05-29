//! in src/router.rs
//!
//! Routes the commands to specific actors for handling them

use tokio::sync::mpsc;
use std::collections::HashMap;
use crate::command::Command;
use crate::actors::{
    user_actor::{UserCommandHandler, spawn_user_actor},
    store_actor::StoreCommandHandler,
};
use std::sync::{Arc, Mutex};

pub type ActorHandle = mpsc::Sender<Command>;

#[derive(Clone)]
pub struct ActorChannels {
    pub user_actors: Arc<Mutex<HashMap<String, UserCommandHandler>>>,
    pub store_actor: StoreCommandHandler,
}

pub async fn route_cmd(cmd: Command, actors: &ActorChannels) {

    match &cmd {
        Command::Set { user_id, .. }
        | Command::Get { user_id, .. }
        | Command::Del { user_id, .. }
        | Command::List { user_id, .. }
        | Command::Range { user_id, .. } => {

            let mut users = actors.user_actors.lock().unwrap();
            let user_actor = users.entry(user_id.clone())
                .or_insert_with(|| spawn_user_actor(actors.store_actor.clone()));
            let _ = user_actor.send(cmd).await;

        }
        Command::Ping { user_id, ..} => {
            let mut users = actors.user_actors.lock().unwrap();
            let user_actor = users.entry(user_id.clone())
                .or_insert_with(|| spawn_user_actor(actors.store_actor.clone()));
            let _ = user_actor.send(cmd).await;
        }
        _ => {
            eprintln!("Route not yet implemented!");
        }
    }
}
