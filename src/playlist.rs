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

use std::time::SystemTime;

/// Either a `MultivariantPlaylist` or `MediaPlaylist`.
pub enum Playlist {
    Master(MultivariantPlaylist),
    Media(MediaPlaylist),
}

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
    pub variables: Vec<(String, String)>,

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
    info: RenditionInfo,

    /// The URI that identifies the Media Playlist file.
    uri: Option<String>,
}

/// A audio rendition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioRendition {
    /// The audio bit depth of the rendition.
    bit_depth: Option<u64>,

    /// The audio sample rate of the rendition.
    sample_rate: Option<u64>,

    /// Information about the audio channels in the rendition.
    channels: Option<crate::AudioChannelInformation>,

    /// Information about this rendition.
    info: RenditionInfo,

    /// The URI that identifies the Media Playlist file.
    uri: Option<String>,
}

/// A subtitle rendition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubtitleRendition {
    /// Information about this rendition.
    info: RenditionInfo,

    /// The URI that identifies the Media Playlist file.
    uri: String,
}

/// A closed caption rendition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosedCaptionRendition {
    in_stream_id: crate::InStreamId,

    /// The priority in which this rendition should be chosen over another rendition.
    priority: crate::RenditionPlaybackPriority,

    /// Information about this rendition.
    info: RenditionInfo,
}

/// Information about a given rendition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenditionInfo {
    /// A RFC5646 tag which identifies the primary language used in the Rendition.
    language: Option<String>,

    /// A RFC5646 tag which identifies a language that is associated with the Rendition.
    assoc_language: Option<String>,

    /// A human-readable description of the Rendition.
    name: String,

    /// Indicates that the client may choose to play this Rendition in the absence
    /// of explicit user preference because it matches the current playback environment,
    /// such as chosen system language.
    can_auto_select: bool,

    /// Media Characteristic Tags that indicate individual characteristics of this Rendition.
    characteristics: Vec<String>,

    /// Allows the URI of a Rendition to change between two distinct downloads of
    /// the `MultivariantPlaylist`.
    stable_rendition_id: Option<String>,
}

/// A set of Renditions that can be combined to play the presentation.
#[derive(Debug, Clone, PartialEq)]
pub struct VariantStream {
    /// Metadata for the stream.
    stream_info: crate::StreamInf,

    /// Describes the maximum frame rate for all the video in the
    /// `VariantStream`.
    frame_rate: Option<f64>,

    /// The group id of the audio [`RenditionGroup`] that should be used when
    /// playing the presentation.
    audio_group_id: Option<String>,

    /// The group id of the video [`RenditionGroup`] that should be used when
    /// playing the presentation.
    video_group_id: Option<String>,

    /// The group id of the subtitle [`RenditionGroup`] that should be used when
    /// playing the presentation.
    subtitles_group_id: Option<String>,

    /// The group id of the closed caption [`RenditionGroup`] that should be used when
    /// playing the presentation.
    closed_captions_group_id: Option<String>,

    /// The `MediaPlaylist` that carries a Rendition of the Variant Stream.
    uri: String,
}

/// Identifies a `MediaPlaylist` containing the I-frames of a multimedia
/// presentation.
#[derive(Debug, Clone, PartialEq)]
pub struct IFrameStream {
    /// The metadata for this stream.
    stream_info: crate::StreamInf,

    /// The group id of the video [`RenditionGroup`] that should be used when
    /// playing the presentation.
    video_group_id: Option<String>,

    /// The URI that identifies the I-frame `MediaPlaylist` file.
    uri: String,
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

    /// An upper bound on the duration of all Partial Segments in the Playlist.
    /// The duration of each Media Segment in a Playlist file, when rounded
    /// to the nearest integer, MUST be less than or equal to the Target Duration.
    pub part_target_duration: Option<u64>,

    /// The media sequence number of the first segment in [`MediaPlaylist::segments`].
    pub first_media_sequence_number: u64,
    // pub discontinuity_sequence_number: todo!(), // TODO: What is this?
    /// True if no more Media Segments will be added to the Media Playlist file.
    pub finished: bool,

    /// Whether or not the playlist is for a live mutable stream, or a
    /// static immutable stream. `Vod` playlists cannot change.
    pub playlist_type: Option<crate::PlaylistType>,

    /// If Some, indicates the server-recommended minimum distance from
    /// the end of the Playlist at which clients should begin to play
    /// or to which they should seek.
    /// If None, this is set to `target_duration * 3`.
    pub hold_back_seconds: Option<f64>,

    /// If Some, indicates the server-recommended minimum distance from
    /// the end of the Playlist at which clients should begin to play
    /// or to which they should seek when playing in Low-Latency Mode.
    pub part_hold_back_seconds: Option<f64>,

    /// True if each Media Segment in the Playlist describes a single I-frame.
    pub iframes_only: bool,

    /// `Some` if the server supports playlist delta updates.,
    pub playlist_delta_updates_information: Option<crate::DeltaUpdateInfo>,

    /// True if the server supports blocking playlist reloads.
    pub supports_blocking_playlist_reloads: bool,

    /// Information about the playlist that is not associated with
    /// specific Media Segments.
    pub metadata: MediaMetadata,
}

/// Information about the playlist that is not associated with
/// specific Media Segments.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaMetadata {
    /// A duration of time with specific attributes.
    date_ranges: Vec<crate::DateRange>,

    /// If Some, this indicates information about skipped `MediaSegments`.
    /// If None, there are no skipped `MediaSegments`.
    skip: Option<SkipInformation>,

    /// Hints that the client should request a resource before
    /// it is available to be delivered.
    preload_hints: Vec<crate::PreloadHint>,

    /// Information about an associated Renditions that is as up-to-date as
    /// the Playlist that contains the report.
    rendition_reports: Vec<crate::RenditionReport>,
}

/// Information about skipped `MediaSegments`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkipInformation {
    /// The number of `MediaSegments` that have been skipped.
    number_of_skipped_segments: u64,

    /// The list of [`crate::DateRange`] IDs that have been removed
    /// from the Playlist recently.
    recently_removed_dataranges: Vec<String>,
}

/// A segment of the larger media file.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaSegment {
    /// The URI Identifying the media resource.
    uri: String,

    /// The duration of this `MediaSegment`.
    duration_seconds: FloatOrInteger,

    /// An optional human-readable informative title of the Media Segment.
    title: Option<String>,

    /// This may contain either a byte range or bitrate, but not both, because they are
    /// mutually exclusive
    byte_range_or_bitrate: Option<ByteRangeOrBitrate>,

    /// True if `MediaSegment` is a discontinuity between the Media Segment
    /// that follows it and the one that preceded it.
    is_discontinuity: bool,

    /// If Some, represents the encryption method used for this `MediaSegment`.
    /// If None, no encryption is used.
    encryption: Option<crate::EncryptionMethod>,

    /// If Some, this `MediaSegment` requires a Media Initialization Section
    /// and the value describes how to acquire it.
    media_initialization_section: Option<MediaInitializationSection>,

    /// If Some, the first sample of the `MediaSegment` is associated with this
    /// time.
    absolute_time: Option<SystemTime>,

    /// If true, this `MediaSegment` does not contain media data
    /// and should not be loaded by clients.
    is_gap: bool,

    /// The partial segments for this `MediaSegment`.
    parts: Vec<PartialSegment>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FloatOrInteger {
    Float(f64),
    Integer(u64),
}

/// A common sequence of bytes to initialize the parser before
/// `MediaSegments` can be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaInitializationSection {
    uri: String,
    range: Option<crate::ByteRangeWithOffset>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ByteRangeOrBitrate {
    /// This `MediaSegment` is a sub-range of the resource
    /// identified by its URI.
    ByteRange(crate::ByteRange),

    /// The approximate segment bit rate of this `MediaSegment`.
    Bitrate(u64),
}

/// A partial slice of a `MediaSegment.`
#[derive(Debug, Clone, PartialEq)]
pub struct PartialSegment {
    /// The URI for this `PartialSegment`.
    uri: String,

    /// The duration of this `PartialSegment`.
    duration_in_seconds: f64,

    /// True if this `PartialSegment` contains an independent frame.
    is_independent: bool,

    /// True if this `PartialSegment` is a sub-range of the resource specified by the URI.
    byte_range: Option<crate::ByteRange>,

    /// True if this `PartialSegment` is not available.
    is_gap: bool,
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
