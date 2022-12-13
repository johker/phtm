extern crate pushr;
extern crate rand;
extern crate time;

#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;

use crate::num_traits::{FromPrimitive, ToPrimitive};

use std::sync::mpsc;
use std::thread;
use std::env;

// Include auto-generated file
mod node;
#[path = "../../msg/rs/msg.rs"]
mod shared;

#[path = "../tests/temporal_memory_test.rs"]
mod test;

use crate::node::control::{Event, State};
use crate::node::execution::PushExecutor;
use crate::node::message::Message;
use crate::node::source::Source;
use crate::pushr::push::interpreter::PushInterpreter;
use crate::pushr::push::instructions::InstructionSet;
use crate::pushr::push::item::Item;
use crate::pushr::push::vector::{BoolVector, IntVector};
use crate::pushr::push::graph::Graph;
use crate::pushr::push::io::PushMessage;
use crate::shared::msg::{MessageCommand, MessageKey, MessageType};
use crate::shared::msg::{
    CMD_OFFSET, DEF_PL_SIZE, ID_OFFSET, KEY_OFFSET, PAYLOAD_OFFSET, TYPE_OFFSET,
};

use crate::test::inject_activate_predicted_column_graph;


fn main() {

    println!("");
    println!("PHTM");
    println!("");

    print!("Initializing Push Executor ... ");
    let args: Vec<String> = env::args().collect(); 
    if args.len() < 2 {
        println!("failed");
        println!("No input provided");
        return;
    }
    let input = args[1].clone();

    let mut executor = PushExecutor::new();
    executor.initialize();
    println!("ok");

    print!("Loading program ... ");
    // Load program from input
    executor.load(input);
    // Inject interpreter binary 
    executor.push_state.name_bindings.insert("BIN".to_string(), Item::id(args[0].clone())); 
    println!("ok");

    print!("Initializing Message Broker ... ");
    let (tx, rx) = mpsc::channel();
    let context = zmq::Context::new();
    let mut state = State::Waiting { waiting_time: 0 };
    let mut m = Message {
        data: vec![0; DEF_PL_SIZE + PAYLOAD_OFFSET],
    };
    // Initialize publisher
    let publisher = context.socket(zmq::PUB).unwrap();
    assert!(publisher.connect("tcp://localhost:6000").is_ok());
    let subscriber = context.socket(zmq::SUB).unwrap();
    // Initialize subsciber
    thread::spawn(move || {
        assert!(subscriber.connect("tcp://localhost:5555").is_ok());
        subscriber
            .set_subscribe(
                &format!(
                    "T{:03}.{:03}",
                    MessageType::CONFIGURATION as u16,
                    MessageCommand::INPUT as u16
                )
                .as_bytes(),
            )
            .expect("Failed to subscribe");
        subscriber
            .set_subscribe(
                &format!(
                    "T{:03}.{:03}",
                    MessageType::DATA as u16,
                    MessageCommand::WRITE as u16
                )
                .as_bytes(),
            )
            .expect("Failed to subscribe");
        loop {
            let s = subscriber.recv_bytes(0).unwrap();
            tx.send(s).unwrap();
        }
    });
    println!("ok");

    // Execute program until end of temporal memory graph creation (BP 1)
    // TODO Solve without breakpoints
    print!("Creating memory graph ... ");
    executor.step_until("Identifier(BP1)".to_string());
    let mut instruction_set = InstructionSet::new();
    let instruction_cache = instruction_set.cache();
    println!("ok");

    loop {
        if PushInterpreter::step(&mut executor.push_state, &mut instruction_set, &instruction_cache) {
            if executor.push_state.exec_stack.size() > 0 {
                println!("EXEC = {}", executor.push_state.exec_stack.copy(0).unwrap().to_string());
            }
            println!("Empty Execution Stack");
            break;
        }
        match rx.try_recv() {
            Err(e) => (),
            Ok(received) => {
                if received[0] == 84 {
                    // Starts with T => Topic
                    continue;
                }

                m.data = received;
                println!("RECV MSG (TOPIC: {})", m.get_topic());

                // Inbound messages
                let header = IntVector::new(vec![m.get_prop(&TYPE_OFFSET) as i32, m.get_prop(&CMD_OFFSET) as i32, m.get_prop(&KEY_OFFSET) as i32]);
                let mut input: Vec<bool> = vec![];
                m.parse_to(&mut input);
                let body = BoolVector::new(input);
                let message = PushMessage::new(header, body);
                executor.push_state.input_stack.push_force(message);

                // Outbound messages from output buffer 
                if let Some(outb_msg) = executor.push_state.output_stack.pop() {
                    for (i,b) in outb_msg.body.values.iter().enumerate() {
                        m.set_payload_bit(&i);
                    }
                    m.set_headers(&outb_msg.header);
                    publisher.send(&m.get_topic(), zmq::SNDMORE).unwrap();
                    publisher.send(&m.data, 0).unwrap();
                }

                // Send name as string msg if flag is set
                if executor.push_state.send_name {
                    executor.push_state.send_name = false;
                    if let Some(str_msg) = executor.push_state.name_stack.pop() {
                        m.create_header(MessageType::DATA, MessageCommand::PRINT, MessageKey::UNDEFINED);
                        m.set_payload(&mut str_msg.into_bytes());
                        publisher.send(&m.get_topic(), zmq::SNDMORE).unwrap();
                        publisher.send(&m.data, 0).unwrap();
                    }
                }

                } 
            } // End of vector size check
        } // End of loop 
        println!("Done.")
    }
