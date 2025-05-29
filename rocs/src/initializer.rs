//! in src/initialzer.rs
//!
//! Initializes all the actors in appropriate order

use crate::actors::{
    store_actor::spawn_store_actor,
    admin_actor::spawn_admin_actor,
    user_actor::spawn_user_actor,
};

use std::collections::HashMap;
use crate::router::ActorChannels;
use std::sync::{
    Arc,
    Mutex,
};

pub async fn initialize_system() -> ActorChannels {

    let store_actor = spawn_store_actor();
    //let admin_actor = spawn_admin_actor();
    
    let mut user_actors = Arc::new(Mutex::new(HashMap::new()));

    ActorChannels {
        user_actors,
        store_actor,
    }
}
