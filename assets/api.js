const API_ROOT = "http://127.0.0.1:8080/";
const NOW_PLAYING_API = "nowPlaying";

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
        return API_ROOT+"artwork/"+meta.deck;
    }
    return "";
}