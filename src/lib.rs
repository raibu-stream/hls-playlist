#![doc = include_str!("../README.md")]
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
#![warn(clippy::pedantic, clippy::nursery, clippy::enum_glob_use)]
#![allow(
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{collections::HashMap, io, num::NonZeroU8};

pub mod playlist;
pub mod tags;

#[cfg_attr(docsrs, doc(cfg(feature = "steering-manifest")))]
#[cfg(feature = "steering-manifest")]
pub mod steering_manifest;

/// The priority in which a given rendition should be chosen over another rendition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenditionPlaybackPriority {
    /// Indicates that the Rendition contains content that is considered essential to play.
    Default,

    /// The client may choose to play this Rendition in the absence of explicit user
    /// preference because it matches the current playback environment, such as
    /// chosen system language.
    AutoSelect,

    /// This rendition may not be auto selected without explicit user preference.
    None,
}

/// Specifies a Rendition within the segments in the `MediaPlaylist`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InStreamId {
    /// Line 21 Data Services channel.
    Cc1,

    /// Line 21 Data Services channel.
    Cc2,

    /// Line 21 Data Services channel.
    Cc3,

    /// Line 21 Data Services channel.
    Cc4,

    /// A Digital Television Closed Captioning service block number between 1 and 63.
    Service(NonZeroU8),
}

/// Information about the audio channels in a given rendition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioChannelInformation {
    NumberOfChannelsOnly {
        /// The count of audio channels.
        number_of_channels: u64,
    },
    WithAudioCodingIdentifiers {
        /// The count of audio channels.
        number_of_channels: u64,

        /// A list of Audio Coding Identifiers.
        audio_coding_identifiers: Vec<String>,
    },
    WithSpecialUsageIdentifiers {
        /// The count of audio channels.
        number_of_channels: u64,

        /// A list of Audio Coding Identifiers.
        audio_coding_identifiers: Vec<String>,

        /// The audio is binaural.
        binaural: bool,

        /// The audio is pre-processed content that should not be dynamically spatialized.
        immersive: bool,

        /// The audio is a downmix derivative of some other audio.
        downmix: bool,
    },
}

/// Metadata for a given stream.
#[derive(Debug, Clone, PartialEq)]
pub struct StreamInf {
    /// Represents the peak segment bit rate of the Stream.
    pub bandwidth_bits_per_second: u64,

    /// Represents the average segment bit rate of the Stream.
    pub average_bandwidth_bits_per_second: Option<u64>,

    /// An abstract, relative measure of the playback quality-of-experience
    /// of the Variant Stream.
    pub score: Option<f64>,

    /// A list of formats, where each format specifies a media sample type
    /// that is present in the Stream.
    pub codecs: Vec<String>,

    /// Describes media samples with both a backward-compatible base layer
    /// and a newer enhancement layer.
    pub supplemental_codecs: Vec<SupplementalCodec>,

    /// Describes the optimal pixel resolution at which to display the
    /// video in the Stream.
    pub resolution: Option<Resolution>,

    /// Indicates that the stream could fail to play unless the
    /// output is protected by High-bandwidth Digital Content Protection.
    pub hdcp_level: Option<HdcpLevel>,

    /// Indicates that the playback of the stream containing encrypted
    /// `MediaSegments` is to be restricted to devices that guarantee
    /// a certain level of content protection robustness.
    pub allowed_cpc: Vec<ContentProtectionConfiguration>,
    pub video_range: VideoRange,

    /// Indicates whether the video content in the Stream requires
    /// specialized rendering to be properly displayed.
    pub required_video_layout: Vec<VideoChannelSpecifier>,

    /// Allows the URI of the Stream to change between two distinct
    /// downloads of the `MultivariantPlaylist`.
    pub stable_variant_id: Option<String>,

    /// Indicates that the Variant Stream belongs to the identified
    /// Content Steering Pathway.
    pub pathway_id: Option<String>,
}

/// Describes media samples with both a backward-compatible base layer
/// and a newer enhancement layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupplementalCodec {
    supplemental_codec: String,
    compatibility_brands: Vec<String>,
}

/// The High-bandwidth Digital Content Protection level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HdcpLevel {
    /// No High-bandwidth Digital Content Protection.
    None,

    /// Type 0 High-bandwidth Digital Content Protection.
    Type0,

    /// Type 1 High-bandwidth Digital Content Protection.
    Type1,
}

/// A video resolution in pixels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub width: u64,
    pub height: u64,
}

/// Represents required content protection robustness for a given `key_format`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentProtectionConfiguration {
    pub key_format: String,

    /// Classes of playback device that implements the `key_format`
    /// with a certain level of content protection robustness.
    pub cpc_labels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoRange {
    Sdr,
    Hlg,
    Pq,
    Other(String),
}

/// Indicates whether some video content is stereoscopic or not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoChannelSpecifier {
    Stereo,
    Mono,
}

/// Arbitrary session data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionData {
    /// Identifies a particular `SessionData`.
    pub data_id: String,

    /// The data value.
    pub value: crate::SessionDataValue,
}

/// Whether the data is stored inline or identified by a URI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionDataValue {
    /// The data is stored inline.
    Value {
        /// The data value.
        value: String,

        /// The language that the value is in.
        language: Option<String>,
    },

    /// The data is identified by a URI.
    Uri {
        /// The URI identifying the data value.
        uri: String,

        /// The format of the data identified by the URI.
        format: UriFormat,
    },
}

/// The format of the data value.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum UriFormat {
    /// The value is json data.
    #[default]
    Json,

    /// The value is a binary file.
    Raw,
}

/// Information about the encryption method of a given `MediaSegment`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncryptionMethod {
    Aes128 {
        /// A URI that specifies how to obtain the key.
        uri: String,

        /// Specifies an initialization vector to be used with the key.
        iv: Option<u128>,

        /// Specifies how the key is represented in the resource identified by the URI.
        key_format: KeyFormat,

        /// Which versions of the `key_format` are this key is in compliance with.
        key_format_versions: Vec<u64>,
    },
    SampleAes {
        /// A URI that specifies how to obtain the key.
        uri: String,

        /// Specifies an initialization vector to be used with the key.
        iv: Option<u128>,

        /// Which versions of the `key_format` are this key is in compliance with.
        key_format_versions: Vec<u64>,
    },
    SampleAesCtr {
        /// A URI that specifies how to obtain the key.
        uri: String,

        /// Which versions of the `key_format` are this key is in compliance with.
        key_format_versions: Vec<u64>,
    },
}

/// Specifies how a given encryption key is represented.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyFormat {
    Identity,
    Other(String),
}

/// Identifies a [`steering_manifest::SteeringManifest`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentSteering {
    /// The URI identifying the [`steering_manifest::SteeringManifest`].
    pub server_uri: String,
    pub pathway_id: Option<String>,
}

/// A duration of time with specific attributes.
#[derive(Debug, Clone, PartialEq)]
pub struct DateRange {
    /// Uniquely identifies the `DateRange` in a given Playlist.
    pub id: String,

    /// Identifies some set of attributes and their associated value semantics
    /// for `client_attributes`.
    pub class: Option<String>,

    /// The time at which the `DateRange` begins.
    pub start_date: chrono::DateTime<chrono::FixedOffset>,

    /// Indicates when to trigger an action associated with the `DateRange`.
    pub cue: Option<DateRangeCue>,

    /// The time at which the `DateRange` ends.
    pub end_date: Option<chrono::DateTime<chrono::FixedOffset>>,

    /// The duration of the `DateRange` in seconds.
    pub duration_seconds: Option<f64>,

    /// The duration that the `DateRange` is expected to be in seconds.
    pub planned_duration_seconds: Option<f64>,

    /// Various client defined attributes. Keys are prefixed with `X-` and
    /// unprefixed on serialization and deserialization respectively.
    pub client_attributes: HashMap<String, AttributeValue>,

    /// Used to carry SCTE-35 data.
    pub scte35_cmd: Vec<u8>,

    /// Used to carry SCTE-35 data.
    pub scte35_in: Vec<u8>,

    /// Used to carry SCTE-35 data
    pub scte35_out: Vec<u8>,

    /// Indicates that the end of the `DateRange` is equal to the `start_date`
    /// of the range that is closest in time after this `DateRange` and has the same schema
    /// of `client_attributes`.
    pub end_on_next: bool,
}

/// When to trigger an action associated with a given `DateRange`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateRangeCue {
    /// Indicates that an action is to be triggered once and never again.
    pub once: bool,

    /// The relative time at which the action is to be triggered.
    pub position: DateRangeCuePosition,
}

/// The relative time at which a given `DateRange` action is to be triggered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateRangeCuePosition {
    /// Indicates that an action is to be triggered before
    /// playback of the primary asset begins.
    Pre,

    /// Indicates that an action is to be triggered after the
    /// primary asset has been played to its end without error.
    Post,

    Neither,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    String(String),
    Bytes(Vec<u8>),
    Float(f64),
}

/// A hint that the client should request a resource before
/// it is available to be delivered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreloadHint {
    /// Whether the resource is a `PartialSegment` or a `MediaInitializationSection`.
    pub hint_type: PreloadHintType,

    /// The URI to the hinted resource.
    pub uri: String,

    /// The byte offset of the first byte of the hinted resource, from
    /// the beginning of the resource identified by the URI.
    pub start_byte_offset: u64,

    /// If Some, the value is the length of the hinted resource.
    /// If None, the last byte of the hinted resource is the last byte of the
    /// resource identified by the URI.
    pub length_in_bytes: Option<u64>,
}

/// Whether a given resource is a `PartialSegment` or a `MediaInitializationSection`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreloadHintType {
    /// The resource is a `PartialSegment`.
    Part,

    /// The resource is a `MediaInitializationSection`.
    Map,
}

/// Represents a range of bytes in a given resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteRange {
    /// The length of the range in bytes.
    pub length_bytes: u64,

    /// The offset from the start of the resource to the start of the range
    /// in bytes.
    pub start_offset_bytes: Option<u64>,
}

/// Represents a range of bytes in a given resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteRangeWithOffset {
    /// The length of the range in bytes.
    pub length_bytes: u64,

    /// The offset from the start of the resource to the start of the range
    /// in bytes.
    pub start_offset_bytes: u64,
}

/// If `Event`, Media Segments can only be added to the end of the Media Playlist.
/// If `Vod`, the Media Playlist cannot change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaylistType {
    Event,
    Vod,
}

/// Information about the server's playlist delta update capabilities.
#[derive(Debug, Clone, PartialEq)]
pub struct DeltaUpdateInfo {
    pub skip_boundary_seconds: f64,

    /// if the Server can produce Playlist Delta Updates that skip
    /// older EXT-X-DATERANGE tags in addition to Media Segments.
    pub can_skip_dateranges: bool,
}

// TODO: Can we fill in these fields when deserializing a playlist?
/// Information about an associated Rendition that is as up-to-date as
/// the Playlist that contains the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenditionReport {
    /// The URI for the `MediaPlaylist` of the specified rendition.
    pub uri: String,

    /// The media sequence number of the last `MediaSegment` currently
    /// in the specified Rendition.
    pub last_sequence_number: Option<u64>,

    /// The part index of the last `PartialSegment` currently in the
    /// specified rendition whose media sequence number is equal to
    /// `last_sequence_number`.
    pub last_part_index: Option<u64>,
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

#[derive(Debug, Clone, PartialEq)]
pub enum FloatOrInteger {
    Float(f64),
    Integer(u64),
}

impl ByteRange {
    fn serialize(&self, mut output: impl io::Write) -> io::Result<()> {
        write!(output, "{}", self.length_bytes)?;
        if let Some(start_offset_bytes) = self.start_offset_bytes {
            write!(output, "@{start_offset_bytes}")?;
        }

        Ok(())
    }
}

impl ByteRangeWithOffset {
    fn serialize(&self, mut output: impl io::Write) -> io::Result<()> {
        write!(output, "{}@{}", self.length_bytes, self.start_offset_bytes)
    }
}

impl EncryptionMethod {
    fn serialize(&self, mut output: impl io::Write) -> io::Result<()> {
        match self {
            Self::Aes128 { uri, .. } => write!(output, "METHOD=AES-128,URI=\"{uri}\"")?,
            Self::SampleAes { uri, .. } => write!(output, "METHOD=SAMPLE-AES,URI=\"{uri}\"")?,
            Self::SampleAesCtr { uri, .. } => {
                write!(output, "METHOD=SAMPLE-AES-CTR,URI=\"{uri}\"")?;
            }
        }

        match self {
            Self::Aes128 { iv: Some(iv), .. } | Self::SampleAes { iv: Some(iv), .. } => {
                write!(output, ",IV={iv:#X}")?;
            }
            _ => (),
        }

        if let Self::Aes128 {
            key_format: KeyFormat::Other(key_format),
            ..
        } = self
        {
            write!(output, ",KEYFORMAT=\"{key_format}\"")?;
        }

        match self {
            Self::Aes128 {
                key_format_versions,
                ..
            }
            | Self::SampleAes {
                key_format_versions,
                ..
            }
            | Self::SampleAesCtr {
                key_format_versions,
                ..
            } => {
                if !(key_format_versions.is_empty()
                    || key_format_versions.len() == 1 && key_format_versions[0] == 1)
                {
                    write!(output, ",KEYFORMATVERSIONS=\"")?;

                    if key_format_versions.len() == 1 {
                        write!(output, "{}", key_format_versions[0])?;
                    } else {
                        for (i, version) in key_format_versions.iter().enumerate() {
                            if i == key_format_versions.len() - 1 {
                                write!(output, "{version}")?;
                            } else {
                                write!(output, "{version}/")?;
                            }
                        }
                    }

                    write!(output, "\"")?;
                }
            }
        }

        Ok(())
    }
}

impl StreamInf {
    fn serialize(&self, mut output: impl io::Write) -> io::Result<()> {
        write!(output, "BANDWIDTH={}", self.bandwidth_bits_per_second)?;

        if let Some(average_bandwidth) = self.average_bandwidth_bits_per_second {
            write!(output, ",AVERAGE-BANDWIDTH={average_bandwidth}")?;
        }

        if let Some(score) = self.score {
            write!(output, ",SCORE={score}")?;
        }

        if !self.codecs.is_empty() {
            write!(output, ",CODECS=\"")?;

            if self.codecs.len() == 1 {
                write!(output, "{}", self.codecs[0])?;
            } else {
                for (i, codec) in self.codecs.iter().enumerate() {
                    if i == self.codecs.len() - 1 {
                        write!(output, "{codec}")?;
                    } else {
                        write!(output, "{codec},")?;
                    }
                }
            }

            write!(output, "\"")?;
        }

        if !self.supplemental_codecs.is_empty() {
            write!(output, ",SUPPLEMENTAL-CODECS=\"")?;

            if self.supplemental_codecs.len() == 1 {
                self.supplemental_codecs[0].serialize(&mut output)?;
            } else {
                for (i, supplemental_codec) in self.supplemental_codecs.iter().enumerate() {
                    supplemental_codec.serialize(&mut output)?;
                    if i != self.supplemental_codecs.len() - 1 {
                        write!(output, ",")?;
                    }
                }
            }

            write!(output, "\"")?;
        }

        if let Some(resolution) = &self.resolution {
            write!(
                output,
                ",RESOLUTION={}x{}",
                resolution.width, resolution.height
            )?;
        }

        if let Some(level) = &self.hdcp_level {
            match level {
                HdcpLevel::None => write!(output, ",HDCP-LEVEL=NONE")?,
                HdcpLevel::Type0 => write!(output, ",HDCP-LEVEL=TYPE-0")?,
                HdcpLevel::Type1 => write!(output, ",HDCP-LEVEL=TYPE-1")?,
            }
        }

        if !self.allowed_cpc.is_empty() {
            write!(output, ",ALLOWED-CPC=\"")?;

            if self.allowed_cpc.len() == 1 {
                self.allowed_cpc[0].serialize(&mut output)?;
            } else {
                for (i, config) in self.allowed_cpc.iter().enumerate() {
                    config.serialize(&mut output)?;
                    if i != self.allowed_cpc.len() - 1 {
                        write!(output, ",")?;
                    }
                }
            }

            write!(output, "\"")?;
        }

        match &self.video_range {
            VideoRange::Sdr => (),
            VideoRange::Hlg => write!(output, ",VIDEO-RANGE=HLG")?,
            VideoRange::Pq => write!(output, ",VIDEO-RANGE=PQ")?,
            VideoRange::Other(other) => write!(output, ",VIDEO-RANGE={other}")?,
        }

        if !(self.required_video_layout.is_empty()
            || (self.required_video_layout.len() == 1
                && matches!(self.required_video_layout[0], VideoChannelSpecifier::Mono)))
        {
            write!(output, ",REQ-VIDEO-LAYOUT=\"")?;

            if self.required_video_layout.len() == 1 {
                #[allow(clippy::match_on_vec_items)]
                match self.required_video_layout[0] {
                    VideoChannelSpecifier::Stereo => write!(output, "CH-STEREO")?,
                    VideoChannelSpecifier::Mono => unreachable!(),
                }
            } else {
                for (i, config) in self.required_video_layout.iter().enumerate() {
                    match config {
                        VideoChannelSpecifier::Stereo => write!(output, "CH-STEREO")?,
                        VideoChannelSpecifier::Mono => write!(output, "CH-MONO")?,
                    }
                    if i != self.required_video_layout.len() - 1 {
                        write!(output, ",")?;
                    }
                }
            }

            write!(output, "\"")?;
        }

        if let Some(id) = &self.stable_variant_id {
            write!(output, ",STABLE-VARIANT-ID=\"{id}\"")?;
        }

        if let Some(id) = &self.pathway_id {
            write!(output, ",PATHWAY-ID=\"{id}\"")?;
        }

        Ok(())
    }
}

impl SupplementalCodec {
    fn serialize(&self, mut output: impl io::Write) -> io::Result<()> {
        write!(output, "{}", self.supplemental_codec)?;

        for brand in &self.compatibility_brands {
            write!(output, "/{brand}")?;
        }

        Ok(())
    }
}

impl ContentProtectionConfiguration {
    fn serialize(&self, mut output: impl io::Write) -> io::Result<()> {
        write!(output, "{}:", self.key_format)?;

        if self.cpc_labels.len() == 1 {
            write!(output, "{}", self.cpc_labels[0])?;
        } else {
            for (i, label) in self.cpc_labels.iter().enumerate() {
                if i == self.cpc_labels.len() - 1 {
                    write!(output, "{label}")?;
                } else {
                    write!(output, "{label}/")?;
                }
            }
        }

        Ok(())
    }
}
