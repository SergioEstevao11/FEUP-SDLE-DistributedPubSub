
#![allow(dead_code)]
#![allow(unused_variables)]
use std::time;
use std::thread;
//use std::{env, collections::VecDeque};

use rpubsub::{SocketAddress};

/* 

fn do_for_get(Message::GET { ip, sequence_num, topic }: Message) {
    todo!();
}

fn do_for_put(Message::PUT { ip, sequence_num, topic, payload }: Message) {
    todo!();
    
}

fn do_for_sub(Message::SUB { ip, topic }: Message) {
    todo!();    
}

fn do_for_unsub(Message::UNSUB { ip, topic }: Message) {
    todo!();
}

fn do_for_up(Message::UP { ip, sequence_num }: Message) {
    todo!();
}

fn do_for_rep(Message::REP { ip, status }: Message) {
    todo!();
}

fn read_request(rep_socket: &zmq::Socket) {
    todo!()
}

fn send_reply(rep_socket: &zmq::Socket) {
    todo!()
}*/

//type TopicsState = HashMap<Topic, TopicInfo>;

/* 
fn main() {
    
    let server_addr = SocketAddress{ip: String::from("127.0.0.2"), port: 9999};
    let context = zmq::Context::new();

    let rep_socket = match context.socket(zmq::REP) {
                            Ok(socket) => socket,
                            Err(e) => panic!("Creating router socket; {}", e),
                        };

    let endpoint = format!("tcp://{}:{}", server_addr.ip, server_addr.port);

    match rep_socket.bind(&endpoint) {
        Ok(()) => println!("Server listening at {}:{}", 
                            server_addr.ip, server_addr.port),

        Err(e) => panic!("Binding socket address {}:{}; {}", 
                                server_addr.ip, server_addr.port, e)
    };

    loop {
        let mut msg = zmq::Message::new();
        let e2 = rep_socket.recv(&mut msg, 0);
        println!("{}", msg.as_str().unwrap());
        let time = time::Duration::from_millis(5000);
        thread::sleep(time);
        println!("{}", "no more sleep");
        let e = rep_socket.send("bye", 0);

        match e {
            Ok(_) => println!("NO ERROR send"),
            Err(_) => println!("ERROR send"),
        }
    }
}*/


fn main() {


    /*let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("wrong number of arguments");
        println!("Usage: server <IP> <BIND_PORT>")
    }

    let server_addr = SocketAddress{ip: args[1].clone(), port: args[2].clone().parse::<u16>().unwrap()};
    let context = zmq::Context::new();

    let rep_socket = match context.socket(zmq::REP) {
                            Ok(socket) => socket,
                            Err(e) => panic!("Creating router socket; {}", e),
                        };

    let endpoint = format!("tcp://{}:{}", server_addr.ip, server_addr.port);

    match rep_socket.bind(&endpoint) {
        Ok(()) => println!("Server listening at {}:{}", 
                            server_addr.ip, server_addr.port),

        Err(e) => panic!("Binding socket address {}:{}; {}", 
                                server_addr.ip, server_addr.port, e)
    };

    loop {
        match rpubsub::receive_message_from(&rep_socket) {
            Ok(message) => println!("{}", message.to_string()),
            Err(_) => (),
        }

        let message = rpubsub::Message::PUT { ip: String::from("127.0.0.2"), sequence_num: 3, 
                                                    topic: String::from("hey"), payload: String::from("eheheheh") };
        match rpubsub::send_message_to(&rep_socket, message) {
            Ok(_) => (),
            Err(_) => ()
        }
    }*/

}