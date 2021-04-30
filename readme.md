# traktor-obs-relay

A tiny [Traktor-API-Client](https://github.com/ErikMinekus/traktor-api-client) server for use on live streams.

P.S. Features like artwork retrieval assume the server is running on the same host as Traktor for now.

## Demo video

[Link on YouTube](https://www.youtube.com/watch?v=KKy2x2GCP5A)

## How to build

Just use the usual Rust workflow (`cargo build` or `cargo run`).

## How to set up

* Install [Traktor-API-Client](https://github.com/ErikMinekus/traktor-api-client) (*note:* the one provided with `Unbox` uses a different protocol and will not work, so make sure to use the one linked).
* Run `traktor-obs-relay.exe`
* Run OBS and add a browser source, point it to e.g. `http://127.0.0.1:8080/lower_third.html` (or use any other file name that resides in `/assets/`)
* Run Traktor and start mixing. The widget should show currently playing songs!

## Config explanation

### HTTP section

* `bind`: the IP address to bind to. For security, recommended to keep it at `"127.0.0.1"`.
* `port`: the port for HTTP server, both Traktor API and our API and widgets folder. Because Traktor-API-Client uses 8080, it's recommended to leave it as is. However if you changed the port in Traktor-API-Client, change it here as well as in the OBS browser URLs and in `assets/api.js` if using the default templates.
* `ws_port`: the port for the websocket that pushes track events to the widgets. If changing it here, change it in your widget code as well (or `assets/api-ws.js` if using the default templates).
* `webroot`: the folder with your widget content. This is what you can access by adding filenames to `http://<your bound IP>:<your port>/` such as in the example setup above.

### Mixing section

* `deck_list`: list of deck letters to acknowledge track names from, the rest will be ignored. Case-sensitive ('A' and 'a' are different).
* `deck_channel_map`: list of which deck goes to which channel. Usually in Traktor's crossfader grid it's `A=1, B=2, C=3, D=4`.

## About the bundled widgets

The widgets bundled are what I use on my own streams. While they can be used as-is, I strongly encourage you to take some time and come up with something unique to give your sessions more personality! Or even share some of your ideas through a pull-request :-)

### `bpm.html`

Shows an animated realtime BPM counter, attached to the MASTER CLOCK of Traktor. Requires the DSEG font to be installed on your computer.

### `bpm_headbang.html`

Displays a headbanging gif animation from a sprite sheet located in `headbanger.png`, adjusted to the master BPM. [Demo video](https://www.youtube.com/watch?v=-6AYDjDig24)

### `column.html`

Displays an animated "Previously Played" tracklist column. Recommended to place this on the left side of the stream (see [demo](https://www.youtube.com/watch?v=KKy2x2GCP5)).

### `lower_third.html`

Displays the currently played songs. Recommended to place this as the lower section of the image.

### `lower_third_artwork.html`

Same, but displays album art next to the track names  (see [demo](https://www.youtube.com/watch?v=KKy2x2GCP5)).

### `video-intro.html`

Place this as a fullscreen overlay over your stream. Place `video.mp4` into the assets folder (**NB:** Unlike all other resources, it's not included, so won't work right away!).

The stream video will become dark. When you start your first song in Traktor, `video.mp4` will play. Once `video.mp4` ends playing, your stream layout is visible. It's recommended to use a video with transparency to create a less abrupt transition into the set.
