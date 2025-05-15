// Reference: https://github.com/vladkorotnev/esp-nje/

if(!'serial' in navigator) { alert("Browser doesn't support serial port function!"); }

const JISTable = { '\u00a5': 0x5c, '\u203e': 0x7e, '\u301c': 0x8160 };
const JISDecoder = new TextDecoder('shift-jis');
for (let i = 0x81; i <= 0xfc; i++) {
    if (i <= 0x84 || i >= 0x87 && i <= 0x9f || i >= 0xe0 && i <= 0xea || i >= 0xed && i <= 0xee || i >= 0xfa) {
        for (let j = 0x40; j <= 0xfc; j++) {
            const c = JISDecoder.decode(new Uint8Array([i, j]));
            if (c.length === 1 && c !== '\ufffd' && !JISTable[c]) {
                JISTable[c] = i << 8 | j;
            }
        }
    }
}

const NJEAttrs = {
	MsgKind: {
		Normal: {add: "A110", delete: "A210"},
		Emergency: {add: "B111", delete: "B211"},
		Commercial: {add: "F120", delete: "F220"}
	},
	Color: {
		Green: "A",
		Red: "B",
		Yellow: "C"
	},
	Decor: {
		Scroll: "A",
		ScrollBlink: "B",
		ScrollInverse: "C",
		ScrollBlinkInverse: "D",
		Pull: "E",
		PullBlink: "F",
		PullInverse: "G",
		PullBlinkInverse: "H",
		Pause: "I",
		PauseBlink: "J",
		PauseInverse: "K",
		PauseBlinkInverse: "L",
		Still: "W",
		StillBlink: "X",
		StillInverse: "Y",
		StillBlinkInverse: "Z"
	}
};

class NJEFormattedSpan {
	constructor(content, color = NJEAttrs.Color.Green, decor = NJEAttrs.Decor.Scroll) {
		this.content = content;
		this.color = color;
		this.decor = decor;
	}

	toString() {
		return "~" + this.color + this.decor + "~" + this.content;
	}
}

class NJEFormattedString {
	constructor(spans) {
		this.spans = spans;
	}

	toString() {
		return this.spans[0].content + this.spans.slice(1).map(x => x.toString()).join("");
	}
}

/// Driver for the NJE-105/NJE-106 display
class NJE {
	constructor() {
		this.baud = 9600;
		this.ready = false;
	}
	
	/// Ask the user to select a serial port and connect to an NJE on that port
	async init() {
		try {
			let port = await navigator.serial.requestPort();
			await port.open({baudRate: this.baud});
			this.writer = port.writable.getWriter();
			this.ready = true;
		} catch (E) {
			console.error("Port open failed: ", E);
			alert('Port open failed!');
		}
	}
	
	/// Convert `str` into an array of bytes in Shift-JIS
	encode(str) {
		let buffer = [];
		for (let i = 0; i < str.length; i++) {
			const c = str.codePointAt(i);
			if (c > 0xffff) {
				i++;
			}
			if (c < 0x80) {
				buffer.push(c);
			}
			else if (c >= 0xff61 && c <= 0xff9f) {
				buffer.push(c - 0xfec0);
			}
			else {
				const d = JISTable[String.fromCodePoint(c)] || 0x3f;
				if (d > 0xff) {
					buffer.push(d >> 8 & 0xff, d & 0xff);
				}
				else {
					buffer.push(d);
				}
			}
		}
		return Uint8Array.from(buffer);
	}

	sendRawBytes(bytes) {
		if(!this.ready) return;

		this.writer.ready
			.then(() => {
				return this.writer.write(bytes);
			})
			.catch((e) => {
				console.error("Write error", e);
			});
	}

	/// Send a UTF8 string into the NJE port
	sendUtfStr(str) {
		console.log("OUT: ", str);
		this.sendRawBytes(this.encode(str));
	}
	
	mkDateTime() {
		let now = new Date();
		let dateStr = (now.getMonth()+1).toString().padStart(2, '0') + now.getDate().toString().padStart(2, '0') + now.getHours().toString().padStart(2, '0') + now.getMinutes().toString().padStart(2, '0');
		return dateStr;
	}

	/// Create a packet with the UTF8 string and send it
	sendPkt(str) {
		if(str.length > 128) console.warn("Packet too long!!", str);

		let dateStr = this.mkDateTime();
		let pktStr = "\r\n" + dateStr + str + "\r\n";
		this.sendUtfStr(pktStr);
	}

	/// Reset the display
	reset() {
		this.sendPkt("NJER")
	}

	setEnergySaver(saver) {
		this.sendPkt("NJES02" + (saver ? "02" : "00"));
	}

	setMessage(kind, number, content, color, decor) {
		if(content.constructor == NJEFormattedString) {
			if(content.spans.length == 0) return;

			color = color || content.spans[0].color;
			decor = decor || content.spans[0].decor;
		}

		this.sendPkt(
			"]011" + kind.add + number.toString().padStart(2, "0") + this.mkDateTime() + color + decor + content.toString()
		);
	}

	deleteMessage(kind, number) {
		this.sendPkt(
			"]011" + kind.delete + number.toString().padStart(2, "0")
		);
	}

	// Speed: scroll speed 0-2
	setScrollSpeed(speed) {
		speed = Math.min(speed, 2);
		speed = Math.max(speed, 0);
		this.sendPkt("NJES06" + speed.toString().padStart(2, "0"))
	}

	// Speed: blink speed 0-3
	setBlinkSpeed(speed) {
		speed = Math.min(speed, 3);
		speed = Math.max(speed, 0);
		this.sendPkt("NJES07" + speed.toString().padStart(2, "0"));
	}

	// Speed: pause speed 0-10
	setPauseSpeed(speed) {
		speed = Math.min(speed, 10);
		speed = Math.max(speed, 0);
		this.sendPkt("NJES08" + speed.toString().padStart(2, "0"));
	}
}

const NJEPort = new NJE();
