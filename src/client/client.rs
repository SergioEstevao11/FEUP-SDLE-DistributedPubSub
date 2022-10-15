use std::env;
use rpubsub::{ SocketAddress, Message };
//use zmq;


fn send_request(message: rpubsub::Message, req_socket: zmq::Socket) {
    todo!()
}

fn parse_operation(op: String) {
    todo!()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        println!("Wrong number of arguments");
        println!("Usage: client <IP> <SERVER_IP> <SERVER_PORT> <OP>");
        println!("OP: GET|<TOPIC>  
                    | PUT|<TOPIC>|<MSG>
                    | SUB|<TOPIC>
                    | UNSUB|<TOPIC>");
    }

    let server_addr= SocketAddress { 
                                            ip: args[2].clone(), 
                                            port: args[3].clone().parse::<u16>().unwrap()
                                        };
    let context = zmq::Context::new();

    let req_socket = match context.socket(zmq::REQ) {
                            Ok(socket) => socket,
                            Err(e) => panic!("Creating REQ socket; {}", e),
                        };

    let endpoint = format!("tcp://{}:{}", server_addr.ip, server_addr.port);

    match req_socket.connect(&endpoint) {
        Ok(()) => println!("Server listening at {}:{}", 
                            server_addr.ip, server_addr.port),

        Err(e) => panic!("Binding socket address {}:{}; {}", 
                                server_addr.ip, server_addr.port, e)
    };

    
    let msg = Message::GET { ip: String::from("127.0.0.1"), sequence_num: 1, 
                                        topic: String::from("hey") };

    match rpubsub::send_message_to(&req_socket, msg) {
        Ok(_) => (),
        Err(_) => panic!("Error sending message"),
    };
    
    match rpubsub::receive_message_from(&req_socket) {
        Ok(message) => println!("{}", message.to_string()),

        Err(_e) => panic!("Error getting message")
    };
}