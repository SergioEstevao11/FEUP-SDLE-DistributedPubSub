use std::env;
use rpubsub::{SocketAddress};
//use zmq;


fn send_request(message: rpubsub::Message, req_socket: zmq::Socket) {

    // use json serialize
}

fn parse_operation(op: String) {

}

fn main() {
    //let m2 = rpubsub::Message::GET { sequence_num: 3, topic: String::from("HEY") };
    //let serialized_message = serde_json::to_string(&m2).unwrap();
    //println!("{}", serialized_message);
    //let deserialized_message: Message = serde_json::from_str(&serialized_message).unwrap();
        
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        println!("Wrong number of arguments");
        println!("Usage: server <IP> <SERVER_IP> <SERVER_PORT> <OP>");
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

    
    match req_socket.send("hey", 0) {
        Err(e) => panic!("Error sending message {}", e),

        Ok(()) => ()
    };
    
    match req_socket.recv_msg(0) {
        Ok(message) => println!("{}", message.as_str().unwrap()),

        Err(e) => panic!("Error getting message {}", e)
    };
}