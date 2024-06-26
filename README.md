# HLS Playlist

A library for serializing and deserializing HLS playlists (aka extended M3U playlists).

As specified by [this updated version of RFC 8216](https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis).

## Usage

### Playlists

```rust
use hls_playlist::playlist::{MediaPlaylist, MediaSegment};
use hls_playlist::{FloatOrInteger};

let playlist = MediaPlaylist {
    segments: vec![
        MediaSegment {
            uri: "https://example.com/1.mp4".into(),
            duration_seconds: FloatOrInteger::Float(5.5),
            title: String::new(),
            byte_range_or_bitrate: None,
            is_discontinuity: false,
            encryption: None,
            media_initialization_section: None,
            absolute_time: None,
            is_gap: false,
            parts: vec![]
        }
    ],
    ..MediaPlaylist::default()
};

let mut output = Vec::new();
playlist.serialize(&mut output).unwrap();

assert_eq!(String::from_utf8(output).unwrap(), "#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:0
#EXTINF:5.5
https://example.com/1.mp4
");
```

### Tags

```rust
use hls_playlist::tags::Tag;

let mut output = vec![];

Tag::M3u.serialize(&mut output).unwrap();
Tag::XStart { offset_seconds: 10.0, is_precise: false }.serialize(&mut output).unwrap();

assert_eq!(String::from_utf8(output).unwrap(), "#EXTM3U
#EXT-X-START:TIME-OFFSET=10
");
```

## Features

* `steering-manifest`: Enables support for serializing and deserializing steering manifests.

## Roadmap

This library is 100% finished and feature-complete as far as serialization goes. I'd like to implement deserialization sometime in the future.

- [x] Serialize steering manifest
- [x] Serialize tags
- [x] Serialize playlist
- [ ] Deserialize steering manifest
- [ ] Deserialize tags
- [ ] Deserialize playlist