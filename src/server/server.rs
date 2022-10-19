use std::{env, collections::HashMap};

use rpubsub::{SocketAddress};

fn process_get(state: &mut topic::TopicsState, topic: &rpubsub::Topic, ip: &String, sequence_num: rpubsub::SequenceNum) -> 
                                                                Result<rpubsub::ReplyOption, rpubsub::ServiceError> {
    let res = topic::get_next_subscriber_update(state, topic, ip, sequence_num);
    return match res {
        Ok(opt) => Ok(rpubsub::ReplyOption::TUP(opt)),
        Err(err) => Err(err),
    }
}

fn process_put(state: &mut topic::TopicsState, topic: &rpubsub::Topic, content: &rpubsub::UpdateContent, _sequence_num: rpubsub::SequenceNum) -> 
                                                                Result<rpubsub::ReplyOption, rpubsub::ServiceError> {
    let res = topic::add_update(state, topic, content);
    return match res {
        Ok(_) => Ok(rpubsub::ReplyOption::NoOk),
        Err(err) => Err(err),
    }
}

fn process_sub(state: &mut topic::TopicsState, topic: &rpubsub::Topic, ip: &String) -> Result<rpubsub::ReplyOption, rpubsub::ServiceError> {
    let res = topic::add_subscription(state, topic, ip);
    return match res {
        Ok(_) => Ok(rpubsub::ReplyOption::NoOk),
        Err(err) => Err(err),
    }
}

fn process_unsub(state: &mut topic::TopicsState, topic: &rpubsub::Topic, ip: &String) -> Result<rpubsub::ReplyOption, rpubsub::ServiceError> {
    let res = topic::remove_subscription(state, topic, ip);
    return match res {
        Ok(_) => Ok(rpubsub::ReplyOption::NoOk),
        Err(err) => Err(err),
    }
}

fn process_up(state: &mut topic::TopicsState, ip: &String, sequence_nums: &HashMap<rpubsub::Topic, rpubsub::SequenceNum>) 
                                                        -> Result<rpubsub::ReplyOption, rpubsub::ServiceError> {
    for pair in sequence_nums {
        let res = topic::update_subscriber_update_ack(state, pair.0, ip, *pair.1);
        if res.is_err() {
            return Err(res.err().unwrap());
        }
    }

    return Ok(rpubsub::ReplyOption::NoOk);
}


fn process_request(state: &mut topic::TopicsState, request: &rpubsub::Message) -> (rpubsub::Message, String) {
    let mut client_ip = String::from("<UNKNOWN>");

    let result = match request {
        rpubsub::Message::GET { ip, sequence_num, topic } => { 
            client_ip = ip.clone(); process_get(state, &topic, &ip, *sequence_num) 
        },

        // TODO SEQUENCE ON PUT FOR IDEMPOTENCE
        rpubsub::Message::PUT { ip, sequence_num, topic, payload } => { 
            client_ip = ip.clone(); process_put(state, &topic, &payload, *sequence_num)
        },

        rpubsub::Message::SUB { ip, topic } => { 
            client_ip = ip.clone(); process_sub(state, &topic, &ip)
        },

        rpubsub::Message::UNSUB { ip, topic } => { 
            client_ip = ip.clone(); process_unsub(state, &topic, &ip) 
        },

        rpubsub::Message::UP { ip, sequence_nums } => {
            client_ip = ip.clone(); process_up(state, &ip, &sequence_nums)
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
        println!("Usage: server <IP> <BIND_PORT>")
    }

    let server_addr = SocketAddress{ ip: args[1].clone(), port: args[2].clone().parse::<u16>().unwrap() };

    let context = zmq::Context::new();

    let rep_socket = match context.socket(zmq::REP) {
                            Ok(socket) => socket,
                            Err(e) => {
                                println!("error: couldn't create socket: {}", e);
                                return;
                            },
                        };

    let mut state = topic::TopicsState::new();

    match rpubsub::bind_to(&rep_socket, &server_addr) {
        Ok(_) => println!("Server listening on {}:{}", server_addr.ip, server_addr.port),
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

        let (reply, client_ip) = process_request(&mut state, &request.unwrap());

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