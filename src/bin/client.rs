use std::{env, error::Error, os::unix::io::AsRawFd, time::Instant};

use tokio::{io::AsyncWriteExt, net::TcpStream};
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

    // Set small send buffer (4KB) to observe backpressure effects
    let fd = stream.as_raw_fd();
    unsafe {
        let size: libc::c_int = 4096;
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_SNDBUF,
            &size as *const _ as *const libc::c_void,
            std::mem::size_of::<libc::c_int>() as libc::socklen_t,
        );
    }

    info!("connected to {}", addr);

    let mut counter = 0u64;

    // loop and send messages every 5 seconds
    loop {
        counter += 1;
        let msg = format!("Hello from client! Message #{} {}\n", counter, "X".repeat(4096));

        info!("sending {} bytes...", msg.len());
        let start = Instant::now();
        match stream.write_all(msg.as_bytes()).await {
            Ok(()) => {
                let elapsed = start.elapsed();
                info!(duration_us = elapsed.as_micros(), "sent");
                // info!(duration_us = elapsed.as_micros(), "sent: {}", msg.trim());
            }
            Err(e) => {
                error!("failed to send: {}", e);
                break;
            }
        }

        // if counter >= 5 {
        //     stream.shutdown().await?;
        //     return Ok(());
        // }

        // time::sleep(time::Duration::from_secs(2)).await;
    }

    Ok(())
}
