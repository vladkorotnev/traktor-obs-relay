const WS_SERV = "ws://127.0.0.1:9090";
window.hasWsPush = true; // marker for api.js

var pushSocket = null;
function createPushSocket() {
    console.log("Creating push socket");
    pushSocket = new WebSocket(WS_SERV);
    pushSocket.onmessage = function (event) {
        let info = JSON.parse(event.data);
        processUpdates(info);
    };

    // redundancy
    pushSocket.onclose = function (c) {
        console.log("Socket closed", c);
        setTimeout(createPushSocket, 500);
    };
}

createPushSocket();