use std::{env, error::Error};

use tokio::{io::AsyncWriteExt, net::TcpStream, time};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // connect and get stream
    let addr = env::var("SERVER_ADDR")?;
    let mut stream = TcpStream::connect(&addr).await.expect("failed to connect");
    info!("connected to {}", addr);

    let mut counter = 0u64;

    // loop and send messages every 5 seconds
    loop {
        counter += 1;
        let msg = format!("Hello from client! Message #{}\n", counter);

        match stream.write_all(msg.as_bytes()).await {
            Ok(()) => {
                info!("sent: {}", msg.trim());
            }
            Err(e) => {
                error!("failed to send: {}", e);
                break;
            }
        }

        if counter >= 5 {
            stream.shutdown().await?;
            return Ok(());
        }

        time::sleep(time::Duration::from_secs(2)).await;
    }

    Ok(())
}
