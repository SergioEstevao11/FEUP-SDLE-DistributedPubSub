use rpubsub::{Message, SocketAddress};
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead,Write};
use std::path::Path;
use std::env;
use std::collections::HashMap;

const MAX_TRIES: i32 = 3;


//use zmq;

pub struct Client {
    pub ip: String,
    pub server_socket: SocketAddress,
    pub sequence_numbers: HashMap<String, u128>, //hashmap [topic] = sequence_number
    pub put_counters: HashMap<String, u128>, //hashmap [topic] = counter
}

impl Client {
    fn increment_sequence_numbers(&mut self, topic: &str){
        *(self.sequence_numbers).get_mut(topic).unwrap() += 1;

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
}

fn process_operation(client: &Client, op: &String) -> Result<Message, String> {
    //let v: Vec<&str> = line.split(" ").collect();
    
    //println!("{:#?}", v);

    /*
    if ["GET", "SUB", "UNSUB"].contains(&operands[1]) {
        if operands.len() < 2 {
            return Err(String::from("error: missing parameters"));
        }
    } else if operands[1] == "PUT" {
        if operands.len() < 3 {
            return Err(String::from("error: missing parameters"));
        }
    } else {
        return Err(String::from("error: missing operator"));
    }

    return match operands[0] {
        "GET" => Ok(Message::GET {
            ip: client.ip.clone(),
            sequence_num: client.sequence_numbers[operands[1]],
            topic: String::from(operands[1]),
        }),

        "PUT" => Ok(Message::PUT {
            ip: client.ip.clone(),
            sequence_num: client.put_counters[operands[1]],
            topic: String::from(operands[1]),
            payload: String::from(operands[2]),
        }),

        "SUB" => Ok(Message::SUB {
            ip: client.ip.clone(),
            topic: String::from(operands[1]),
        }),

        "UNSUB" => Ok(Message::UNSUB {
            ip: client.ip.clone(),
            topic: String::from(operands[1]),
        }),

        _ => Err(String::from("Unknown parameters")),
    };*/
}



fn main() {

    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
        line = String::from(line.trim());
        process_operation(client, &line);    
    }    /*
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        println!("Wrong number of arguments");
        println!("Usage: client <IP> <SERVER_IP> <SERVER_PORT>");
    }

    let server_addr = SocketAddress {
        ip: args[2].clone(),
        port: args[3].clone().parse::<u16>().unwrap(),
    };

    let context = zmq::Context::new();

    let req_socket = match context.socket(zmq::REQ) {
        Ok(socket) => socket,
        Err(e) => {
            println!("error: couldn't create socket: {}", e);
            return;
        },
    };

    match rpubsub::connect_to(&req_socket, &server_addr) {
        Ok(_) => println!("Connected to server listening on {}:{}", server_addr.ip, server_addr.port),
        Err(e) => { 
            println!("{}", e.to_string().as_str());
            return;
        },
    }

    let mut client = Client {
        ip: args[1].clone(),
        server_socket: server_addr,
        sequence_numbers: HashMap::new(),
        put_counters: HashMap::new(),
    };*/

    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    line = String::from(line.trim());

    let v: Vec<&str> = line.splitn(2, " ").collect();

    /*if v.len() == 2 && !v[1].chars().all(char::is_alphanumeric) {
        println!("Topics can not have non alphanumeric chars!");
        continue
    }*/

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
