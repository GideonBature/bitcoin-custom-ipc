use std::os::unix::net::UnixListener;

use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

mod protocol {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Req {
        pub id: u32,
        pub method: String,
        pub params: Vec<Value>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Res {
        pub id: u32,
        pub result: Value,
        pub error: Option<String>,
    }
}
use protocol::{Req, Res};

async fn handle_connection(mut stream: UnixStream) -> tokio::io::Result<()> {
    let (reader, mut writer) = stream.split();
    let mut buf_reader = BufReader::new(reader);
    let mut buffer = String::new();

    while buf_reader.read_line(&mut buffer).await? != 0 {
        let req: Req = match serde_json::from_str(&buffer) {
            Ok(req) => req,
            Err(e) => {
                eprint!("Error parsing request: {}", e);
                buffer.clear();
                continue;
            }
        };
        println!("Received request: {::?}", req);

        let res = match req.method.as_str() {
            "getblockhash" => {
                let block_height = req.params.get(0).and_then(|v| v.as_u64()).unwrap();

                let dummy_hash = format!("0000000000000000000{:x}", block_height);
                Res {
                    id: req.id,
                    result: json!(dummy_hash),
                    error: None,
                }
            }

            "getblockcount" => Res {
                id: req.id,
                result: json!(84372),
                error: None,
            },
            _ => Res {
                id: req.id,
                result: json!(null),
                error: Some("Unknown method".to_string()),
            },
        };

        let res_str = serde_json::to_string(&res).unwrap();
        writer.write_all(res_str.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
        buffer.clear();
    }
    Ok(())
}

#[tokio::main]
fn main() -> tokio::io::Result<()> {
    let socket_path = "/tmp/ipc_socket";

    let _ = std::fs::remove_file(socket_path);
    let listener = UnixListener::bind(socket_path)?;
    println!("Server listening on {}", socket_path);

    loop {
        let (stream, _) = listener.accept()?;

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                eprintln!("Connection error: {:?}", e)
            }
        });
    }
}
