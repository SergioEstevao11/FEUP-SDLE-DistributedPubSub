extern crate serde;
extern crate serde_json;

use rpubsub::{Message, SocketAddress};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::Path;

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

/*sequence_numtopic).unwrap() += 1;

    }

    fn increment_put_counter(&mut self, topic: &str){
        *(self.put_counters).get_mut(topic).unwrap() += 1;
    }

    fn read_savefile(&mut self) {
        let binding = self.create_path();
        let path = Path::new(binding.as_str());
        let _ = match File::open(&path) {
            Err(_) => self.create_savefile(true),
            Ok(_) => self.recovery(),
        };
    }

    fn create_savefile(&self,flag: bool) {
        if flag == true {
            println!("No file found for this client - creating new file...");
        }
        let binding = self.create_path();
        let path = Path::new(binding.as_str());
        let display = path.display();
        let _file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}: {}", display, why),
            Ok(_file) => _file,
        };
    }

    fn create_path(&self) -> String {
        let dir = "./savefiles/".to_owned();
        let path_name = dir.clone() + &self.ip;
        return path_name.to_string();
    }

    fn recovery(&mut self) {
        self.sequence_numbers.clear();
        let binding = self.create_path();
        if let Ok(lines) = self.read_lines(binding) {
            for line in lines {
                if let Ok(ip) = line {
                    let temp: Vec<&str> = ip.split(':').collect();
                    let topic = temp[0].to_string();
                    let sequence_num = temp[1].parse::<u128>().unwrap();
                    self.sequence_numbers.insert(topic,sequence_num);
                }
            }
        }
    }
    fn read_lines<P>(&self,filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }

    fn write_newline(&self,topic: &str, num: &str) {
        let binding = self.create_path();
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(binding)
            .unwrap();

        // write a newline to the file
        if let Err(e) = writeln!(file, "{}:{}", topic,num) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }

    fn save_client_state(&mut self, topic: &str){
        if self.sequence_numbers.contains_key(topic) {
            for (key, val) in self.sequence_numbers.iter_mut() {
                if key == topic{
                    *val += 1;
                }
            }    } else {
                self.sequence_numbers.insert((&"topic").to_string(), 1);
        }
        self.create_savefile(false);
        for (key,value) in &self.sequence_numbers {
            self.write_newline( &key, &value.to_string());
        }
    }
}*/

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
                                    if let Some(counter) = client.state.sequence_numbers.get_mut(topic) {
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
            Err(e) => {
                if tries_counter > MAX_TRIES {
                    panic!("error: exceeded tries in polling");
                } else {
                    tries_counter += 1;
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
            }
            Err(e) => println!("{}", e),
        }
    }

    /*
    client.read_savefile();

    let mut poll_list = [req_socket.as_poll_item(zmq::POLLIN)];

    //Main message loop
    loop {
        let op: String = env::args().collect();
        let msg: Message;
        let msg = match parse_operation(&mut client, op) {
            Ok(ret_msg) => ret_msg,
            Err(_) => {
                println!("Invalid operation arguments");
                println!(
                    "OP: GET|<TOPIC>
                    | PUT|<TOPIC>|<MSG>
                    | SUB|<TOPIC>
                    | UNSUB|<TOPIC>"
                );
                continue;
            }
        };


        match rpubsub::send_message_to(&req_socket, &msg){
            Ok(_) => println!("Sent message!"),
            Err(_) => panic!("Error sending message"),
        };

        //client.save_client_state(topic);


        //poll
        let mut poll_list = [req_socket.as_poll_item(zmq::POLLIN)];
        let mut tries_counter = 0;

        let mut revent_list = Vec::new();
        println!("Polling...");
        loop{
            match zmq::poll(&mut poll_list, 3000) {
                Ok(_) => {
                    for poll_item in poll_list.into_iter() {
                        revent_list.push(poll_item.get_revents());
                    }
                    break;
                },
                Err(e) => {
                    if tries_counter > MAX_TRIES{
                        panic!("Error: exceeded tries in polling");
                    }
                    else{
                        tries_counter+=1;
                        match rpubsub::send_message_to(&req_socket, &msg){
                            Ok(_) => println!("Sent message"),
                            Err(_) => panic!("Error sending message"),
                        };

                    }
                },
            }
        }
        //receive message

        match rpubsub::receive_message_from(&req_socket){
            Ok(rec_msg) => {
                match msg{
                    Message::GET { ip, topic, sequence_num } => {
                        if client.sequence_numbers[&topic] == sequence_num{
                            println!("Received message! {}", rec_msg.to_string());
                            client.increment_sequence_numbers(&topic);
                            client.save_client_state(&topic);
                        }
                        else{
                            println!("Received message, but outdated content! {}", rec_msg.to_string());
                        }
                    }
                    Message::PUT { ip, topic, sequence_num, payload } => {
                        println!("Received message! {}", rec_msg.to_string());
                        client.increment_put_counter(&topic);
                    }
                    Message::SUB { ip, topic } => {
                        println!("Received message! {}", rec_msg.to_string());

                        if !client.sequence_numbers.contains_key(&topic) {
                            client.sequence_numbers.insert(topic.clone(), 0);
                        }

                        if !client.put_counters.contains_key(&topic) {
                            client.put_counters.insert(topic.clone(), 0);
                        }
                    }
                    _ => {println!("Received message! {}", rec_msg.to_string());}
                }

            }
            Err(e) => println!("Error! {}", e.to_string()),
        };



    }*/
}
