use std::env;
use rpubsub::ServerAddress;
//use zmq;


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("wrong number of arguments");
        println!("Usage: server <IP> <BIND_PORT>")
    }

    let server_addr = ServerAddress{ip: args[1].clone(), bind_port: args[2].clone().parse::<u16>().unwrap()};
    let context = zmq::Context::new();

    let rep_socket = match context.socket(zmq::REP) {
                            Ok(socket) => socket,
                            Err(e) => panic!("Creating router socket; {}", e),
                        };

    let endpoint = format!("tcp://{}:{}", server_addr.ip, server_addr.bind_port);

    match rep_socket.bind(&endpoint) {
        Ok(()) => println!("Server listening at {}:{}", 
                            server_addr.ip, server_addr.bind_port),

        Err(e) => panic!("Binding socket address {}:{}; {}", 
                                server_addr.ip, server_addr.bind_port, e)
    };

    loop {
        match rep_socket.recv_msg(0) {
            Ok(msg) => println!("Received {}", msg.as_str().unwrap()),
            Err(e) => panic!("Error in recv {}", e)
        };

        match rep_socket.send("Hello", 0){
            Err(e) => panic!("Error in send {}", e),
            _ => ()
        }
    }
}