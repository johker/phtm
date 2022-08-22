use std::thread;

fn trace_task(context: &zmq::Context) {
    println!("Starting trace task");
    let receiver = context.socket(zmq::PAIR).unwrap();
    receiver
        .connect("inproc://trace")
        .expect("listener thread failed connecting");
    loop {
        // let msg = receiver
        //     .recv_string(0)
        //     .expect("worker failed receiving")
        //     .unwrap();
        let raw_msg = receiver.recv_bytes(0).expect("Failed receiving bytes");
        println!("Bytes: {:?}", raw_msg);
    }
}

fn main() {
    println!("0mq broker started!");
    let context = zmq::Context::new();
    let mut sub_socket = context.socket(zmq::SUB).unwrap();
    let mut pub_socket = context.socket(zmq::PUB).unwrap();
    let mut capture = context.socket(zmq::PAIR).unwrap();

    sub_socket
        .bind("tcp://*:6000")
        .expect("failed binding sub socket");
    pub_socket
        .bind("tcp://*:5555")
        .expect("failed binding pub socket");
    capture
        .bind("inproc://trace")
        .expect("failed binding pair socket");

    sub_socket.set_subscribe(b"").expect("failed to subscribe");

    let ctx = context.clone();

    thread::spawn(move || trace_task(&ctx));
    zmq::proxy_with_capture(&mut sub_socket, &mut pub_socket, &mut capture)
        .expect("failed proxying");
    //zmq::proxy(&frontend, &backend).unwrap();
    println!("0mq broker terminated");
}
