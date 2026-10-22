# Historical Web measurement sources

[#111](https://github.com/edwardkim/ohmylemmings/issues/111) measured the app before #113. These three byte-identical files retain the original paths and SHA-256 in `assets/manifests/web_deadlines.json`. They are source snapshots, not current app entry points.

For replay, copy `issue111-app.js` to `app.js`, `issue111-live.js` to `live.js` and `issue111-cdp.cjs` to the measurement CLI path, together with the matching pinned host/audio/controls/measure/WASM/input files. The earlier captured CLI producer and complete raw pins remain in the ignored local evidence recorded by #111.

#113's paired runs use the same #111 live observer and final CLI for both apps; their exact source and staging pins are recorded separately. Current opt-in diagnostic tools do not replace those producers or erase any observed miss.
