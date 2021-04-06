const API_ROOT = "http://127.0.0.1:8080/";
const NOW_PLAYING_API = "nowPlaying";
const ART_API = "artwork/";
const CHECK_INTERVAL = 2000;

function query() {
    return new Promise((resolve, reject) => {
        let xhr = new XMLHttpRequest();
        xhr.open("GET", API_ROOT+NOW_PLAYING_API);
        xhr.onload = () => {
            if (xhr.status >= 200 && xhr.status < 300) {
                resolve(JSON.parse(xhr.response));
            } else {
                reject(xhr.statusText);
            }
        };
        xhr.onerror = () => reject(xhr.statusText);
        xhr.send();
    });
}

function getArtUrl(meta) {
    if(meta.deck) {
        return API_ROOT+ART_API+meta.deck+"?nocache="+Date.now();
    }
    return "";
}

var tracks = {};
var oldBpm = 0;
function processUpdates(info) {
    if(!info) return;
    if(info.songsOnAir) {
        let newTracks = Object.fromEntries( info.songsOnAir.map(x => [x.filePath, x]) );

        let newPaths = Object.keys(newTracks);
        let oldPaths = Object.keys(tracks);

        let goneTracks = oldPaths.filter(path => newPaths.indexOf(path) == -1).map(x => tracks[x]);
        let addedTracks = newPaths.filter(path => oldPaths.indexOf(path) == -1).map(x => newTracks[x]);

        goneTracks.forEach(element => popTrack(element));
        addedTracks.forEach(element => pushTrack(element));

        tracks = newTracks;
    }
    if(info.bpm && info.bpm != oldBpm) {
        onBpmChanged(info.bpm);
        oldBpm = info.bpm;
    }
}

function watchLoop() {
    query()
        .then((info) => {
            processUpdates(info);
        })
        .finally(() => {
            if(!window.hasWsPush) {
                setTimeout(watchLoop, CHECK_INTERVAL);
            }
            else {
                console.log("Websocket enabled, not rescheduling timer");
            }
        })
}

window.onload = watchLoop;