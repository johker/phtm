var express = require('express');
var app = express();
const http = require('http').Server(app);
const io = require('socket.io')(http);
const port = process.env.PORT || 3000;
const zmq = require('zeromq');
const util = require('util');
const fs = require('fs');
const {StringDecoder} = require('string_decoder');

const MSG = require('../msg/js/msg.js');
const Message = require('./message.js');
const zeroPad = (num, places) => String(num).padStart(places, '0');


const msgSize = MSG.PAYLOAD_OFFSET + MSG.DEF_PL_SIZE;
var msg = new Message(msgSize);
var data_idx = 0;

function resolve(path, obj=self, separator='.') {
	    var properties = Array.isArray(path) ? path : path.split(separator)
	    return properties.reduce((prev, curr) => prev && prev[curr], obj)
}

function syncReadFile(filename) {
	const contents = fs.readFileSync(filename, 'UTF8');
	const arr = contents.split(/\r?\n/);
	return arr;
}

data = syncReadFile('../data/rec-center-hourly.csv');

function send_data() {
	topic = MSG.MessageCommand.INPUT;
	msg.create_header(MSG.MessageType.DATA, MSG.MessageCommand.WRITE, MSG.MessageKey.D_INPUT);
	console.log(data[(data_idx % data.length-1)]);
	msg.set_payload_bit(3);
	msg.set_payload_bit(5);
	msg.set_payload_bit(7);
	msg.set_payload_bit(80);
	msg.clear_payload_bit(5);
	const decoder = new StringDecoder('utf8');
	const outb  = Buffer.from(msg.buffer);
	console.log('SENT MSG (TOPIC: ' + msg.get_topic() + ')');
	// console.log('SENT ZMQ: ' + msg.toString());
	var pub_topic = Buffer.from(msg.get_topic()); 
	publisher.send([pub_topic, outb]);
	data_idx += 1;
}


var publisher = zmq.socket("pub");
publisher.connect("tcp://127.0.0.1:6000");
console.log("Publisher bound to port 6000");

var subscriber = zmq.socket('sub');
subscriber.connect('tcp://127.0.0.1:5555');
console.log("Subscriber bound to port 5555");

var sts = 'T' + zeroPad(MSG.MessageType.DATA,3) + '.' + zeroPad(MSG.MessageCommand.PRINT,3);
console.log("Subscribed to topic " + sts);
var sub_topic = Buffer.from(sts);
subscriber.subscribe(sub_topic);

subscriber.on('message', function(topic, message) {
	msg.parse(message);
	console.log('RECV MSG (TOPIC: ' + topic + ')');
	console.log('RECV ZMQ: ' + msg.toString());
	io.emit('sdr', msg.buffer);
});

// Send data update every 5 secs
setInterval(send_data, 5000);

app.use('/public', express.static(__dirname + '/public'));

app.get('/', (req, res) => {
	res.sendFile(__dirname + '/index.html');
});

io.on('connection', (socket) => {
	
	socket.on('data', param => {
		msg.create_header(MSG.MessageType.DATA, MSG.MessageCommand.WRITE, resolve(param.key, MSG));
		msg.set_payload_float(param.value);
		publisher.send([Buffer.from(msg.get_topic()), Buffer.from(msg.buffer)]);
		console.log('SENT MSG (TOPIC: ' + msg.get_topic() + ')');
	});

	socket.on('config', param => {
		//msg.create_header(MSG.MessageType.CONFIGURATION, MSG.MessageCommand.WRITE, resolve(param.key, MSG));
		msg.create_header(MSG.MessageType.DATA, MSG.MessageCommand.WRITE, resolve(param.key, MSG));
		msg.set_payload_float(param.value);
		publisher.send([Buffer.from(msg.get_topic()), Buffer.from(msg.buffer)]);
		console.log('SENT MSG (TOPIC: ' + msg.get_topic() + ')');
	});

	socket.on('cmd', cmd => {

		msg.create_header(MSG.MessageType.DATA, MSG.MessageCommand.WRITE, MSG.MessageKey.D_INPUT);

		msg.set_payload_bit(3);
		msg.set_payload_bit(5);
		msg.set_payload_bit(7);
		msg.set_payload_bit(80);
		msg.clear_payload_bit(5);

		const decoder = new StringDecoder('utf8');
		const outb  = Buffer.from(msg.buffer);
		let topic = MSG.MessageType.UNDEFINED;
		if (cmd == "data") {
			topic = MSG.MessageCommand.INPUT;
			console.log('SENT MSG (TOPIC: ' + msg.get_topic() + ')');
			// console.log('SENT ZMQ: ' + msg.toString());
			var pub_topic = Buffer.from(msg.get_topic()); 
			publisher.send([pub_topic, outb]);
		} else {
			console.log('UNDEFINED MSG');
		}
	});
});

http.listen(port, () => {
  console.log(`Socket.IO server running at http://localhost:${port}/`);
});



