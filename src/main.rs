use std::net::{TcpListener, TcpStream};
use std::io::{Write, Read};
use std::thread;
use std::time::{Duration, SystemTime};
use std::sync::{Arc, Mutex};


fn main() {
    let server_on = Arc::new(Mutex::new(true));
    let server_update = Arc::new(Mutex::new(false));
    let destination:String = String::from("127.0.0.1:8080");
    let source:String = String::from("127.0.0.1:8081");
    let source_listener = TcpListener::bind(&source).unwrap();
    let dest_listener = TcpListener::bind(&destination).unwrap();

    //why the fuck is this necessary
    //this cant possibly be necessary
    let server_on1 = server_on.clone();
    let server_on2 = server_on.clone();
    let server_on3 = server_on.clone();
    let server_update1 = server_update.clone();
    let server_update2 = server_update.clone();
    let server_update3 = server_update.clone();
    //process connections continuously in its own thread
    thread::spawn(move ||{
        listener_thread(source_listener, destination,
                        server_on1.clone(),
                        server_update1.clone())
    });
    thread::spawn(move||listener_thread(dest_listener, source, server_on2.clone(),
    server_update2.clone()));
    //Separate thread checking how long the server has been inactive
    thread::spawn(move || {server_switch_thread(server_on3.clone(),
                                           server_update3.clone())});
    loop{}
}
//To be called by a listener thread. Listens on a port and handles connections
fn listener_thread(listener: TcpListener, dest: String, server_on: Arc<Mutex<bool>>,
                    server_update: Arc<Mutex<bool>>) {
    //this loop checks that the streams are valid, sets the server_on bool, and handles connections.
    //listener.incoming() causes this to loop indefinitely (i think)
    for stream in listener.incoming() {
        //if stream is valid, set it to stream_source as a tcp stream instead of a result
        let stream_source = match stream {
            Ok(t) => t,
            Err(e) => panic!("invalid TCP listener! Error: {} ", e),
        };
        //if the connection is successful, set stream_dest to that connection
        let stream_dest = match TcpStream::connect(&dest) {
            Ok(s) => s,
            Err(e) => panic!("Unable to connect to address! Error: {}", e),
        };
        //the receiver attempts to get a message from the server_switch thread
        //about the status of the server, then sets server_on to that status

        //attempts to pass packets between the two streams
        match handle_tcp(stream_source, stream_dest, server_on.clone()) {
            Ok(..) => (),
            Err(..) => panic!("Could not handle tcp connection!"),
        }
        //handle_udp_connection (should this be its own thread?)
        {
            let mut update = server_update.lock().unwrap();
            *update = true;
        }
    }
}

//to be called by a server_switch thread. Continuously checks if the listener thread
//has sent an update message. If 30 minutes have passed with no update message
//an off signal should be sent to the destination machine.
fn server_switch_thread(server_on: Arc<Mutex<bool>>, server_update: Arc<Mutex<bool>>) {
    let mut time_last_handled = SystemTime::now();
    //server_switch loop
    loop {
        //if there's been an update, make time_last_handled the current time
        //since the listener thread is currently handling connections
        {
            let mut update = server_update.lock().unwrap();
            if *update{
                time_last_handled = SystemTime::now();
            }
            *update = false;
        }
        // if it has been more than 30 minutes since the last update, send an off signal
        //to the server machine.
        if time_last_handled.elapsed().unwrap() > Duration::from_secs(1800) {
            //send off signal
            let mut server_signal = server_on.lock().unwrap();
            *server_signal = false;
        }
    }
}

//wake the server machine if its off and pass the packet :)
fn handle_tcp(mut stream_source: TcpStream, mut stream_dest: TcpStream, server_on: Arc<Mutex<bool>>)
                         -> std::io::Result<()> {
    {
        let server_status = server_on.lock().unwrap();
        if !*server_status {
            //send on signal
        }
    }
    let mut buffer_source: Vec<u8> = vec![0; 4096];
    stream_source.read_to_end(&mut buffer_source)?;
    stream_dest.write(&mut buffer_source)?;
    Ok(())
}
