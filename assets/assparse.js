String.prototype.rsplit = function (separator, n) {
    let sep = new RegExp(separator,"g");
    var out = [];

    while(n--) out.push(this.slice(sep.lastIndex, sep.exec(this).index));

    out.push(this.slice(sep.lastIndex));
    return out;
};

const AssEventType = {
    Comment: "Comment",
    Dialogue: "Dialogue"
};

class AssParse {
    constructor(text, filter) {
        this.text = text;
        this.events = [];

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

        this.parse(filter);
    }

    convertTime(timeStr) {
        let nrmlTimeStr = (timeStr[1] == ":") ? "0"+timeStr : timeStr;
        return (new Date('1970-01-01T' + nrmlTimeStr + 'Z').getTime() / 1000.0);
    }

    parse(filter) {
        this.events = [];

        let lines = this.text.split("\n");
        let fltFn = filter || ((x) => x);
        
        var formatSpec = null;

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

            if(isInEventSection) {
                var rslt = null;

                if(line.startsWith(this.Consts.FormatPrefix)) {
                    formatSpec = parseLine(line, this.Consts.FormatPrefix, null);
                } else if (line.startsWith(this.Consts.CommentPrefix)) {
                    rslt = parseLine(line, this.Consts.CommentPrefix, formatSpec)
                } else if (line.startsWith(this.Consts.DialoguePrefix)) {
                    rslt = parseLine(line, this.Consts.DialoguePrefix, formatSpec)
                }

                if(rslt) {
                    let flt = fltFn(rslt);
                    if(flt) {
                        this.events.push(flt);
                    }
                }
            }
        }
    }
}

class AssLooper {
    constructor(events, executor, tempo, fps) {
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

        this.useAnimationFrame = false;
    }

    startLooping(nowTime) {
        this.elapsed = nowTime || this.elapsed;

        this.running = true;
        this.lastTick = new Date();

        this._tick();
    }

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
            this.elapsed += ((frameElapsed/1000.0) * this.tempo);

            let nextEvents = this.events.filter(x=> {
                let untilStart = Math.abs(x.start - this.elapsed);
                return (untilStart < (this.frameTime*this.tempo)/1000) && (!x.passed);
            });
            nextEvents.forEach(x => {
                x.passed = true;
                this.callback(x);
            });
        }
    }

    reportTimeAndTempo(newTime, newTempo) {
        if( (Math.abs(newTime - this.elapsed) > (this.fps/4)*this.fpsRecip) || this.tempo != newTempo) {
            // if astray for more than 0.25s of frames, force set time
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

let VFDLineFilter = (ev) => {return (ev._type == AssEventType.Comment && ev.style == "VFD") ? ev : null};