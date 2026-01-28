use std::{env, error::Error, os::unix::io::AsRawFd, time::Duration};

use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    time::sleep,
};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // create ipv4 socket and bind to all interfaces
    let addr = env::var("SERVER_ADDR")?;
    let listener = TcpListener::bind(&addr).await?;
    info!("server listening on {}", addr);

    loop {
        // accept connection and spawn task to handle each
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                info!("accepted connection from {}", peer_addr);
                tokio::spawn(handle_connection(stream));
            }
            Err(e) => {
                error!("failed to accept connection: {}", e);
            }
        }
    }
}

async fn handle_connection(mut stream: TcpStream) {
    // Set small receive buffer (4KB) to observe backpressure effects
    let fd = stream.as_raw_fd();
    unsafe {
        let size: libc::c_int = 4096;
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_RCVBUF,
            &size as *const _ as *const libc::c_void,
            std::mem::size_of::<libc::c_int>() as libc::socklen_t,
        );
    }

    let peer_addr = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let mut buf = [0u8; 1024];

    // loop forever reading from stream
    loop {
        match stream.read(&mut buf).await {
            Ok(0) => {
                info!("connection closed by {}", peer_addr);
                break;
            }
            Ok(n) => {
                let msg = String::from_utf8_lossy(&buf[..n]);
                info!("received from {}: {}", peer_addr, msg.trim());
            }
            Err(e) => {
                error!("error reading from {}: {}", peer_addr, e);
                break;
            }
        }

        sleep(Duration::from_millis(500)).await;
    }
}
