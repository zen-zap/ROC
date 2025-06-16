use anyhow::Result;
use quinn::{RecvStream, SendStream};
use crate::router::{ActorChannels, route_cmd};
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncBufReadExt, BufReader};
use tokio::sync::oneshot;
use crate::command::Command;
use crate::wire_cmd::{WireCommand, WireResponseReceiver};

pub async fn handle_connection(
	mut send: SendStream,
	mut recv: RecvStream,
	system: ActorChannels,
) -> Result<()> {

    let mut reader = BufReader::new(&mut recv);
    let mut buffer = Vec::new();

    loop {
        buffer.clear();

        let bytes_read = reader.read_until(b'\n', &mut buffer).await.expect("failed to read into the buffer");

        if bytes_read == 0 {
            break;
        }

        let line = String::from_utf8_lossy(&buffer);
        //eprintln!("GOT THIS FROM CLIENT: {:?}", line);

        let mut wire_cmd: WireCommand = match serde_json::from_str(&line) {
            Ok(wcmd) => wcmd,
            Err(e) => {
                eprintln!("failed to parse wired command: {}", e);
                break;
            }
        };
        //eprintln!("PARSED WIRED COMMAND: {:?}", wire_cmd);

        let (cmd, wire_response_recv) = wire_cmd.clone().into_internal();


        route_cmd(cmd, &system).await;
        //eprintln!("ROUTING DONE!");

        let response_json = match wire_response_recv {
            WireResponseReceiver::UserId(rx) => {
                let user_id = rx.await?;
                serde_json::json!({ "user_id": user_id }).to_string()
            }
            WireResponseReceiver::String(rx) => {
                let s = rx.await?;
                serde_json::json!({ "response": s }).to_string()
            }
            WireResponseReceiver::ResultUnit(rx) => {
                let res = rx.await?;
                serde_json::to_string(&res)?
            }
            WireResponseReceiver::ResultOptUsize(rx) => {
                let res = rx.await?;
                serde_json::to_string(&res)?
            }
            WireResponseReceiver::ResultKvVec(rx) => {
                let res = rx.await?;
                serde_json::to_string(&res)?
            }
        };

        send.write_all(response_json.as_bytes()).await?;
        send.write_all(b"\n").await?;
        send.flush().await?;

        if matches!(wire_cmd, WireCommand::Exit { .. }) {
            break;
        }
    }
	
	Ok(())
}
