use std::{env, collections::HashMap};
use std::fs;
use serde::{Deserialize, Serialize};

use rpubsub::{SocketAddress};



pub struct Server {
    pub socket_address: SocketAddress,
    pub state_path: String,
    pub state: topic::State,
}

fn get_state_file_content(server: &mut Server) {
    let server_path = String::from("./data/server_data/");
    server.state_path = server_path.clone() + "state.json";

    let res = fs::read_dir(&server_path);
    const WITH_STATE: bool = true;

    if res.is_err() {
        println!("info: server state not found. creating one..");
        fs::create_dir_all(&server_path);
        let serialized_state = serde_json::to_string(&server.state).unwrap();

        fs::write(&server.state_path, serialized_state);

    } else {
        println!("info: server state found. backing up..");
        let state_json = fs::read(&server.state_path);
        //String::from_utf8(fs::read(client_path + "state.json").unwrap()).unwrap();
        //
        server.state = serde_json::from_slice(&state_json.unwrap().as_slice()).unwrap();
    }
}

fn process_get(server: &mut Server, topic: &rpubsub::Topic, ip: &String, sequence_num: rpubsub::SequenceNum) -> 
                                                                Result<rpubsub::ReplyOption, rpubsub::ServiceError> {
    let res = topic::get_next_subscriber_update(&mut server.state, topic, ip, sequence_num, &server.state_path);
    return match res {
        Ok(opt) => Ok(rpubsub::ReplyOption::TUP(opt)),
        Err(err) => Err(err),
    }
}

fn process_put(server: &mut Server, topic: &rpubsub::Topic, content: &rpubsub::UpdateContent, _sequence_num: rpubsub::SequenceNum) -> 
                                                                Result<rpubsub::ReplyOption, rpubsub::ServiceError> {
    let res = topic::add_update(&mut server.state, topic, content, &server.state_path);
    return match res {
        Ok(_) => Ok(rpubsub::ReplyOption::NoOk),
        Err(err) => Err(err),
    }
}

fn process_sub(server: &mut Server, topic: &rpubsub::Topic, ip: &String) -> Result<rpubsub::ReplyOption, rpubsub::ServiceError> {
    let res = topic::add_subscription(&mut server.state, topic, ip, &server.state_path);
    return match res {
        Ok(_) => Ok(rpubsub::ReplyOption::NoOk),
        Err(err) => Err(err),
    }
}

fn process_unsub(server: &mut Server, topic: &rpubsub::Topic, ip: &String) -> Result<rpubsub::ReplyOption, rpubsub::ServiceError> {
    let res = topic::remove_subscription(&mut server.state, topic, ip, &server.state_path);
    return match res {
        Ok(_) => Ok(rpubsub::ReplyOption::NoOk),
        Err(err) => Err(err),
    }
}

fn process_up(server: &mut Server, ip: &String, sequence_nums: &HashMap<rpubsub::Topic, rpubsub::SequenceNum>) 
                                                        -> Result<rpubsub::ReplyOption, rpubsub::ServiceError> {
    for pair in sequence_nums {
        let res = topic::update_subscriber_update_ack(&mut server.state, pair.0, ip, *pair.1,  &server.state_path);
        if res.is_err() {
            return Err(res.err().unwrap());
        }
    }

    return Ok(rpubsub::ReplyOption::NoOk);
}


fn process_request(server: &mut Server, request: &rpubsub::Message) -> (rpubsub::Message, String) {
    let mut client_ip = String::from("<UNKNOWN>");

    let result = match request {
        rpubsub::Message::GET { ip, sequence_num, topic } => { 
            client_ip = ip.clone(); process_get(server, &topic, &ip, *sequence_num) 
        },

        // TODO SEQUENCE ON PUT FOR IDEMPOTENCE
        rpubsub::Message::PUT { ip, sequence_num, topic, payload } => { 
            client_ip = ip.clone(); process_put(server, &topic, &payload, *sequence_num)
        },

        rpubsub::Message::SUB { ip, topic } => { 
            client_ip = ip.clone(); process_sub(server, &topic, &ip)
        },

        rpubsub::Message::UNSUB { ip, topic } => { 
            client_ip = ip.clone(); process_unsub(server, &topic, &ip) 
        },

        rpubsub::Message::UP { ip, sequence_nums } => {
            client_ip = ip.clone(); process_up(server, &ip, &sequence_nums)
        },

        rpubsub::Message::NOMSG => {
            Err(rpubsub::ServiceError::UNKNOMSG)
        }
        // This one never happens
        _ => Ok(rpubsub::ReplyOption::NoOk)
    };

    (rpubsub::Message::REP { result: result }, client_ip)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("wrong number of arguments");
        println!("Usage: server <IP> <BIND_PORT>");
        return;
    }
    let mut server = Server {
        socket_address: SocketAddress{ 
            ip: args[1].clone(), 
            port: args[2].clone().parse::<u16>().unwrap() 
        },
        state_path: String::new(),
        state: topic::State { topics: topic::Topics::new() },
    };


    let context = zmq::Context::new();

    let rep_socket = match context.socket(zmq::REP) {
                            Ok(socket) => socket,
                            Err(e) => {
                                println!("error: couldn't create socket: {}", e);
                                return;
                            },
                        };


    get_state_file_content(&mut server);

    match rpubsub::bind_to(&rep_socket, &server.socket_address) {
        Ok(_) => println!("Server listening on {}:{}", server.socket_address.ip, server.socket_address.port),
        Err(e) => { 
            println!("{}", e.to_string().as_str());
            return;
        },
    }

    loop {
        let request = match rpubsub::receive_message_from(&rep_socket) {
            Ok(message) => {
                println!("Received request: {}", message.to_string());
                Some(message)
            },
            Err(e) => {
                println!("{}", e.to_string());

                match e {
                    rpubsub::IOError::EDSL(_) => Some(rpubsub::Message::NOMSG),
                    _ => None
                }
            },
        };

        if request.is_none() {
            continue;
        }

        let (reply, client_ip) = process_request(&mut server, &request.unwrap());

        match rpubsub::send_message_to(&rep_socket, &reply) {
            Ok(_) => {
                println!("Sent reply to client {}: {}", client_ip, reply.to_string());
            },
            Err(e) => {
                println!("{}", e.to_string());
            },
        };
    }
}