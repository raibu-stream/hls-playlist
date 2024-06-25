# HLS Playlist

A library for serializing and deserializing HLS playlists (aka extended M3U playlists).

As specified by [this updated version of RFC 8216](https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis).

## Usage

```rust
use hls_playlist::tags::Tag;

let mut output = vec![];

Tag::M3u.serialize(&mut output).unwrap();
Tag::XStart { offset_seconds: 10.0, is_precise: false }.serialize(&mut output).unwrap();

assert_eq!(String::from_utf8(output).unwrap(), "\
#EXTM3U
#EXT-X-START:TIME-OFFSET=10
");
```

## Features

* `steering-manifest`: Enables support for serializing and deserializing steering manifests.

## Roadmap

This library is 100% finished and feature-complete as far as serializing tags goes, but I'd like to eventually implement a serializer for the higher level playlist representation, and also deserialization.

- [x] Serialize steering manifest
- [x] Serialize tags
- [ ] Serialize playlist
- [ ] Deserialize steering manifest
- [ ] Deserialize tags
- [ ] Deserialize playlist