use tokio::net::{TcpListener, TcpStream};
use tokio::io::{copy, split};
// use std::sync::{Arc, Mutex};
// use std::time::{Duration, SystemTime};

const DEST_MACHINE: &'static str = "127.0.0.1:25565";
const LISTEN_PORT: &'static str = "127.0.0.1:23";

async fn handle_client(mut stream: TcpStream) {
    println!("handle client called");
    let mut tunnel = TcpStream::connect(DEST_MACHINE).await.unwrap();
    println!("tunnel established");
    let (mut tunnel_reader, mut tunnel_writer) =
        split(tunnel);
    let (mut stream_reader, mut stream_writer) =
        split(stream);
    tokio::spawn(async move {
        copy(&mut stream_reader, &mut tunnel_writer).await;
    });
    tokio::spawn(async move {
        copy(&mut tunnel_reader, &mut stream_writer).await;
    });
}

//TODO
// Implement server_timer

// async fn server_timer(server_on: Arc<Mutex<bool>>) {
//
// }


//TODO
// Clean this up

#[tokio::main]
async fn main() {

    //let server_on = Arc::new(Mutex::new(true));
    let listener = TcpListener::bind(LISTEN_PORT).await.unwrap();
    println!("entering loop");
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            handle_client(socket).await;
        });
    }
}