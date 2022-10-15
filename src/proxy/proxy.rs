use std::env;
use rpubsub::SocketAddress;
//use zmq;


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("wrong number of arguments");
        println!("Usage: server <IP> <BIND_PORT>")
    }

    let server_addr = SocketAddress{ip: args[1].clone(), port: args[2].clone().parse::<u16>().unwrap()};
    let context = zmq::Context::new();

    let router_socket = match context.socket(zmq::ROUTER) {
                            Ok(socket) => socket,
                            Err(e) => panic!("Creating router socket; {}", e),
                        };

    let endpoint = format!("tcp://{}:{}", server_addr.ip, server_addr.port);

    match router_socket.bind(&endpoint) {
        Ok(()) => println!("Server listening at {}:{}", 
                            server_addr.ip, server_addr.port),

        Err(e) => panic!("Binding socket address {}:{}; {}", 
                                server_addr.ip, server_addr.port, e)
    };
}