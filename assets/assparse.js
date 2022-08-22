/// Java-style split with maximum number of separators specification
String.prototype.rsplit = function (separator, n) {
    let sep = new RegExp(separator,"g");
    var out = [];

    while(n--) out.push(this.slice(sep.lastIndex, sep.exec(this).index));

    out.push(this.slice(sep.lastIndex));
    return out;
};

/// Subtitle event type
const AssEventType = {
    /// Comment event
    Comment: "Comment",
    /// Dialogue event
    Dialogue: "Dialogue"
};

/// An ASS subtitle file parser
class AssParse {
    /// `filter` can be used to alter the events prior to them being added into `events`, or dropping them if `null` is returned.
    constructor(text, filter) {
        this.text = text;
        this.events = [];
        this.filter = filter || ((x) => x);

        this.fieldFilters = {
            start: this.convertTime,
            end: this.convertTime
        };

        this.Consts = {
            EventHeader: "[Events]",

            FormatPrefix: "Format: ",
            CommentPrefix: "Comment: ",
            DialoguePrefix: "Dialogue: "
        };

        this.parse();
    }

    /// Convert an ASS time string (e.g. 0:12:34.56) into seconds value
    convertTime(timeStr) {
        let nrmlTimeStr = (timeStr[1] == ":") ? "0"+timeStr : timeStr;
        return (new Date('1970-01-01T' + nrmlTimeStr + 'Z').getTime() / 1000.0);
    }

    /// Parse the current `this.text` buffer and fill the parsed events into `this.events` array.
    parse() {
        this.events = [];

        let lines = this.text.split("\n");
        
        // Current CSV format specifier
        var formatSpec = null;

        /// Parse a single CSV line into an object based on the `fmtSpec` specifier.
        /// If `fmtSpec` is null, treat the line as a format specifier and return a parsed array.
        let parseLine = (inLine, prefix, fmtSpec) => {
            let line = inLine.substring(prefix.length);
            if(!fmtSpec) return line.split(',').map((x) => x.trim().toLowerCase());

            var rslt = {
                _type: prefix.rsplit(':',1)[0].trim()
            };

            let splitLine = line.rsplit(',', fmtSpec.length - 1);
            for(var i = 0; i < fmtSpec.length; i++) {
                let fldName = fmtSpec[i];
                var fldData = splitLine[i];
                if(this.fieldFilters[fldName]) {
                    fldData = this.fieldFilters[fldName](fldData);
                }
                rslt[fldName] = fldData;
            }

            return rslt;
        };

        var isInEventSection = false;
        for(let line of lines) {
            if(line.trim()[0] == ';') continue; // skip comment lines
            if(line[0] == '[') {
                // read only event lines
                if(!isInEventSection && line.trim() == this.Consts.EventHeader) {
                    isInEventSection = true;
                    continue;
                } else if (isInEventSection) {
                    isInEventSection = false;
                    break;
                }
            }

            // If line is in the `[Events]` section of the file
            if(isInEventSection) {
                var rslt = null;

                if(line.startsWith(this.Consts.FormatPrefix)) { // Format: lines will set the format specifier
                    formatSpec = parseLine(line, this.Consts.FormatPrefix, null);
                } else if (line.startsWith(this.Consts.CommentPrefix)) {
                    rslt = parseLine(line, this.Consts.CommentPrefix, formatSpec)
                } else if (line.startsWith(this.Consts.DialoguePrefix)) {
                    rslt = parseLine(line, this.Consts.DialoguePrefix, formatSpec)
                }

                // If parsing succeeded
                if(rslt) {
                    let flt = this.filter(rslt);
                    // If filter function didn't drop the object
                    if(flt) {
                        this.events.push(flt);
                    }
                }
            }
        }
    }
}

/// An ASS subtitle scheduler / timing loop
class AssLooper {
    /// Construct an `AssLooper`
    /// * `events` — array of timed subtitle events, such as from `AssParse.events`
    /// * `executor` — function to be called with a subtitle event every time it's time to output one
    /// * `tempo` — rate at which to play the subtitle events (1.0 is realtime, also default)
    /// * `fps` — rate at which to execute the run loop, default 15. Setting this too low might miss events, setting too high might be overkill.
    /// * `completion` — optional callback that will be called once all events have been emitted
    constructor(events, executor, tempo, fps, completion) {
        this.events = events;
        this.callback = executor;

        this.fps = fps || 15;
        this.tempo = tempo || 1.0;

        this.fpsRecip = 1/this.fps;
        this.frameTime = 1000*this.fpsRecip;

        this.nextTickTimer = undefined;
        this.lastTick = new Date();

        this.elapsed = 0;
        this.running = false;

        this.completion = completion;

        // Turns out when disabling background throttling in Chrome, it affects timers (setTimeout),
        // but not animation frames. Since my use-case for this is in a backgrounded Chrome tab,
        // it makes sense to use setTimeout instead. However it's possible to switch to requestAnimationFrame
        // if necessary by changing this flag.
        this.useAnimationFrame = false;
    }

    reset() {
        this.running = false;
        this.elapsed = 0;
        this.stopLooping();
        this.events
            .forEach(x => x.passed = false);
    }

    /// Begin emitting subtitle events
    /// * `nowTime` — timestamp (seconds) at which to start/resume emitting events (default: current timestamp)
    startLooping(nowTime) {
        this.elapsed = nowTime || this.elapsed;

        this.running = true;
        this.lastTick = new Date();

        this._tick();
    }

    /// Stop emitting subtitle events
    stopLooping() {
        this.running = false;
        if(this.nextTickTimer)
            this._rmTimer(this.nextTickTimer);
    }

    _mkTimer(cb) {
        if(this.useAnimationFrame) {
            return requestAnimationFrame(cb);
        } else {
            return setTimeout(cb);
        }
    }

    _rmTimer(handle) {
        if(this.useAnimationFrame) {
            cancelAnimationFrame(handle);
        } else {
            clearTimeout(handle);
        }
    }

    _tick() {
        if(!this.running) return;

        this.nextTickTimer = this._mkTimer(() => this._tick());

        var now = Date.now();
        var frameElapsed = now - this.lastTick;

        if(frameElapsed > this.frameTime) {
            this.lastTick = now;

            // Advance the elapsed time in seconds by (real elapsed time × playback rate)
            this.elapsed += ((frameElapsed/1000.0) * this.tempo);

            // Find the events which have not yet been emitted and will occur within the next 
            // (framerate period × playback rate) seconds
            let nextEvents = this.events.filter(x=> {
                let untilStart = Math.abs(x.start - this.elapsed);
                return (untilStart < (this.frameTime*this.tempo)/1000) && (!x.passed);
            });
            // Emit all of the events
            nextEvents.forEach(x => {
                x.passed = true;
                this.callback(x);
            });

            if(this.completion) {
                if(this.events.filter(x => !x.passed).length == 0) {
                    this.completion();
                }
            }
        }
    }

    /// Update the looper with the current playback status
    /// * `newTime` — current time elapsed, in decimal seconds
    /// * `newTempo` — current playback rate, decimal, 1.0 is real-time
    reportTimeAndTempo(newTime, newTempo) {
        if( (Math.abs(newTime - this.elapsed) > (this.fps/4)*this.fpsRecip) || this.tempo != newTempo) {
            // If lagging ahead or behind for around 0.25s worth of frames, force sync the loop
            this._rmTimer(this.nextTickTimer);
            console.warn("Looper went astray: local time ", this.elapsed, " actual time ", newTime, "delta", this.elapsed - newTime);
            if(newTime < this.elapsed) {
                this.events
                    .filter(x => x.start >= newTime && x.passed)
                    .forEach(x => x.passed = false);
            }
            this.elapsed = newTime;
            this.lastTick = Date.now() + this.frameTime;
            setTimeout(() => { if(this.running) this.nextTickTimer = this._mkTimer(() => this._tick()); }, this.frameTime/2.0);
        }
        this.tempo = newTempo;
    }
}
