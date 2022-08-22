const MSG = require('../msg/js/msg.js');
const zeroPad = (num, places) => String(num).padStart(places, '0');

class Message {

	constructor(msgSize) {
		this.buffer = new ArrayBuffer(msgSize);
		this.arrayView = new Uint8Array(this.buffer);
		this.dataView = new DataView(this.buffer);
	}
	
	create_header(type,cmd,key) {
		this.dataView.setUint16(MSG.ID_OFFSET, this.get_uid());
		this.dataView.setUint16(MSG.TYPE_OFFSET, type);
		this.dataView.setUint16(MSG.CMD_OFFSET,cmd);
		this.dataView.setUint16(MSG.KEY_OFFSET, key);
	}
	
	parse(data) {
		this.arrayView = new Uint8Array(data);
		this.buffer = this.arrayView.buffer;
		this.dataView = new DataView(this.buffer);
	}
	
	toString() {
		var bufStr = Array.apply([], this.arrayView).join(",");
		var update = `${bufStr}`;
		return update;
	}

	get_cmd() {
		return this.dataView.getUint16(MSG.CMD_OFFSET);
	}

	get_key() {
		return this.dataView.getUint16(MSG.KEY_OFFSET);
	}

	get_type() {
		return this.dataView.getUint16(MSG.TYPE_OFFSET);
	}

	get_topic() {
		return 'T' + zeroPad(this.get_type(),3) + '.' + zeroPad(this.get_cmd(),3);
	}


	set_payload_bit(idx) {
		let byteIdx = (idx >> 3) + MSG.PAYLOAD_OFFSET;
		let bitIdx = idx % 8;
		this.arrayView[byteIdx] = this.arrayView[byteIdx] | 1 << bitIdx;
	}
	
	set_payload_float(value) {
		this.dataView.setFloat32(MSG.PAYLOAD_OFFSET, value)
	}

	clear_payload_bit(idx) {
		let byteIdx = (idx >> 3) + MSG.PAYLOAD_OFFSET;
		let bitIdx = idx % 8;
		this.arrayView[byteIdx] = this.arrayView[byteIdx] & ~(1 << bitIdx);
	}

	is_active(idx) {
		let byteIdx = (idx >> 3) + MSG.PAYLOAD_OFFSET;
		let bitIdx = idx % 8;
		return (this.arrayView[byteIdx] >> bitIdx) % 2 != 0;
	}
	
	get_uid() {
		// TODO Generate UID
		return 1;
	}
}

module.exports = Message;
