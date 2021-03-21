use std::net::{TcpListener, TcpStream};
use std::io::{Write, Read};
use std::thread;
use std::time::{Duration, SystemTime};
use std::sync::mpsc;


fn main() {

    let listener = match TcpListener::bind("127.0.0.1:8080") {
        Ok(t) => t,
        Err(e) => panic!("Listener failed to initialize. Error: {}", e),
    };
    let destination:String = String::from("127.0.0.1:8080");
    let (sender1, receiver1) = mpsc::sync_channel(0);
    let (sender2, receiver2) = mpsc::sync_channel(0);
    //process connections continuously in its own thread
    thread::spawn(||match listener_thread(listener, destination,
                                          receiver2, sender1) {
        Ok(..) => (),
        Err(e) => panic!("{}", e),
    });
    //Separate thread checking how long the server has been inactive
    thread::spawn(|| {server_switch_thread(receiver1, sender2)});
    loop{}
}
//To be called by a listener thread. Listens on a port and handles connections
fn listener_thread(listener: TcpListener, dest: String, receiver: mpsc::Receiver<bool>,
                   sender: mpsc::SyncSender<bool>) -> std::io::Result<()> {
    let mut server_on = true;
    //this loop checks that the streams are valid, sets the server_on bool, and handles connections.
    //listener.incoming() causes this to loop indefinitely (i think)
    for stream in listener.incoming() {
        //if stream is valid, set it to stream_source as a tcp stream instead of a result
        if let Ok(t) = stream {
            let stream_source = t;
            if let Ok(y) = TcpStream::connect(&dest) {
                //if the connection is successful, set stream_dest to that connection
                let stream_dest = y;
                //the receiver attempts to get a message from the server_switch thread
                //about the status of the server, then sets server_on to that status
                //true if the server is on, false if it is off.
                if let Ok(b) = receiver.recv() {
                    server_on = b;
                }
                //attempts to pass packets between the two streams
                match handle_tcp_connection(stream_source, stream_dest, server_on) {
                    Ok(..) => (),
                    Err(..) => (),
                }
                //handle_udp_connection (should this be its own thread?)
                match sender.send(true) {
                    Ok(..) => (),
                    Err(..) => (),
                } //sends an update to the server_switch indicating access
            }
        }
    }
    //should never actually reach here so it should probably return error but :/
    Ok(())
}

//to be called by a server_switch thread. Continuously checks if the listener thread
//has sent an update message. If 30 minutes have passed with no update message
//an off signal should be sent to the destination machine.
fn server_switch_thread(receiver: mpsc::Receiver<bool>, sender: mpsc::SyncSender<bool>) {
    let mut time_last_handled = SystemTime::now();
    let mut server_on = true;
    //server_switch loop
    loop {

        //check if there's been an update message, if there has been set update = true
        //if not, update = false
        let update = match receiver.try_recv() {
            Ok(..) => true,
            _ => false,
        };
        //if there's been an update, make time_last_handled the current time
        //since the listener thread is currently handling connections
        if update {
            time_last_handled = SystemTime::now();
        }
        // if it has been more than 30 minutes since the last update, send an off signal
        //to the server machine.
        if time_last_handled.elapsed().unwrap() > Duration::from_secs(1800) {
            //send off signal
            server_on = false;
        }
        //try to send a message indicating to the listener whether or not the server is on
        //if the listener is not looking for this message, the thread is not blocked
        match sender.try_send(server_on) {
            _ => (),
        }
    }
}

//wake the server machine if its off and pass the connection :)
fn handle_tcp_connection(mut stream_source: TcpStream, mut stream_dest: TcpStream, server_on: bool)
                         -> std::io::Result<()> {
    if !server_on {
        //send on signal
    }
    let mut buffer_source: Vec<u8> = vec![0; 4096];
    let mut buffer_dest: Vec<u8> = vec![0; 4097];
    stream_source.read_to_end(&mut buffer_source)?;
    stream_dest.read_to_end(&mut buffer_dest)?;
    stream_dest.write(&mut buffer_source)?;
    stream_source.write(&mut buffer_dest)?;
    Ok(())
}