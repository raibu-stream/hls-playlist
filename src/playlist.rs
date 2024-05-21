//! High level representations of extended M3U playlists.

// Copyright 2024 Logan Wemyss
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// A playlist representing a list of renditions and variants of a given piece of media.
pub struct MultivariantPlaylist {
    /// True if all media samples in a Media Segment can be decoded without information
    /// from other segments.
    pub is_independent_segments: bool,

    /// A preferred point at which to start playing a Playlist.
    pub start_offset: Option<StartOffset>,

    /// A list of name value pairs where the name can be substituted for the
    /// value (e.g. `{$<name>}`) in URI lines, quoted string attribute list
    /// values, and hexadecimal-sequence attribute values.
    pub variables: Vec<crate::DefinitionType>,

    /// Groups of renditions that are all alternative renditions of the same content.
    pub renditions_groups: Vec<RenditionGroup>,

    /// A set of [`VariantStream`]s.
    pub variant_streams: Vec<VariantStream>,

    /// The `MediaPlaylist` files containing the I-frames of a multimedia
    /// presentation.
    pub i_frame_streams: Vec<IFrameStream>,

    /// Arbitrary session data.
    pub session_data: Vec<crate::SessionData>,

    /// Encryption keys used in the `MediaPlaylist`s that should be preloaded.
    pub session_key: Vec<crate::EncryptionMethod>,

    /// Identifies a [`crate::steering_manifest::SteeringManifest`].
    pub content_steering: Vec<crate::ContentSteering>,
}

/// Groups of renditions that are all alternative renditions of the same content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenditionGroup {
    /// A group of video renditions.
    Video {
        /// The group id for this group.
        group_id: String,

        /// All the renditions a part of this group.
        renditions: Vec<VideoRendition>,
    },

    /// A group of audio renditions.
    Audio {
        /// The group id for this group.
        group_id: String,

        /// All the renditions a part of this group.
        renditions: Vec<AudioRendition>,
    },

    /// A group of subtitle renditions.
    Subtitles {
        /// The group id for this group.
        group_id: String,

        /// All the renditions a part of this group.
        renditions: Vec<SubtitleRendition>,
    },

    /// A group of closed caption renditions.
    ClosedCaptions {
        /// The group id for this group.
        group_id: String,

        /// All the renditions a part of this group.
        renditions: Vec<ClosedCaptionRendition>,
    },
}

/// A video rendition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoRendition {
    /// Information about this rendition.
    pub info: RenditionInfo,

    /// The URI that identifies the Media Playlist file.
    pub uri: Option<String>,
}

/// A audio rendition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioRendition {
    /// The audio bit depth of the rendition.
    pub bit_depth: Option<u64>,

    /// The audio sample rate of the rendition.
    pub sample_rate: Option<u64>,

    /// Information about the audio channels in the rendition.
    pub channels: Option<crate::AudioChannelInformation>,

    /// Information about this rendition.
    pub info: RenditionInfo,

    /// The URI that identifies the Media Playlist file.
    pub uri: Option<String>,
}

/// A subtitle rendition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubtitleRendition {
    /// Information about this rendition.
    pub info: RenditionInfo,

    /// The URI that identifies the Media Playlist file.
    pub uri: String,
}

/// A closed caption rendition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosedCaptionRendition {
    pub in_stream_id: crate::InStreamId,

    /// The priority in which this rendition should be chosen over another rendition.
    pub priority: crate::RenditionPlaybackPriority,

    /// Information about this rendition.
    pub info: RenditionInfo,
}

/// Information about a given rendition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenditionInfo {
    /// A RFC5646 tag which identifies the primary language used in the Rendition.
    pub language: Option<String>,

    /// A RFC5646 tag which identifies a language that is associated with the Rendition.
    pub assoc_language: Option<String>,

    /// A human-readable description of the Rendition.
    pub name: String,

    /// Indicates that the client may choose to play this Rendition in the absence
    /// of explicit user preference because it matches the current playback environment,
    /// such as chosen system language.
    pub can_auto_select: bool,

    /// Media Characteristic Tags that indicate individual characteristics of this Rendition.
    pub characteristics: Vec<String>,

    /// Allows the URI of a Rendition to change between two distinct downloads of
    /// the `MultivariantPlaylist`.
    pub stable_rendition_id: Option<String>,
}

/// A set of Renditions that can be combined to play the presentation.
#[derive(Debug, Clone, PartialEq)]
pub struct VariantStream {
    /// Metadata for the stream.
    pub stream_info: crate::StreamInf,

    /// Describes the maximum frame rate for all the video in the
    /// `VariantStream`.
    pub frame_rate: Option<f64>,

    /// The group id of the audio [`RenditionGroup`] that should be used when
    /// playing the presentation.
    pub audio_group_id: Option<String>,

    /// The group id of the video [`RenditionGroup`] that should be used when
    /// playing the presentation.
    pub video_group_id: Option<String>,

    /// The group id of the subtitle [`RenditionGroup`] that should be used when
    /// playing the presentation.
    pub subtitles_group_id: Option<String>,

    /// The group id of the closed caption [`RenditionGroup`] that should be used when
    /// playing the presentation.
    pub closed_captions_group_id: Option<String>,

    /// The `MediaPlaylist` that carries a Rendition of the Variant Stream.
    pub uri: String,
}

/// Identifies a `MediaPlaylist` containing the I-frames of a multimedia
/// presentation.
#[derive(Debug, Clone, PartialEq)]
pub struct IFrameStream {
    /// The metadata for this stream.
    pub stream_info: crate::StreamInf,

    /// The group id of the video [`RenditionGroup`] that should be used when
    /// playing the presentation.
    pub video_group_id: Option<String>,

    /// The URI that identifies the I-frame `MediaPlaylist` file.
    pub uri: String,
}

/// A playlist representing a list of `MediaSegment`s and relevant information.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq)]
pub struct MediaPlaylist {
    /// The `MediaSegments` representing segments of the media stream in order.
    pub segments: Vec<MediaSegment>,

    /// A preferred point at which to start playing a Playlist.
    pub start_offset: Option<StartOffset>,

    /// A list of name value pairs where the name can be substituted for the
    /// value (e.g. `{$<name>}`) in URI lines, quoted string attribute list
    /// values, and hexadecimal-sequence attribute values.
    pub variables: Vec<(String, String)>,

    /// True if all media samples in a Media Segment can be decoded without information
    /// from other segments.
    pub is_independent_segments: bool,

    /// An upper bound on the duration of all Media
    /// Segments in the Playlist. The duration of each Media Segment
    /// in a Playlist file, when rounded to the nearest integer, MUST be
    /// less than or equal to the Target Duration.
    pub target_duration: u64,

    /// The media sequence number of the first segment in [`MediaPlaylist::segments`].
    pub first_media_sequence_number: u64,

    /// Allows synchronization between different renditions of the same `VariantStream`
    /// or different `VariantStream`s that have EXT-X-DISCONTINUITY tags in their
    /// Media Playlists.
    pub discontinuity_sequence_number: u64,

    /// True if no more Media Segments will be added to the Media Playlist file.
    pub finished: bool,

    /// Whether or not the playlist is for a mutable append-only stream, or a
    /// static immutable stream.
    pub playlist_type: Option<crate::PlaylistType>,

    /// If Some, indicates the server-recommended minimum distance from
    /// the end of the Playlist at which clients should begin to play
    /// or to which they should seek.
    /// If None, this is set to `target_duration * 3`.
    pub hold_back_seconds: Option<f64>,

    /// True if each Media Segment in the Playlist describes a single I-frame.
    pub iframes_only: bool,

    /// `Some` if the server supports playlist delta updates.,
    pub playlist_delta_updates_information: Option<crate::DeltaUpdateInfo>,

    /// True if the server supports blocking playlist reloads.
    pub supports_blocking_playlist_reloads: bool,

    /// Information about the `PartialSegments` in this playlist.
    pub part_information: Option<PartInformation>,

    /// Information about the playlist that is not associated with
    /// specific Media Segments.
    pub metadata: MediaMetadata,
}

/// Information about `PartialSegments` in a given playlist.
#[derive(Debug, Clone, PartialEq)]
pub struct PartInformation {
    /// If Some, indicates the server-recommended minimum distance from
    /// the end of the Playlist at which clients should begin to play
    /// or to which they should seek when playing in Low-Latency Mode.
    pub part_hold_back_seconds: f64,

    /// An upper bound on the duration of all Partial Segments in the Playlist.
    /// The duration of each Media Segment in a Playlist file, when rounded
    /// to the nearest integer, MUST be less than or equal to the Target Duration.
    pub part_target_duration: u64,
}

/// Information about the playlist that is not associated with
/// specific Media Segments.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaMetadata {
    /// A duration of time with specific attributes.
    pub date_ranges: Vec<crate::DateRange>,

    /// If Some, this indicates information about skipped `MediaSegments`.
    /// If None, there are no skipped `MediaSegments`.
    pub skip: Option<SkipInformation>,

    /// Hints that the client should request a resource before
    /// it is available to be delivered.
    pub preload_hints: Vec<crate::PreloadHint>,

    /// Information about an associated Renditions that is as up-to-date as
    /// the Playlist that contains the report.
    pub rendition_reports: Vec<crate::RenditionReport>,
}

/// Information about skipped `MediaSegments`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkipInformation {
    /// The number of `MediaSegments` that have been skipped.
    pub number_of_skipped_segments: u64,

    /// The list of [`crate::DateRange`] IDs that have been removed
    /// from the Playlist recently.
    pub recently_removed_dataranges: Vec<String>,
}

/// A segment of the larger media file.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaSegment {
    /// The URI Identifying the media resource.
    pub uri: String,

    /// The duration of this `MediaSegment`.
    pub duration_seconds: crate::FloatOrInteger,

    /// An optional human-readable informative title of the Media Segment.
    pub title: Option<String>,

    /// This may contain either a byte range or bitrate, but not both, because they are
    /// mutually exclusive
    pub byte_range_or_bitrate: Option<ByteRangeOrBitrate>,

    /// True if `MediaSegment` is a discontinuity between the Media Segment
    /// that follows it and the one that preceded it.
    pub is_discontinuity: bool,

    /// If Some, represents the encryption method used for this `MediaSegment`.
    /// If None, no encryption is used.
    pub encryption: Option<crate::EncryptionMethod>,

    /// If Some, this `MediaSegment` requires a Media Initialization Section
    /// and the value describes how to acquire it.
    pub media_initialization_section: Option<MediaInitializationSection>,

    /// If Some, the first sample of the `MediaSegment` is associated with this
    /// time.
    pub absolute_time: Option<chrono::DateTime<chrono::FixedOffset>>,

    /// If true, this `MediaSegment` does not contain media data
    /// and should not be loaded by clients.
    pub is_gap: bool,

    /// The partial segments for this `MediaSegment`.
    pub parts: Vec<PartialSegment>,
}

/// A common sequence of bytes to initialize the parser before
/// `MediaSegments` can be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaInitializationSection {
    pub uri: String,
    pub range: Option<crate::ByteRangeWithOffset>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ByteRangeOrBitrate {
    /// This `MediaSegment` is a sub-range of the resource
    /// identified by its URI.
    ByteRange(crate::ByteRange),

    /// The approximate segment bit rate of this `MediaSegment`
    /// in kbps.
    Bitrate(u64),
}

/// A partial slice of a `MediaSegment.`
#[derive(Debug, Clone, PartialEq)]
pub struct PartialSegment {
    /// The URI for this `PartialSegment`.
    pub uri: String,

    /// The duration of this `PartialSegment`.
    pub duration_in_seconds: f64,

    /// True if this `PartialSegment` contains an independent frame.
    pub is_independent: bool,

    /// Some if this `PartialSegment` is a sub-range of the resource specified by the URI.
    pub byte_range: Option<crate::ByteRange>,

    /// True if this `PartialSegment` is not available.
    pub is_gap: bool,
}

/// A preferred point at which to start playing a Playlist.
#[derive(Debug, Clone, PartialEq)]
pub struct StartOffset {
    /// A positive offset indicates a time offset from the beginning of the Playlist.
    /// A negative offset indicates a negative time offset from the end of the last Media
    /// Segment in the Playlist.
    pub offset_in_seconds: f64,

    /// If `true`, clients should start playback at the Media
    /// Segment containing the [`StartOffset::offset_in_seconds`], but should not render
    /// media samples in that segment whose presentation times are prior to the
    /// [`StartOffset::offset_in_seconds`].  If `false`, clients should attempt to render
    /// every media sample in that segment.
    pub is_precise: bool,
}

// impl MediaPlaylist {
//     /// Serializes the `MediaPlaylist` as a extended M3U playlist into `output`.
//     /// Guaranteed to write valid UTF-8 only.
//     ///
//     /// This method makes lots of small calls to write on `output`. If the implementation
//     /// of write on `output` makes a syscall, like with a `TcpStream`, you should wrap it
//     /// in a [`std::io::BufWriter`].
//     ///
//     /// # Errors
//     ///
//     /// May return `Err` when encountering an io error on `output`.
//     pub fn serialize(&self, mut output: impl io::Write) -> io::Result<()> {
//         Tag::M3u.serialize(&mut output)?;
//         todo!();
//     }
// }

// #[cfg(test)]
// mod tests {
//     use crate::{EncryptionMethod, FloatOrInteger, PreloadHint};

//     use super::*;

//     #[test]
//     fn serialize_media_playlist() {
//         let mut output = Vec::new();

//         let playlist = MediaPlaylist {
//             segments: vec![
//                 MediaSegment {
//                     uri: "https://example.com/1.mp4".into(),
//                     duration_seconds: FloatOrInteger::Float(5.045),
//                     title: Some("This is the first thingy!".into()),
//                     byte_range_or_bitrate: Some(ByteRangeOrBitrate::Bitrate(8000)),
//                     is_discontinuity: false,
//                     encryption: Some(EncryptionMethod::Aes128 {
//                         uri: "https://example.com/key.key".into(),
//                         iv: Some(0x0F91_DC05),
//                         key_format: crate::KeyFormat::Identity,
//                         key_format_versions: vec![1, 7, 6],
//                     }),
//                     media_initialization_section: Some(MediaInitializationSection {
//                         uri: "https://example.com/1.mp4".into(),
//                         range: Some(crate::ByteRangeWithOffset {
//                             length_bytes: 400,
//                             start_offset_bytes: 0,
//                         }),
//                     }),
//                     absolute_time: Some(SystemTime::now()),
//                     is_gap: false,
//                     parts: vec![
//                         PartialSegment {
//                             uri: "https://example.com/1.mp4".into(),
//                             duration_in_seconds: 5.045 / 2.0,
//                             is_independent: true,
//                             byte_range: Some(crate::ByteRange {
//                                 length_bytes: 400,
//                                 start_offset_bytes: None,
//                             }),
//                             is_gap: false,
//                         },
//                         PartialSegment {
//                             uri: "https://example.com/1.mp4".into(),
//                             duration_in_seconds: 5.045 / 2.0,
//                             is_independent: false,
//                             byte_range: Some(crate::ByteRange {
//                                 length_bytes: 400,
//                                 start_offset_bytes: Some(400),
//                             }),
//                             is_gap: false,
//                         },
//                     ],
//                 },
//                 MediaSegment {
//                     uri: "https://example.com/2.mp4".into(),
//                     duration_seconds: FloatOrInteger::Float(5.045),
//                     title: Some("This is the second thingy!".into()),
//                     byte_range_or_bitrate: Some(ByteRangeOrBitrate::Bitrate(8000)),
//                     is_discontinuity: false,
//                     encryption: Some(EncryptionMethod::Aes128 {
//                         uri: "https://example.com/key.key".into(),
//                         iv: Some(0x0F91_DC05),
//                         key_format: crate::KeyFormat::Identity,
//                         key_format_versions: vec![1, 7, 6],
//                     }),
//                     media_initialization_section: Some(MediaInitializationSection {
//                         uri: "https://example.com/1.mp4".into(),
//                         range: Some(crate::ByteRangeWithOffset {
//                             length_bytes: 400,
//                             start_offset_bytes: 0,
//                         }),
//                     }),
//                     absolute_time: None,
//                     is_gap: false,
//                     parts: vec![
//                         PartialSegment {
//                             uri: "https://example.com/2.mp4".into(),
//                             duration_in_seconds: 5.045 / 2.0,
//                             is_independent: true,
//                             byte_range: Some(crate::ByteRange {
//                                 length_bytes: 400,
//                                 start_offset_bytes: None,
//                             }),
//                             is_gap: false,
//                         },
//                         PartialSegment {
//                             uri: "https://example.com/2.mp4".into(),
//                             duration_in_seconds: 5.045 / 2.0,
//                             is_independent: false,
//                             byte_range: Some(crate::ByteRange {
//                                 length_bytes: 400,
//                                 start_offset_bytes: Some(400),
//                             }),
//                             is_gap: false,
//                         },
//                     ],
//                 },
//                 MediaSegment {
//                     uri: "https://example.com/3.mp4".into(),
//                     duration_seconds: FloatOrInteger::Float(5.045),
//                     title: Some("This is the third thingy!".into()),
//                     byte_range_or_bitrate: Some(ByteRangeOrBitrate::Bitrate(5000)),
//                     is_discontinuity: false,
//                     encryption: None,
//                     media_initialization_section: None,
//                     absolute_time: None,
//                     is_gap: false,
//                     parts: vec![
//                         PartialSegment {
//                             uri: "https://example.com/3.mp4".into(),
//                             duration_in_seconds: 5.045 / 2.0,
//                             is_independent: true,
//                             byte_range: Some(crate::ByteRange {
//                                 length_bytes: 400,
//                                 start_offset_bytes: None,
//                             }),
//                             is_gap: false,
//                         },
//                         PartialSegment {
//                             uri: "https://example.com/3.mp4".into(),
//                             duration_in_seconds: 5.045 / 2.0,
//                             is_independent: false,
//                             byte_range: Some(crate::ByteRange {
//                                 length_bytes: 400,
//                                 start_offset_bytes: Some(400),
//                             }),
//                             is_gap: false,
//                         },
//                     ],
//                 },
//             ],
//             start_offset: Some(StartOffset {
//                 offset_in_seconds: 2.0,
//                 is_precise: false,
//             }),
//             variables: vec![("cool".into(), "foo".into())],
//             is_independent_segments: false,
//             target_duration: 5,
//             first_media_sequence_number: 0,
//             finished: false,
//             playlist_type: Some(crate::PlaylistType::Event),
//             hold_back_seconds: None,
//             part_information: Some(PartInformation {
//                 part_hold_back_seconds: 3.0 * 3.0,
//                 part_target_duration: 3,
//             }),
//             iframes_only: false,
//             playlist_delta_updates_information: Some(crate::DeltaUpdateInfo {
//                 skip_boundary_seconds: 3.0 * 6.0,
//                 can_skip_dateranges: true,
//             }),
//             supports_blocking_playlist_reloads: true,
//             metadata: MediaMetadata {
//                 date_ranges: vec![],
//                 skip: None,
//                 preload_hints: vec![PreloadHint {
//                     hint_type: crate::PreloadHintType::Part,
//                     uri: "https://example.com/4.mp4".into(),
//                     start_byte_offset: 0,
//                     length_in_bytes: Some(400),
//                 }],
//                 rendition_reports: vec![crate::RenditionReport {
//                     uri: Some("https://example.com/different.m3u8".into()),
//                     last_sequence_number: None,
//                     last_part_index: None,
//                 }],
//             },
//         };

//         playlist.serialize(&mut output).unwrap();

//         assert_eq!(
//             String::from_utf8(output).unwrap(),
//             "TODO"
//         );
//     }
// }
