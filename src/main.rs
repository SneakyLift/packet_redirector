use tokio::net::{TcpListener, TcpStream};
use tokio::io::{copy, split};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tokio::sync::Mutex;

const DEST_MACHINE: &'static str = "127.0.0.1:25565";
const LISTEN_PORT: &'static str = "127.0.0.1:23";

async fn handle_client(stream: TcpStream) {
    println!("handle client called");
    let tunnel = TcpStream::connect(DEST_MACHINE).await.unwrap();
    println!("tunnel established");
    let (mut tunnel_reader, mut tunnel_writer) =
        split(tunnel);
    let (mut stream_reader, mut stream_writer) =
        split(stream);
    tokio::spawn(async move {
        match copy(&mut stream_reader, &mut tunnel_writer).await {
            Ok(n) => println!("Sent {} bytes!", n),
            Err(e) => println!("stream_reader copy future error: {}", e)
        }
    });
    tokio::spawn(async move {
        match copy(&mut tunnel_reader, &mut stream_writer).await {
            Ok(n) => println!("Received {} bytes!", n),
            Err(e) => println!("tunnel_reader copy future error: {}", e)
        }
    });
}

//TODO
// Build wake on lan function to be called when the server must be turned on.
// Should be passed the server_on arc and set it appropriately



//TODO
// Build a function which turns off the server machine
// Should be passed the server_on arc and set it appropriately



//TODO
// Make this function stop checking while the server is off
//This function checks to see whether or not the server has been inactive (0 clients) for 30 minutes
async fn server_timer(client_count: Arc<Mutex<u8>>) {
    loop {
        //do this check every 2 minutes
        sleep(Duration::from_secs(120)).await;
        let clients = client_count.lock().await;
        //if there are no clients, wait 30 minutes and check if there are setill no clients
        if *clients == 0 {
            drop(clients);
            sleep(Duration::from_secs(1800)).await;
            let clients = client_count.lock().await;
            //if there are still no clients after 30 minutes, send the off signal
            if *clients == 0 {
                //send server off signal
            }
        }
    }
}


//TODO
// Clean this up
#[tokio::main]
async fn main() {

    let listener = TcpListener::bind(LISTEN_PORT).await.unwrap();
    let client_count = Arc::new(Mutex::new(0 as u8));
    let server_on = Arc::new(Mutex::new(true));

    tokio::spawn(async move {
        server_timer(client_count.clone()).await
    });

    println!("entering loop");
    loop {
        let (socket, _) = listener.accept().await.unwrap();

        let server_signal = server_on.lock().await;
        if !*server_signal {
            //Not entirely sure this line is necessary, because it should go out of scope
            //after the thread is spawned, but for now better safe than sorry
            drop(server_signal);
            //send the server on signal
        }

        tokio::spawn(async move {
            handle_client(socket).await;
        });

    }
}