use anyhow::Result;
use quinn::{RecvStream, SendStream};
use crate::router::ActorChannels;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn handle_connection(
	send: SendStream,
	recv: RecvStream,
	system: ActorChannels,
) -> Result<()> {
	
	Ok(())
}
