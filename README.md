### Zoovebot

#### Description

Zoovebot is a (tiny) twitter bot that converts a link to a track from one music streaming platform to others. When the bot is mentioned on twitter, it detects and fetches the link to the track on `Deezer`, `Spotify` and `Tidal`.

#### Roadmap

Currently, the bot isn't fancy as it does not properly handle errors, etc. As time goes on, its expected that the bot might become more complex and some of the ideas currently being explored are:

- [ ] Supporting converting playlists
- [ ] Multiple tracks conversion
- [ ] Linking to a (simple) webview for playlist and multiple track conversions
- [ ] Twitter preview (automatically generating media from preview url in order to preview tracks without leaving twitter)

The above may change anytime due to any reason but contributions are welcome. **One of the limitations is that the Orchdio API is currently in closed beta and not publicly available yet. Please reach out privately for API key for dev.**

#### TODO

Currently things that are left todo and are short-term needs are:

- [ ] Tests (probably not necessary at the moment because its pretty small)
- [ ] Error handling
- [ ] Multiple links detection and handling
