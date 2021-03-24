use tokio::net::{TcpListener, TcpStream};
use tokio::io::copy;
// use std::sync::{Arc, Mutex};
// use std::time::{Duration, SystemTime};

const DEST_MACHINE: &'static str = "127.0.0.1:25565";
const LISTEN_PORT: &'static str = "127.0.0.1:23";

//TODO
// Check if this is broken or not. Current testing can't get the tunnel established,
// but may be a telnet thing

async fn handle_client(mut stream: TcpStream) {
    println!("handle client called");
    let mut tunnel = TcpStream::connect(DEST_MACHINE).await.unwrap();
    println!("tunnel established");
    loop {
        println!("Forwarded {} bytes", copy(&mut tunnel, &mut stream).await.unwrap());
        println!("Forwarded {} bytes", copy(&mut stream, &mut tunnel).await.unwrap());
    }
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