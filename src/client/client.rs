use std::env;
use rpubsub::ServerAddress;
//use zmq;


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        println!("Wrong number of arguments");
        println!("Usage: server <IP> <BIND_PORT> <Msg>")
    }

    let server_addr = ServerAddress{ip: args[1].clone(), bind_port: args[2].clone().parse::<u16>().unwrap()};
    let context = zmq::Context::new();

    let req_socket = match context.socket(zmq::REQ) {
                            Ok(socket) => socket,
                            Err(e) => panic!("Creating REQ socket; {}", e),
                        };

    let endpoint = format!("tcp://{}:{}", server_addr.ip, server_addr.bind_port);

    match req_socket.connect(&endpoint) {
        Ok(()) => println!("Server listening at {}:{}", 
                            server_addr.ip, server_addr.bind_port),

        Err(e) => panic!("Binding socket address {}:{}; {}", 
                                server_addr.ip, server_addr.bind_port, e)
    };

    match req_socket.send("Olá", 0) {
        Err(e) => panic!("Error sending message {}", e),

        Ok(()) => ()
    };
    
    match req_socket.recv_msg(0) {
        Ok(message) => println!("{}", message.as_str().unwrap()),

        Err(e) => panic!("Error getting message {}", e)
    };
}