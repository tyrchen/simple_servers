use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:8080".parse::<SocketAddr>()?;
    let mut listener = TcpListener::bind(&addr).await?;

    loop {
        match listener.accept().await {
            Ok((socket, raddr)) => {
                println!("New client {:?} connected!", raddr);
                handle_request(socket);
            }
            Err(err) => println!("Failed to get client: {:?}", err),
        }
    }
}

fn handle_request(mut socket: TcpStream) {
    tokio::spawn(async move {
        let mut buf = [0; 1024];
        // in a loop, read data from the socket and write the data back.
        loop {
            let n = match socket.read(&mut buf).await {
                Ok(n) if n == 0 => {
                    println!("client left.");
                    return;
                }
                Ok(n) => n,
                Err(e) => {
                    println!("failed to read fro msocket: err = {:?}", e);
                    return;
                }
            };

            // write the data back
            if let Err(e) = socket.write_all(&buf[0..n]).await {
                println!("failed to write to socket: err = {:?}", e);
                return;
            }
        }
    });
}
