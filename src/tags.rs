//! Representations of extended M3U playlist tags.

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

use std::time::SystemTime;

/// A representation of all possible tags.
#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    MediaPlaylistTag(MediaPlaylistTag),
    MediaSegmentTag(MediaSegmentTag),
    MediaMetadataTag(MediaMetadataTag),
    MultivariantPlaylistTag(MultivariantPlaylistTag),

    /// The EXT-X-VERSION tag indicates the compatibility version of the
    /// Playlist file, its associated media, and its server.
    XVersion {
        version: u8,
    },

    /// The EXTM3U tag indicates that the file is an Extended M3U Playlist file.
    M3u,

    /// The EXT-X-DEFINE tag provides a Playlist variable definition or
    /// declaration.
    XDefine(DefinitionType),

    /// The EXT-X-START tag indicates a preferred point at which to start
    /// playing a Playlist.
    XStart {
        offset_seconds: f64,
        is_precise: bool,
    },

    /// The EXT-X-INDEPENDENT-SEGMENTS tag indicates that all media samples
    /// in a Media Segment can be decoded without information from other
    /// segments.
    XIndependentSegments,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefinitionType {
    /// The variable is defined here.
    Inline { name: String, value: String },

    /// Use a variable defined in the Multivariant Playlist that referenced
    /// this playlist.
    Import { name: String },

    /// Use the value of the query parameter named `name` from the current
    /// playlist's URI. If the URI is redirected, look for the query
    /// parameter in the 30x response URI.
    QueryParameter { name: String },
}

/// A tag applying to a `MediaSegment`
#[derive(Debug, Clone, PartialEq)]
pub enum MediaSegmentTag {
    /// The EXTINF tag specifies the duration of a Media Segment.
    Inf {
        duration_seconds: f64,
        title: Option<String>,
    },

    /// The EXT-X-BYTERANGE tag indicates that a Media Segment is a sub-range
    /// of the resource identified by its URI.
    XByterange(crate::ByteRange),

    /// The EXT-X-DISCONTINUITY tag indicates a discontinuity between the
    /// Media Segment that follows it and the one that preceded it.
    XDiscontinuity,

    /// Media Segments MAY be encrypted.  The EXT-X-KEY tag specifies how to
    /// decrypt them.
    XKey(Option<crate::EncryptionMethod>),

    /// The EXT-X-MAP tag specifies how to obtain the Media Initialization
    /// Section required to parse the applicable Media Segments.
    XMap {
        uri: String,
        range: Option<crate::ByteRangeWithOffset>,
    },

    /// The EXT-X-PROGRAM-DATE-TIME tag associates the first sample of a
    /// Media Segment with an absolute date and/or time.
    XProgramDateTime(SystemTime),

    /// The EXT-X-GAP tag indicates that the segment URI to which it applies
    /// does not contain media data and SHOULD NOT be loaded by clients.
    XGap,

    XBitrate {
        kbps: u64,
    },

    /// The EXT-X-PART tag identifies a Partial Segment.
    XPart {
        uri: String,
        duration_seconds: f64,
        is_independent: bool,
        byte_range: Option<crate::ByteRange>,
        is_gap: bool,
    },
}

/// Media Playlist tags describe global parameters of the Media Playlist.
/// There MUST NOT be more than one Media Playlist tag of each type in
/// any Media Playlist.
#[derive(Debug, Clone, PartialEq)]
pub enum MediaPlaylistTag {
    /// The EXT-X-TARGETDURATION tag specifies the maximum Media Segment
    /// duration.
    XTargetDuration { target_duration_seconds: u64 },

    /// The EXT-X-MEDIA-SEQUENCE tag indicates the Media Sequence Number of
    /// the first Media Segment that appears in a Playlist file.
    XMediaSequence { sequence_number: u64 },

    /// The EXT-X-DISCONTINUITY-SEQUENCE tag allows synchronization between
    /// different Renditions of the same Variant Stream or different Variant
    /// Streams that have EXT-X-DISCONTINUITY tags in their Media Playlists.
    XDiscontinuitySequence { sequence_number: u64 },

    /// The EXT-X-ENDLIST tag indicates that no more Media Segments will be
    /// added to the Media Playlist file.
    XEndList,

    /// The EXT-X-PLAYLIST-TYPE tag provides mutability information about the
    /// Media Playlist file.
    XPlaylistType(crate::PlaylistType),

    /// The EXT-X-I-FRAMES-ONLY tag indicates that each Media Segment in the
    /// Playlist describes a single I-frame.
    XIFramesOnly,

    /// The EXT-X-PART-INF tag provides information about the Partial
    /// Segments in the Playlist.
    XPartInf { part_target_duration_seconds: f64 },

    /// The EXT-X-SERVER-CONTROL tag allows the Server to indicate support
    /// for Delivery Directives.
    XServerControl {
        delta_update_info: Option<crate::DeltaUpdateInfo>,
        hold_back: Option<f64>,
        part_hold_back: Option<f64>,
        can_block_reload: bool,
    },
}

/// Multivariant Playlist tags define the variant streams, renditions, and
/// other global parameters of the presentation.
#[derive(Debug, Clone, PartialEq)]
pub enum MultivariantPlaylistTag {
    /// The EXT-X-MEDIA tag is used to relate Media Playlists that contain
    /// alternative Renditions of the same content.
    XMedia {
        media_type: MediaType,
        group_id: String,
        language: Option<String>,
        assoc_language: Option<String>,
        name: String,
        stable_rendition_id: Option<String>,
        playback_priority: crate::RenditionPlaybackPriority,
        characteristics: Option<Vec<String>>,
    },

    /// The EXT-X-STREAM-INF tag specifies a Variant Stream, which is a set
    /// of Renditions that can be combined to play the presentation.
    XStreamInf {
        stream_inf: crate::StreamInf,
        frame_rate: Option<f64>,
        audio_group_id: Option<String>,
        video_group_id: Option<String>,
        subtitles_group_id: Option<String>,
        closed_captions_group_id: Option<String>,
        uri: String,
    },

    /// The EXT-X-I-FRAME-STREAM-INF tag identifies a Media Playlist file
    /// containing the I-frames of a multimedia presentation.
    XIFrameStreamInf {
        stream_inf: crate::StreamInf,
        video_group_id: Option<String>,
        uri: String,
    },

    /// The EXT-X-SESSION-DATA tag allows arbitrary session data to be
    /// carried in a Multivariant Playlist.
    XSessionData(crate::SessionData),

    /// The EXT-X-SESSION-KEY tag allows encryption keys from Media Playlists
    /// to be specified in a Master Playlist.
    XSessionKey(crate::EncryptionMethod),

    /// The EXT-X-CONTENT-STEERING tag allows a server to provide a Content
    /// Steering (Section 7) Manifest.
    XContentSteering(crate::ContentSteering),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaType {
    Audio {
        uri: Option<String>,
        channels: Option<crate::AudioChannelInformation>,
        bit_depth: Option<u64>,
        sample_rate: Option<u64>,
    },
    Video {
        uri: Option<String>,
    },
    Subtitles {
        uri: String,
        forced: bool,
    },
    ClosedCaptions {
        in_stream_id: crate::InStreamId,
    },
}

/// A tag describing metadata about a given `MediaPlaylist`.
#[derive(Debug, Clone, PartialEq)]
pub enum MediaMetadataTag {
    /// The EXT-X-DATERANGE tag associates a Date Range (i.e., a range of
    /// time defined by a starting and ending date) with a set of attribute/
    /// value pairs.
    XDateRange(crate::DateRange),

    /// A server produces a Playlist Delta Update by replacing
    /// tags earlier than the Skip Boundary with an EXT-X-SKIP tag.
    XSkip {
        number_of_skipped_segments: u64,
        recently_removed_dataranges: Vec<String>,
    },

    /// The EXT-X-PRELOAD-HINT tag allows a Client loading media from a live
    /// stream to reduce the time to obtain a resource from the Server by
    /// issuing its request before the resource is available to be delivered.
    XPreloadHint(crate::PreloadHint),

    /// The EXT-X-RENDITION-REPORT tag carries information about an
    /// associated Rendition that is as up-to-date as the Playlist that
    /// contains it.
    XRenditionReport(crate::RenditionReport),
}

// impl Tag {
//     pub fn serialize(&self, output: impl Write) {
//         todo!()
//     }
// }
