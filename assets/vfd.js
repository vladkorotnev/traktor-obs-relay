if(!'serial' in navigator) { alert("Browser doesn't support serial port function!"); }

class CD7220 {
	constructor() {
		this.baud = 9600;
		this.encoder = new TextEncoder();
		
		this.SEQ = {
			OVERWRITE: "\x1B\x11",
			VSCROLL: "\x1B\x12",
			HSCROLL: "\x1B\x13",
			
			STRING_UPPER: "\x1B\x51\x41",
			STRING_LOWER: "\x1B\x51\x42",
			STRING_UPPER_SCRL: "\x1B\x51\x44",
			STRING_END: "\x0D",
			
			CURS_LEFT: "\x08",
			CURS_RIGHT: "\x09",
			CURS_UP: "\x1B\x5B\x41",
			CURS_DOWN: "\x0A",
			CURS_HOME: "\x0B",
			CURS_LEFTMOST: "\x0D",
			CURS_RIGHTMOST: "\x1B\x5B\x52",
			CURS_BOTTOM: "\x1B\x5B\x4B",
			CURS_SET_PRELUDE: "\x1B\x6C",
			
			CLEAR_ALL: "\x0C",
			CLEAR_LINE: "\x18",
			
			
			CURSOR_SET: function(x, y) { return this.CURS_SET_PRELUDE+String.fromCharCode(x)+String.fromCharCode(y) },
			STR_UPPER_SET: function(str) { return this.STRING_UPPER+str+this.STRING_END },
			STR_LOWER_SET: function(str) { return this.STRING_LOWER+str+this.STRING_END },
			STR_UPPER_SCROLL_SET: function(str) { return this.STRING_UPPER_SCRL+str+this.STRING_END },
			CURSOR_VISIBLE: function(show) { return "\x1B\x5F" + (show ? "\x01" : "\x00"); }
		};
	}
	
	async init() {
		try {
			let port = await navigator.serial.requestPort();
			await port.open({baudRate: this.baud});
			this.writer = port.writable.getWriter();
			this.clear();
			this.setShowCursor(false);
		} catch (E) {
			console.error("Port open failed: ", E);
			alert('Port open failed!');
		}
	}
	
	encode(str) {
		return new Uint8Array(str.split('').map((char) => char.charCodeAt(0)));
	}
	
	sendRawStr(str) {
		this.writer.ready
			.then(() => {
				return this.writer.write(this.encode(str));
			})
			.catch((e) => {
				console.error("Write error", e);
			});
	}
	
	setShowCursor(show) {
		this.sendRawStr(this.SEQ.CURSOR_VISIBLE(show));
	}
	
	clear() {
		this.sendRawStr(this.SEQ.CLEAR_ALL);
	}
	
	setTopLine(line, scroll) {
		if(scroll) {
			this.sendRawStr(this.SEQ.STR_UPPER_SCROLL_SET(line));
		} else {
			this.sendRawStr(this.SEQ.STR_UPPER_SET(line));
		}
	}
	
	setBottomLine(line) {
		this.sendRawStr(this.SEQ.STR_LOWER_SET(line));
	}
	
	overwrite() { this.sendRawStr(this.SEQ.OVERWRITE); }
	vScroll() { this.sendRawStr(this.SEQ.VSCROLL); }
	hScroll() { this.sendRawStr(this.SEQ.HSCROLL); }
	
	setCursor(x, y) {
		if(x < 1 || x > 20 || y < 1 || y > 2) console.error("Invalid cursor coords", x, y);
		this.sendRawStr(this.SEQ.CURSOR_SET(x, y));
	}
}

class EffectPromise extends Promise {
	constructor(fn, on) {
		super(fn);
		this.disp = on;
	}
	
	next(effect) {
		return new EffectPromise((resolve, reject) => {
			this.then(() => {
				effect.run(this.disp).then(resolve);
			});
		}, this.disp);
	}
}

class DispEffect {
	constructor(interval, frame) {
		this.interval = interval;
		this.frame = frame;
	}
	
	run(on) {
		return new EffectPromise((success, error) => {
			this.counter = 0;
			this._curTimer = setInterval(() => {
				if(!this.frame(on, this.counter)) {
					clearInterval(this._curTimer);
					success();
				} else {
					this.counter++;
				}
			}, this.interval);
		}, on);
	}
}

class DispEffectPause extends DispEffect {
	constructor(delay) {
		super(delay, () => false);
	}
}

class DispEffectResetAndCursor extends DispEffect {
	constructor(cursor) {
		super(0, (disp, frame) => {
			disp.clear();
			disp.setCursor(cursor.x, cursor.y);
			disp.setShowCursor(cursor.show);
		});
	}
}

class DispEffectTyping extends DispEffect {
	constructor(text, interval) {
		super(interval || 50, (disp, frame) => {
			if(frame == text.length) return false;
			
			disp.sendRawStr(text[frame]);
			return true;
		});
	}
}

class DispEffectSlideInRight extends DispEffect {
	constructor(texts, interval, pad) {
		pad = pad || ' ';
		let width = 20;
		
		super(interval || 125, (disp, frame) => {
			if(frame == width)  {
				if(texts.bottom) {
					disp.setBottomLine(texts.bottom);
				}
				if(texts.top) {
					disp.setTopLine(texts.top, texts.topScrolls);
				}
				return false;	
			}
			
			if(texts.top) {
				disp.setTopLine(pad.repeat(width - frame) + texts.top);
			}
			if(texts.bottom) {
				disp.setBottomLine(pad.repeat(width - frame) + texts.bottom);
			}
			
			return true;
		});
	}
}

class DispEffectFlipIn extends DispEffect {
	constructor(texts, interval, maxFrames, topScrolls) {
		let firstLetterCode = "A".charCodeAt(0);
		let lastLetterCode = "z".charCodeAt(0);
		maxFrames = maxFrames || (lastLetterCode - firstLetterCode);
		let step = Math.ceil( (lastLetterCode - firstLetterCode) / maxFrames);
		if(step < 1) step = 1;
		
		super(interval || 50, (disp, frame) => {
			if(frame == 0) {
				this.stopTop = (!texts.top);
				this.stopBottom = (!texts.bottom);
			}
			
			if(frame == maxFrames || (this.stopTop && this.stopBottom)) {
				if(texts.bottom) disp.setBottomLine(texts.bottom);
				if(texts.top) disp.setTopLine(texts.top, topScrolls);
				return false;
			}
			
			let transform = (text, frame) => {
				let nowCharCode = (firstLetterCode + frame*step);
				return text.padEnd(20).split('').map((x) => (x.charCodeAt(0) > nowCharCode || x == ' ') ? String.fromCharCode(nowCharCode) : x).join('');
			};
			
			if(texts.top && !this.stopTop) {
				let tfm = transform(texts.top, frame);
				disp.setTopLine(tfm);
				if(tfm == texts.top) this.stopTop = true;
			}
			if(texts.bottom && !this.stopBottom) {
				let tfm = transform(texts.bottom, frame);
				disp.setBottomLine(tfm);
				if(tfm == texts.bottom) this.stopBottom = true;
			}
			
			return true;
		});
	}
}

class DispEffectWipeDown extends DispEffect {
	constructor(interval, topText, bottomText) {
		let width = 20;
		super(interval || 50, (disp, frame) => {
			switch(frame) {
				case 0:
					disp.setTopLine("\xDF".repeat(width));
					break;
					
				case 1:
					disp.setTopLine("\xDB".repeat(width));
					break;
					
				case 2:
					disp.setBottomLine("\xDF".repeat(width));
					break;
					
				case 3:
					disp.setBottomLine("\xDB".repeat(width));
					break;
					
				case 4:
					disp.setTopLine("\xDC".repeat(width));
					break;
					
				case 5:
					disp.setTopLine(topText || " ".repeat(width));
					break;
					
				case 6:
					disp.setBottomLine("\xDC".repeat(width));
					break;
					
				case 7:
					disp.setBottomLine(bottomText || " ".repeat(width));
					return false;
					break;
			}
			
			return true;
		});
	}
}

class DispEffectWipeUp extends DispEffect {
	constructor(interval, topText, bottomText) {
		let width = 20;
		super(interval || 50, (disp, frame) => {
			switch(frame) {
				case 0:
					disp.setBottomLine("\xDC".repeat(width));
					break;
					
				case 1:
					disp.setBottomLine("\xDB".repeat(width));
					break;
					
				case 2:
					disp.setTopLine("\xDC".repeat(width));
					break;
					
				case 3:
					disp.setTopLine("\xDB".repeat(width));
					break;
					
				case 4:
					disp.setBottomLine("\xDF".repeat(width));
					break;
					
				case 5:
					disp.setBottomLine(bottomText || " ".repeat(width));
					break;
					
				case 6:
					disp.setTopLine("\xDF".repeat(width));
					break;
					
				case 7:
					disp.setTopLine(topText || " ".repeat(width));
					return false;
					break;
			}
			
			return true;
		});
	}
}


class DispEffector {
	constructor(disp) {
		this.disp = disp;
		this.nowEffect = undefined;
	}
	
	run(effect) {
		if(!this.nowEffect) {
			this.nowEffect = effect.run(this.disp);
		} else {
			this.nowEffect = this.nowEffect.next(effect);
		}
		return this.nowEffect;
	}
}

const DISPLAY = new CD7220();
const EFF = new DispEffector(DISPLAY);
