extern crate serde;
extern crate serde_json;

use rpubsub::{Message, SocketAddress};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;


const MAX_TRIES: u32 = 3;
const TIMEOUT_MS: i64 = 3000;

//use zmq;

pub struct Client {
    pub ip: String,
    pub state: State,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub sequence_numbers: HashMap<String, u128>, //hashmap [topic] = sequence_number
    pub put_counters: HashMap<String, u128>,     //hashmap [topic] = counter
}

fn get_state_file_content(client: &mut Client) -> bool {
    let client_path = format!("./clients/{}/", client.ip);
    let state_path = client_path.clone() + "state.json";

    let res = fs::read_dir(&client_path);
    const WITH_STATE: bool = true;

    if res.is_err() {
        println!("info: client state not found. creating one..");
        fs::create_dir_all(&client_path);
        let serialized_state = serde_json::to_string(&client.state).unwrap();

        fs::write(state_path, serialized_state);
        !WITH_STATE
    } else {
        println!("info: client state found. backing up..");
        let state_json = fs::read(state_path);
        //String::from_utf8(fs::read(client_path + "state.json").unwrap()).unwrap();
        //
        let state: State = serde_json::from_slice(&state_json.unwrap().as_slice()).unwrap();
        client.state = state;
        WITH_STATE
    }
}

fn save_state(client: & Client) -> Result<(), io::Error> {
    let client_path = format!("./clients/{}/", client.ip);
    let serialized_state = serde_json::to_string(&client.state).unwrap();
    
    match fs::create_dir_all(&client_path){
        Err(e) => return Err(e),
        Ok(_) => (),
    }

    fs::write(client_path.clone() + "state.json", serialized_state)
}

fn process_operation(client: &mut Client, op: &String) -> Result<Message, String> {
    let operands: Vec<&str> = op.split(" ").collect();

    println!("{:#?}", operands);

    if operands.len() < 2 {
        return Err(String::from("error: no operation was inputed"));
    }

    match operands[0] {
        "GET" | "SUB" | "UNSUB" => {
            if operands.len() != 2 {
                return Err(String::from("error: missing parameters"));
            }
        }

        "PUT" => {
            if operands.len() != 3 {
                return Err(String::from("error: missing parameters"));
            }
        }

        _ => (),
    };

    // TODO THIS
    /*
                if !client.sequence_numbers.contains_key(&topic) {
                client.sequence_numbers.insert(topic.clone(), 0);
                println!("info: no sequence number is associated to topic {}. Creating one", topic);
            }

    */
    let topic = String::from(operands[1]);
    match operands[0] {
        "SUB" => Ok(Message::SUB {
            ip: client.ip.clone(),
            topic: topic,
        }),

        "UNSUB" => Ok(Message::UNSUB {
            ip: client.ip.clone(),
            topic: topic,
        }),

        "PUT" => {
            // The client doesn't need to subscribe to put a message on a topic
            if !client.state.put_counters.contains_key(&topic) {
                client.state.put_counters.insert(topic.clone(), 0);
                println!(
                    "info: no put counter is associated to topic {}. Creating one",
                    topic
                );
            }
            // TODO
            let payload = String::from(operands[2]);
            Ok(Message::PUT {
                ip: client.ip.clone(),
                topic: topic.clone(),
                sequence_num: *client.state.put_counters.get(&topic).unwrap(),
                payload: payload,
            })
        }

        "GET" => {
            if !client.state.sequence_numbers.contains_key(&topic) {
                return Err::<Message, String>(String::from(format!(
                    "error: not subscribed to topic {}",
                    topic
                )));
            }

            Ok(Message::GET {
                ip: client.ip.clone(),
                sequence_num: *client.state.sequence_numbers.get(&topic).unwrap(),
                topic: topic,
            })
        }
        _ => Err(String::from("error: unknown operation")),
    }
}

fn process_reply(client: &mut Client, request: &Message, reply: &Message) {
    match reply {
        Message::REP { result } => {
            match request {
                Message::SUB { ip: _, topic } => {
                    if result.is_err() {
                        println!(
                            "error: cannot subsribe to topic. topic: {}; reason: {:?}",
                            topic,
                            result.as_ref().unwrap_err()
                        );
                        return;
                    }

                    if !client.state.sequence_numbers.contains_key(topic) {
                        client.state.sequence_numbers.insert(topic.clone(), 0);
                        println!(
                            "info: no sequence number is associated to topic {}. Creating one",
                            topic
                        );
                    }
                }

                Message::UNSUB { ip: _, topic } => {
                    // This error should never happen
                    if result.is_err() {
                        println!(
                            "error: cannot unsubsribe to topic. topic: {}; reason: {:?}",
                            topic,
                            &result.as_ref().unwrap_err()
                        );
                        return;
                    }

                    if client.state.sequence_numbers.contains_key(topic) {
                        client.state.sequence_numbers.remove(topic);
                        println!(
                            "info: removed sequence number associated to topic {}",
                            topic
                        );
                    }
                }

                Message::PUT {
                    ip: _,
                    topic,
                    sequence_num: _,
                    payload: _,
                } => {
                    if result.is_err() {
                        println!(
                            "error: cannot put message on topic. topic: {}; reason: {:?}",
                            topic,
                            &result.as_ref().unwrap_err()
                        );
                        return;
                    }

                    if !client.state.put_counters.contains_key(topic) {
                        client.state.put_counters.insert(topic.clone(), 0);
                        println!(
                            "info: no put counter is associated to topic {}. Creating one",
                            topic
                        );
                    }

                    let reply_option = result.as_ref().unwrap();

                    match reply_option {
                        rpubsub::ReplyOption::NoOk => {
                            if let Some(counter) = client.state.put_counters.get_mut(topic) {
                                *counter += 1;
                            }
                        }
                        rpubsub::ReplyOption::TUP(_) => (),
                    }
                }

                Message::GET {
                    ip: _,
                    topic,
                    sequence_num,
                } => {
                    if result.is_err() {
                        println!(
                            "error: cannot get message from topic. topic: {}; reason: {:?}",
                            topic,
                            &result.as_ref().unwrap_err()
                        );
                        return;
                    }

                    let reply_option = result.as_ref().unwrap();

                    match reply_option {
                        rpubsub::ReplyOption::TUP(tup) => {
                            if *sequence_num == tup.1 {
                                if tup.0.is_some() {
                                    if let Some(counter) =
                                        client.state.sequence_numbers.get_mut(topic)
                                    {
                                        *counter += 1;
                                    }
                                }
                            } else {
                                println!("error: receiving outdated messages. topic: {} seq_nums(client, server): ({}, {})", topic, sequence_num, tup.1);
                                println!("info: Syncronizing local sequence number with server");

                                if let Some(counter) = client.state.put_counters.get_mut(topic) {
                                    *counter = tup.1;
                                }
                            }
                        }
                        rpubsub::ReplyOption::NoOk => (),
                    }
                }

                // Message::UP { ip: _, sequence_nums } => {},
                _ => (),
            };
        }
        _ => (),
    }
}

pub fn send_message_with_retries(socket: &zmq::Socket, message: &Message) -> Message {
    let mut poll_list = [socket.as_poll_item(zmq::POLLIN)];
    let mut tries_counter = 0;

    println!("Polling...");

    loop {
        match rpubsub::send_message_to(socket, message) {
            Ok(_) => println!("Sent message"),
            Err(_) => panic!("Error sending message"),
        };

        match zmq::poll(&mut poll_list, TIMEOUT_MS) {
            Ok(_) => {
                match rpubsub::receive_message_from(socket) {
                    Ok(reply) => return reply,
                    Err(_) => panic!("ssss"),
                };
            }
            Err(_) => {
                if tries_counter > MAX_TRIES {
                    panic!("error: exceeded tries in polling");
                } else {
                    tries_counter += 1;
                    println!("info: Poll timeout exceeded. Retrying")
                }
            }
        }
    }
}

fn main() {
    println!("{}", std::env::current_dir().unwrap().to_str().unwrap());
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        println!("Wrong number of arguments");
        println!("Usage: client <IP> <SERVER_IP> <SERVER_PORT>");
    }

    let server_addr = SocketAddress {
        ip: args[2].clone(),
        port: args[3].clone().parse::<u16>().unwrap(),
    };

    let mut client = Client {
        ip: args[1].clone(),
        state: State {
            sequence_numbers: HashMap::new(),
            put_counters: HashMap::new(),
        },
    };

    let with_state = get_state_file_content(&mut client);

    let context = zmq::Context::new();

    let req_socket = match context.socket(zmq::REQ) {
        Ok(socket) => socket,
        Err(e) => {
            println!("error: couldn't create socket: {}", e);
            return;
        }
    };

    match rpubsub::connect_to(&req_socket, &server_addr) {
        Ok(_) => println!(
            "Connected to server listening on {}:{}",
            server_addr.ip, server_addr.port
        ),
        Err(e) => {
            println!("{}", e.to_string().as_str());
            return;
        }
    };

    if with_state {
        let message = rpubsub::Message::UP {
            ip: client.ip.clone(),
            sequence_nums: client.state.sequence_numbers.clone(),
        };
        let reply = send_message_with_retries(&req_socket, &message);
        println!("Received reply: {}", reply.to_string());
    }

    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
        line = String::from(line.trim());

        match process_operation(&mut client, &line) {
            Ok(request) => {
                let reply = send_message_with_retries(&req_socket, &request);

                println!("Received reply {}", reply.to_string());

                process_reply(&mut client, &request, &reply);

                match save_state(&client){
                    Err(e) => println!("error: while saving state. e: {}", e),
                    Ok(_) => (),
                }
            }
            Err(e) => println!("{}", e),
        }
    }
}
