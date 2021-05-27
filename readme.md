# traktor-obs-relay

A tiny [Traktor-API-Client](https://github.com/ErikMinekus/traktor-api-client) server for use on live streams.

P.S. Features like artwork or subtitle retrieval assume the server is running on the same host as Traktor for now.

## Demo video

Check this small [video on YouTube](https://www.youtube.com/watch?v=KKy2x2GCP5A), or have a look at my debut set for Anison Hijack [also on YouTube](https://youtu.be/hcdWe641rPE) which makes use of all available widgets at the time.

## Core architecture

The executable itself only acts as a state store, tiny web server and an event bus in one thing.

It also "demuxes" track and deck events, combining them together to effectively figure out what tracks are hearable to the listeners. Thus, a playing deck won't affect the widgets if it's closed behind the Xfader or the channel fader. 

Clients can then poll the host executable on `/nowPlaying` or get the same update data on most major events via websocket. 

There is a simple JavaScript client layer implemented under `assets/api` which will, when loaded into a page, execute the following functions in the page context:

* `pushTrack(meta)`: when a new track is introduced into the mix
* `popTrack(meta)`: when a track is removed from the mix
* `onBpmChanged(bpm)`: when the master clock BPM is changed
* `trackTick(meta)`: when a track receives a minor update (elapsed time or BPM change)

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
* `more_events`: enables sending of minor events, such as elapsed time ticks, to the websocket.

### Mixing section

* `deck_list`: list of deck letters to acknowledge track names from, the rest will be ignored. Case-sensitive ('A' and 'a' are different).
* `deck_channel_map`: list of which deck goes to which channel. Usually in Traktor's crossfader grid it's `A=1, B=2, C=3, D=4`.
* `default_cover`: path to the default cover art when reading one from the deck info is not possible.

## Exposed endpoints

Aside from the usual endpoints from Traktor-API-Client, the HTTP host also provides the following URLs:

* `/nowPlaying`: get the current on-air state of everything that can be heard by the listeners (on-air tracks, master clock BPM and etc.)
* `/artwork/<deck letter>`: get the artwork for the track playing in the specified deck. Currently only reading artwork from FLAC and MP3 files is supported.
* `/subtitles/<deck letter>`: get the subtitle file for the track playing in the specified deck. It should reside in the same folder as the track, with the same name and the extension `ass` for Advanced Substation format. E.g. if you are playing a track from `D:\Music\The Beatles\Help.mp3`, the subtitles should be located in `D:\Music\The Beatles\Help.ass`.
* `/video/<deck letter>`: get the video file for the track playing in the specified deck. It should reside in the same folder as the track, with the same name and the extension `mp4` or `webm`. E.g. if you are playing a track from `D:\Music\The Beatles\Help.mp3`, the video should be located in `D:\Music\The Beatles\Help.webm`.

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

### `subtitle.html`

Based upon the [JavascriptSubtitlesOctopus](https://github.com/libass/JavascriptSubtitlesOctopus) library.

Place this as a fullscreen overlay over your stream. It will play subtitle files to your songs (Advanced Substation Alpha, a.k.a. ASS format) on the first played, first shown basis. This is probably resource-heavy so don't go too hard on adding that vintage fansub karaoke flair with half a dozen animations per symbol!

### `auto-vj.html`

Plays video files stored by the same name as the played audio files automatically. Not guaranteed to be 100% in sync for obvious reasons. Supports `webm` and `mp4` with opacity.
