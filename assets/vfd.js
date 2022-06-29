if(!'serial' in navigator) { alert("Browser doesn't support serial port function!"); }

/// Driver for the CD7220 based VFD pole customer display
class CD7220 {
	constructor() {
		this.baud = 9600;
		this.encoder = new TextEncoder();
		
		this.SEQ = {
			// Sequences from datasheet in native CD7220 mode

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
	
	/// Ask the user to select a serial port and connect to a CD7220 controller on that port
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
	
	/// Convert `str` into an array of bytes
	encode(str) {
		return new Uint8Array(str.split('').map((char) => char.charCodeAt(0)));
	}
	
	/// Send a raw string into the CD7220 port
	sendRawStr(str) {
		this.writer.ready
			.then(() => {
				return this.writer.write(this.encode(str));
			})
			.catch((e) => {
				console.error("Write error", e);
			});
	}
	
	/// Toggle showing the cursor
	setShowCursor(show) {
		this.sendRawStr(this.SEQ.CURSOR_VISIBLE(show));
	}
	
	/// Clear the screen
	clear() {
		this.sendRawStr(this.SEQ.CLEAR_ALL);
	}
	
	/// Set the top line string to `text`, scrolling if `scroll` is on, and set the display to fixed line mode
	setTopLine(line, scroll) {
		if(scroll) {
			this.sendRawStr(this.SEQ.STR_UPPER_SCROLL_SET(line));
		} else {
			this.sendRawStr(this.SEQ.STR_UPPER_SET(line));
		}
	}
	
	/// Set the bottom line string to `text` and set the display to fixed line mode
	setBottomLine(line) {
		this.sendRawStr(this.SEQ.STR_LOWER_SET(line));
	}
	
	/// Set the display to terminal mode with overwrite on overflow
	overwrite() { this.sendRawStr(this.SEQ.OVERWRITE); }
	/// Set the display to terminal mode with line scroll on overflow
	vScroll() { this.sendRawStr(this.SEQ.VSCROLL); }
	/// Set the display to terminal mode with column scroll on overflow
	hScroll() { this.sendRawStr(this.SEQ.HSCROLL); }
	
	/// Set the cursor position to column `x` (1~20), row `y` (1 or 2)
	setCursor(x, y) {
		if(x < 1 || x > 20 || y < 1 || y > 2) console.error("Invalid cursor coords", x, y);
		this.sendRawStr(this.SEQ.CURSOR_SET(x, y));
	}
}

// Honestly I don't know why I made this effect object model in such a weird way
// But now it seems to work reliably so any improvements are a TODO!
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

/// A display effect
class DispEffect {
	/// Construct a display effect
	/// * `interval` — interval (in ms) to call the render function
	/// * `frame` — render function (arguments: display, frameNumber). Return `false` if end of effect.
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

/// Effect to pause the effect chain for a set number of ms
class DispEffectPause extends DispEffect {
	constructor(delay) {
		super(delay, () => false);
	}
}

/// Effect to clear the screen and set the cursor position/display
class DispEffectResetAndCursor extends DispEffect {
	/// Sets the cursor position and display state and clears the screen.
	/// * `cursor` — contains `x` (1~20), `y` (1 or 2) and `show` (boolean)
	constructor(cursor) {
		super(0, (disp, frame) => {
			disp.clear();
			disp.setCursor(cursor.x, cursor.y);
			disp.setShowCursor(cursor.show);
		});
	}
}

/// Effect to set the cursor position/display
class DispEffectCursor extends DispEffect {
	/// Sets the cursor position and display state.
	/// * `cursor` — contains `x` (1~20), `y` (1 or 2) and `show` (boolean)
	constructor(cursor) {
		super(0, (disp, frame) => {
			disp.setCursor(cursor.x, cursor.y);
			disp.setShowCursor(cursor.show);
		});
	}
}

/// Effect to send bytes to the screen one by one at a set interval
class DispEffectTyping extends DispEffect {
	/// Sends the `text` symbol by symbol every `interval` ms.
	constructor(text, interval) {
		super(interval || 50, (disp, frame) => {
			if(frame == text.length) return false;
			
			disp.sendRawStr(text[frame]);
			return true;
		});
	}
}

/// Effect to slide in one or two lines from the right to the left
class DispEffectSlideInRight extends DispEffect {
	/// Slide in `texts.bottom` and/or `texts.top` from right to left.
	/// Every `interval` ms move one column to the left, unused columns padded with `pad` (default empty).
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

/// Effect to just replace one or two lines on the display
class DispEffectJustShow extends DispEffect {
	/// Show `texts.top` and/or `texts.bottom` on the display at once.
	constructor(texts) {
		super(0, (disp, frame) => {
			if(texts.bottom) disp.setBottomLine(texts.bottom);
			if(texts.top) disp.setTopLine(texts.top, texts.topScrolls);
			return false;
		});
	}
}

/// Flip through characters in sequence, leaving only correct ones behind
class DispEffectFlipIn extends DispEffect {
	/// Show `texts.top` and/or `texts.bottom` by flipping through ASCII symbols
	/// at `interval` rate, but in `maxFrames` iterations. 
	/// If `topScrolls` is set, scroll the top line when finished.
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

/// Wipe the screen with solid fill top to bottom.
class DispEffectWipeDown extends DispEffect {
	/// Wipe the screen with solid fill top to bottom, optionally leaving behind `topText` and `bottomText`.
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
			}
			
			return true;
		});
	}
}

/// Wipe the screen with solid fill bottom to top.
class DispEffectWipeUp extends DispEffect {
	/// Wipe the screen with solid fill bottom to top, optionally leaving behind `topText` and `bottomText`.
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
			}
			
			return true;
		});
	}
}

/// Sequencer to run display effects
class DispEffector {
	/// Run effects on the `disp` display
	constructor(disp) {
		this.disp = disp;
		this.nowEffect = undefined;
	}
	
	/// Run an effect, or enqueue it if the previous one isn't yet completed
	run(effect) {
		if(!this.nowEffect) {
			this.nowEffect = effect.run(this.disp);
		} else {
			this.nowEffect = this.nowEffect.next(effect);
			//console.log("append", effect);
		}
		return this.nowEffect;
	}
}

const DISPLAY = new CD7220();
const EFF = new DispEffector(DISPLAY);

/// Object filter for AssParse to exclude all but VFD lines
const VFDLineFilter = (ev) => {return (ev._type == AssEventType.Comment && ev.style == "VFD") ? ev : null};