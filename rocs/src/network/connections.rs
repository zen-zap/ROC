use anyhow;
use quinn::{SendStream, RecvStream};

pub async fn handle_connection<S, System>(send: SendStream, recv: RecvStream, _system: System) -> anyhow::Result<()> {
    // TODO: implement actual connection handling
    Ok(())
}

