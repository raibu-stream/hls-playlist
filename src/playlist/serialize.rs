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

use super::{
    ByteRangeOrBitrate, IFrameStream, MediaMetadata, MediaPlaylist, MediaSegment,
    MultivariantPlaylist, RenditionGroup, VariantStream,
};
use crate::tags::Tag;
use std::{cmp::max, io};

impl MediaPlaylist {
    /// Serializes the `MediaPlaylist` as a extended M3U playlist into `output`.
    /// Guaranteed to write valid UTF-8 only.
    ///
    /// This method makes lots of small calls to write on `output`. If the implementation
    /// of write on `output` makes a syscall, like with a `TcpStream`, you should wrap it
    /// in a [`std::io::BufWriter`].
    ///
    /// # Note
    ///
    /// This method is not guaranteed to write a valid M3U playlist. It's your job to create
    /// valid input.
    ///
    /// # Errors
    ///
    /// May return `Err` when encountering an io error on `output`.
    pub fn serialize(&self, mut output: impl io::Write) -> io::Result<()> {
        Tag::M3u.serialize(&mut output)?;
        let version = self.get_version();
        if version != 1 {
            Tag::XVersion {
                version: self.get_version(),
            }
            .serialize(&mut output)?;
        }

        for variable in &self.variables {
            Tag::XDefine(variable.clone()).serialize(&mut output)?;
        }
        if self.is_independent_segments {
            Tag::XIndependentSegments.serialize(&mut output)?;
        }
        if let Some(offset) = &self.start_offset {
            Tag::XStart {
                offset_seconds: offset.offset_in_seconds,
                is_precise: offset.is_precise,
            }
            .serialize(&mut output)?;
        }

        Tag::XTargetDuration {
            target_duration_seconds: self.target_duration,
        }
        .serialize(&mut output)?;
        if self.first_media_sequence_number != 0 {
            Tag::XMediaSequence {
                sequence_number: self.first_media_sequence_number,
            }
            .serialize(&mut output)?;
        }
        if self.discontinuity_sequence_number != 0 {
            Tag::XDiscontinuitySequence {
                sequence_number: self.discontinuity_sequence_number,
            }
            .serialize(&mut output)?;
        }
        if self.finished {
            Tag::XEndList.serialize(&mut output)?;
        }
        if let Some(playlist_type) = &self.playlist_type {
            Tag::XPlaylistType(playlist_type.clone()).serialize(&mut output)?;
        }
        if self.iframes_only {
            Tag::XIFramesOnly.serialize(&mut output)?;
        }
        if let Some(part_information) = &self.part_information {
            Tag::XPartInf {
                part_target_duration_seconds: part_information.part_target_duration,
            }
            .serialize(&mut output)?;
        }
        if self.playlist_delta_updates_information.is_some()
            || self.hold_back_seconds.is_some()
            || self.part_information.is_some()
            || self.supports_blocking_playlist_reloads
        {
            Tag::XServerControl {
                delta_update_info: self.playlist_delta_updates_information.clone(),
                hold_back: self.hold_back_seconds,
                part_hold_back: self
                    .part_information
                    .clone()
                    .map(|info| info.part_hold_back_seconds),
                can_block_reload: self.supports_blocking_playlist_reloads,
            }
            .serialize(&mut output)?;
        }

        self.metadata.serialize(&mut output)?;

        let mut last_media_segment = &MediaSegment {
            uri: String::new(),
            duration_seconds: crate::FloatOrInteger::Integer(0),
            title: String::new(),
            byte_range_or_bitrate: None,
            is_discontinuity: false,
            encryption: None,
            media_initialization_section: None,
            absolute_time: None,
            is_gap: false,
            parts: vec![],
        };
        for segment in &self.segments {
            segment.serialize(last_media_segment, &mut output)?;
            last_media_segment = segment;
        }

        Ok(())
    }

    fn get_version(&self) -> u8 {
        let mut version = 1;

        let mut has_map = false;
        for segment in &self.segments {
            if let Some(method) = &segment.encryption {
                if let crate::EncryptionMethod::Aes128 { iv, key_format, .. } = method {
                    if iv.is_some() {
                        version = max(version, 2);
                    }

                    if let crate::KeyFormat::Other(_) = key_format {
                        version = 5;
                    }
                } else if let crate::EncryptionMethod::SampleAes { .. } = method {
                    version = 5;
                }

                let (crate::EncryptionMethod::Aes128 {
                    key_format_versions,
                    ..
                }
                | crate::EncryptionMethod::SampleAes {
                    key_format_versions,
                    ..
                }
                | crate::EncryptionMethod::SampleAesCtr {
                    key_format_versions,
                    ..
                }) = method;
                for key_version in key_format_versions {
                    if *key_version != 1 {
                        version = 5;
                        break;
                    }
                }
            }

            if let crate::FloatOrInteger::Float(_) = segment.duration_seconds {
                version = max(version, 3);
            }

            if let Some(ByteRangeOrBitrate::ByteRange(_)) = segment.byte_range_or_bitrate {
                version = max(version, 4);
            }

            if segment.media_initialization_section.is_some() {
                has_map = true;
                version = 5;
                break;
            }
        }

        if self.iframes_only {
            version = max(version, 4);
        } else if has_map {
            version = 6;
        }

        // NOTE: Might be wrong? This is just checking whether we define any
        // variables, not if we use variable substitution. And what if we use
        // variable substitution, but define no variables? Should be a parse
        // error anyways right? But maybe not in the lower versions?
        if !self.variables.is_empty() {
            version = 8;
        }

        if let Some(skip_information) = &self.metadata.skip {
            if skip_information.recently_removed_dataranges.is_empty() {
                version = 9;
            } else {
                version = 10;
            }
        }

        for variable in &self.variables {
            if let crate::DefinitionType::QueryParameter { .. } = variable {
                version = 11;
            }
        }

        version
    }
}

impl MediaMetadata {
    fn serialize(&self, mut output: impl io::Write) -> io::Result<()> {
        for date_range in &self.date_ranges {
            Tag::XDateRange(date_range.clone()).serialize(&mut output)?;
        }

        if let Some(skip) = &self.skip {
            Tag::XSkip {
                number_of_skipped_segments: skip.number_of_skipped_segments,
                recently_removed_dataranges: skip.recently_removed_dataranges.clone(),
            }
            .serialize(&mut output)?;
        }

        for hint in &self.preload_hints {
            Tag::XPreloadHint(hint.clone()).serialize(&mut output)?;
        }

        for report in &self.rendition_reports {
            Tag::XRenditionReport(report.clone()).serialize(&mut output)?;
        }

        Ok(())
    }
}

impl MediaSegment {
    fn serialize(&self, last_media_segment: &Self, mut output: impl io::Write) -> io::Result<()> {
        if self.is_discontinuity {
            Tag::XDiscontinuity.serialize(&mut output)?;
        }

        Tag::Inf {
            duration_seconds: self.duration_seconds.clone(),
            title: self.title.clone(),
        }
        .serialize(&mut output)?;

        if let Some(byte_range_or_bitrate) = &self.byte_range_or_bitrate {
            match byte_range_or_bitrate {
                ByteRangeOrBitrate::ByteRange(byte_range) => {
                    Tag::XByterange(byte_range.clone()).serialize(&mut output)?;
                }
                ByteRangeOrBitrate::Bitrate(kbps) => {
                    if self.byte_range_or_bitrate != last_media_segment.byte_range_or_bitrate {
                        Tag::XBitrate { kbps: *kbps }.serialize(&mut output)?;
                    }
                }
            }
        }

        if self.encryption != last_media_segment.encryption {
            Tag::XKey(self.encryption.clone()).serialize(&mut output)?;
        }

        if let Some(map) = &self.media_initialization_section {
            if self.media_initialization_section != last_media_segment.media_initialization_section
            {
                Tag::XMap {
                    uri: map.uri.clone(),
                    range: map.range.clone(),
                }
                .serialize(&mut output)?;
            }
        }

        if let Some(time) = self.absolute_time {
            Tag::XProgramDateTime(time).serialize(&mut output)?;
        }

        if self.is_gap {
            Tag::XGap.serialize(&mut output)?;
        }

        for part in &self.parts {
            Tag::XPart {
                uri: part.uri.clone(),
                duration_seconds: part.duration_in_seconds,
                is_independent: part.is_independent,
                byte_range: part.byte_range.clone(),
                is_gap: part.is_gap,
            }
            .serialize(&mut output)?;
        }

        writeln!(output, "{}", self.uri)?;

        Ok(())
    }
}

impl MultivariantPlaylist {
    /// Serializes the `MultivariantPlaylist` as a extended M3U playlist into `output`.
    /// Guaranteed to write valid UTF-8 only.
    ///
    /// This method makes lots of small calls to write on `output`. If the implementation
    /// of write on `output` makes a syscall, like with a `TcpStream`, you should wrap it
    /// in a [`std::io::BufWriter`].
    ///
    /// # Note
    ///
    /// This method is not guaranteed to write a valid M3U playlist. It's your job to create
    /// valid input.
    ///
    /// # Errors
    ///
    /// May return `Err` when encountering an io error on `output`.
    pub fn serialize(&self, mut output: impl io::Write) -> io::Result<()> {
        Tag::M3u.serialize(&mut output)?;
        let version = self.get_version();
        if version != 1 {
            Tag::XVersion {
                version: self.get_version(),
            }
            .serialize(&mut output)?;
        }

        for variable in &self.variables {
            Tag::XDefine(variable.clone()).serialize(&mut output)?;
        }
        if self.is_independent_segments {
            Tag::XIndependentSegments.serialize(&mut output)?;
        }
        if let Some(offset) = &self.start_offset {
            Tag::XStart {
                offset_seconds: offset.offset_in_seconds,
                is_precise: offset.is_precise,
            }
            .serialize(&mut output)?;
        }

        for rendition_group in &self.renditions_groups {
            rendition_group.serialize(&mut output)?;
        }

        for variant_stream in &self.variant_streams {
            variant_stream.clone().serialize(&mut output)?;
        }

        for i_frame_stream in &self.i_frame_streams {
            i_frame_stream.clone().serialize(&mut output)?;
        }

        for data in &self.session_data {
            data.clone().serialize(&mut output)?;
        }

        for key in &self.session_key {
            Tag::XSessionKey(key.clone()).serialize(&mut output)?;
        }

        for content_steering in &self.content_steering {
            Tag::XContentSteering(content_steering.clone()).serialize(&mut output)?;
        }

        Ok(())
    }

    fn get_version(&self) -> u8 {
        let mut version = 1;

        'outer: for rendition_group in &self.renditions_groups {
            if let RenditionGroup::ClosedCaptions { renditions, .. } = rendition_group {
                for rendition in renditions {
                    if let crate::InStreamId::Service(_) = rendition.in_stream_id {
                        version = 7;
                        break 'outer;
                    }
                }
            }
        }

        // NOTE: Might be wrong? This is just checking whether we define any
        // variables, not if we use variable substitution. And what if we use
        // variable substitution, but define no variables? Should be a parse
        // error anyways right? But maybe not in the lower versions?
        if !self.variables.is_empty() {
            version = 8;
        }

        for variable in &self.variables {
            if let crate::DefinitionType::QueryParameter { .. } = variable {
                version = 11;
            }
        }

        for stream in &self.variant_streams {
            if !stream.stream_info.required_video_layout.is_empty() {
                version = 12;
            }
        }

        version
    }
}

impl crate::SessionData {
    fn serialize(self, mut output: impl io::Write) -> io::Result<()> {
        Tag::XSessionData(self).serialize(&mut output)?;

        Ok(())
    }
}

impl IFrameStream {
    fn serialize(self, mut output: impl io::Write) -> io::Result<()> {
        Tag::XIFrameStreamInf {
            stream_inf: self.stream_info,
            video_group_id: self.video_group_id,
            uri: self.uri,
        }
        .serialize(&mut output)?;

        Ok(())
    }
}

impl VariantStream {
    fn serialize(self, mut output: impl io::Write) -> io::Result<()> {
        Tag::XStreamInf {
            stream_inf: self.stream_info,
            frame_rate: self.frame_rate,
            audio_group_id: self.audio_group_id,
            video_group_id: self.video_group_id,
            subtitles_group_id: self.subtitles_group_id,
            closed_captions_group_id: self.closed_captions_group_id,
            uri: self.uri,
        }
        .serialize(&mut output)?;

        Ok(())
    }
}

impl RenditionGroup {
    fn serialize(&self, mut output: impl io::Write) -> io::Result<()> {
        match self {
            Self::Video {
                group_id,
                renditions,
            } => {
                for rendition in renditions {
                    Tag::XMedia {
                        media_type: crate::tags::MediaType::Video {
                            uri: rendition.uri.clone(),
                        },
                        group_id: group_id.clone(),
                        language: rendition.info.language.clone(),
                        assoc_language: rendition.info.assoc_language.clone(),
                        name: rendition.info.name.clone(),
                        stable_rendition_id: rendition.info.stable_rendition_id.clone(),
                        playback_priority: rendition.info.priority.clone(),
                        characteristics: rendition.info.characteristics.clone(),
                    }
                    .serialize(&mut output)?;
                }
            }
            Self::Audio {
                group_id,
                renditions,
            } => {
                for rendition in renditions {
                    Tag::XMedia {
                        media_type: crate::tags::MediaType::Audio {
                            uri: rendition.uri.clone(),
                            channels: rendition.channels.clone(),
                            bit_depth: rendition.bit_depth,
                            sample_rate: rendition.sample_rate,
                        },
                        group_id: group_id.clone(),
                        language: rendition.info.language.clone(),
                        assoc_language: rendition.info.assoc_language.clone(),
                        name: rendition.info.name.clone(),
                        stable_rendition_id: rendition.info.stable_rendition_id.clone(),
                        playback_priority: rendition.info.priority.clone(),
                        characteristics: rendition.info.characteristics.clone(),
                    }
                    .serialize(&mut output)?;
                }
            }
            Self::Subtitles {
                group_id,
                renditions,
            } => {
                for rendition in renditions {
                    Tag::XMedia {
                        media_type: crate::tags::MediaType::Subtitles {
                            uri: rendition.uri.clone(),
                            forced: rendition.forced,
                        },
                        group_id: group_id.clone(),
                        language: rendition.info.language.clone(),
                        assoc_language: rendition.info.assoc_language.clone(),
                        name: rendition.info.name.clone(),
                        stable_rendition_id: rendition.info.stable_rendition_id.clone(),
                        playback_priority: rendition.info.priority.clone(),
                        characteristics: rendition.info.characteristics.clone(),
                    }
                    .serialize(&mut output)?;
                }
            }
            Self::ClosedCaptions {
                group_id,
                renditions,
            } => {
                for rendition in renditions {
                    Tag::XMedia {
                        media_type: crate::tags::MediaType::ClosedCaptions {
                            in_stream_id: rendition.in_stream_id.clone(),
                        },
                        group_id: group_id.clone(),
                        language: rendition.info.language.clone(),
                        assoc_language: rendition.info.assoc_language.clone(),
                        name: rendition.info.name.clone(),
                        stable_rendition_id: rendition.info.stable_rendition_id.clone(),
                        playback_priority: rendition.info.priority.clone(),
                        characteristics: rendition.info.characteristics.clone(),
                    }
                    .serialize(&mut output)?;
                }
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU8;

    use crate::{
        playlist::{
            ClosedCaptionRendition, MediaInitializationSection, PartInformation, PartialSegment,
            RenditionInfo, StartOffset, VideoRendition,
        },
        EncryptionMethod, FloatOrInteger, PreloadHint,
    };

    use super::*;

    #[test]
    fn serialize_multivariant_playlist() {
        let mut output = Vec::new();

        let playlist = MultivariantPlaylist {
            is_independent_segments: true,
            start_offset: Some(StartOffset {
                offset_in_seconds: 2.0,
                is_precise: true,
            }),
            variables: vec![
                crate::DefinitionType::Inline {
                    name: "cool".into(),
                    value: "foo".into(),
                },
                crate::DefinitionType::QueryParameter {
                    name: "super_cool_actually".into(),
                },
            ],
            renditions_groups: vec![
                RenditionGroup::Video {
                    group_id: "cool_video".into(),
                    renditions: vec![VideoRendition {
                        info: RenditionInfo {
                            language: Some("en_US".into()),
                            assoc_language: Some("de".into()),
                            name: "English".into(),
                            priority: crate::RenditionPlaybackPriority::Default,
                            characteristics: vec!["private.cool.example".into()],
                            stable_rendition_id: Some("very_stable".into()),
                        },
                        uri: Some("https://example.com/video.m3u8".into()),
                    }],
                },
                RenditionGroup::ClosedCaptions {
                    group_id: "cool_captions".into(),
                    renditions: vec![ClosedCaptionRendition {
                        info: RenditionInfo {
                            language: None,
                            assoc_language: None,
                            name: "somethin".into(),
                            priority: crate::RenditionPlaybackPriority::None,
                            characteristics: vec![],
                            stable_rendition_id: None,
                        },
                        in_stream_id: crate::InStreamId::Service(NonZeroU8::new(8).unwrap()),
                    }],
                },
            ],
            variant_streams: vec![VariantStream {
                stream_info: crate::StreamInf {
                    bandwidth_bits_per_second: 8024,
                    average_bandwidth_bits_per_second: Some(8000),
                    score: Some(2.0),
                    codecs: vec!["mp4a.40.2".into(), "avc1.4d401e".into()],
                    supplemental_codecs: vec![crate::SupplementalCodec {
                        supplemental_codec: "somethin2".into(),
                        compatibility_brands: vec![],
                    }],
                    resolution: Some(crate::Resolution {
                        width: 1080,
                        height: 1920,
                    }),
                    hdcp_level: Some(crate::HdcpLevel::Type1),
                    allowed_cpc: vec![crate::ContentProtectionConfiguration {
                        key_format: "com.example.drm2".into(),
                        cpc_labels: vec![],
                    }],
                    video_range: crate::VideoRange::Pq,
                    required_video_layout: vec![crate::VideoChannelSpecifier::Stereo],
                    stable_variant_id: Some("azBY09+/=.-_".into()),
                    pathway_id: Some("cool-pathway".into()),
                },
                frame_rate: Some(60.0),
                audio_group_id: None,
                video_group_id: Some("cool_video".into()),
                subtitles_group_id: None,
                closed_captions_group_id: Some("cool_captions".into()),
                uri: "https://example.com/stuffs.m3u8".into(),
            }],
            i_frame_streams: vec![IFrameStream {
                stream_info: crate::StreamInf {
                    bandwidth_bits_per_second: 8000,
                    average_bandwidth_bits_per_second: None,
                    score: None,
                    codecs: vec![],
                    supplemental_codecs: vec![],
                    resolution: None,
                    hdcp_level: None,
                    allowed_cpc: vec![],
                    video_range: crate::VideoRange::Sdr,
                    required_video_layout: vec![],
                    stable_variant_id: None,
                    pathway_id: None,
                },
                video_group_id: Some("cool_video".into()),
                uri: "https://example.com/video2.m3u8".into(),
            }],
            session_data: vec![crate::SessionData {
                data_id: "i_am_above_you".into(),
                value: crate::SessionDataValue::Value {
                    value: "aksjfnaj".into(),
                    language: None,
                },
            }],
            session_key: vec![EncryptionMethod::SampleAes {
                uri: "https://example.com/key.key".into(),
                iv: None,
                key_format_versions: vec![],
            }],
            content_steering: vec![crate::ContentSteering {
                server_uri: "https://example.com/manifest.json".into(),
                pathway_id: Some("cool-pathway".into()),
            }],
        };

        playlist.serialize(&mut output).unwrap();

        assert_eq!(
            String::from_utf8(output).unwrap(),
            "#EXTM3U
#EXT-X-VERSION:12
#EXT-X-DEFINE:NAME=\"cool\",VALUE=\"foo\"
#EXT-X-DEFINE:QUERYPARAM=\"super_cool_actually\"
#EXT-X-INDEPENDENT-SEGMENTS
#EXT-X-START:TIME-OFFSET=2,PRECISE=YES
#EXT-X-MEDIA:TYPE=VIDEO,URI=\"https://example.com/video.m3u8\",GROUP-ID=\"cool_video\",LANGUAGE=\"en_US\",ASSOC-LANGUAGE=\"de\",NAME=\"English\",STABLE-RENDITION-ID=\"very_stable\",DEFAULT=YES,CHARACTERISTICS=\"private.cool.example\"
#EXT-X-MEDIA:TYPE=CLOSED-CAPTIONS,GROUP-ID=\"cool_captions\",NAME=\"somethin\",INSTREAM-ID=\"SERVICE8\"
#EXT-X-STREAM-INF:BANDWIDTH=8024,AVERAGE-BANDWIDTH=8000,SCORE=2,CODECS=\"mp4a.40.2,avc1.4d401e\",SUPPLEMENTAL-CODECS=\"somethin2\",RESOLUTION=1080x1920,HDCP-LEVEL=TYPE-1,ALLOWED-CPC=\"com.example.drm2:\",VIDEO-RANGE=PQ,REQ-VIDEO-LAYOUT=\"CH-STEREO\",STABLE-VARIANT-ID=\"azBY09+/=.-_\",PATHWAY-ID=\"cool-pathway\",FRAME-RATE=60.000,VIDEO=\"cool_video\",CLOSED-CAPTIONS=\"cool_captions\"
https://example.com/stuffs.m3u8
#EXT-X-I-FRAME-STREAM-INF:BANDWIDTH=8000,VIDEO=\"cool_video\",URI=\"https://example.com/video2.m3u8\"
#EXT-X-SESSION-DATA:DATA-ID=\"i_am_above_you\",VALUE=\"aksjfnaj\"
#EXT-X-SESSION-KEY:METHOD=SAMPLE-AES,URI=\"https://example.com/key.key\"
#EXT-X-CONTENT-STEERING:SERVER-URI=\"https://example.com/manifest.json\",PATHWAY-ID=\"cool-pathway\"
"
        );
    }

    #[test]
    fn serialize_media_playlist() {
        let mut output = Vec::new();

        let playlist = MediaPlaylist {
            segments: vec![
                MediaSegment {
                    uri: "https://example.com/1.mp4".into(),
                    duration_seconds: FloatOrInteger::Float(5.045),
                    title: "This is the first thingy!".into(),
                    byte_range_or_bitrate: Some(ByteRangeOrBitrate::Bitrate(8000)),
                    is_discontinuity: false,
                    encryption: Some(EncryptionMethod::Aes128 {
                        uri: "https://example.com/key.key".into(),
                        iv: Some(0x0F91_DC05),
                        key_format: crate::KeyFormat::Identity,
                        key_format_versions: vec![1, 7, 6],
                    }),
                    media_initialization_section: Some(MediaInitializationSection {
                        uri: "https://example.com/1.mp4".into(),
                        range: Some(crate::ByteRangeWithOffset {
                            length_bytes: 400,
                            start_offset_bytes: 0,
                        }),
                    }),
                    absolute_time: Some(
                        chrono::DateTime::parse_from_rfc3339("2010-02-19T14:54:23.031+08:00")
                            .unwrap(),
                    ),
                    is_gap: false,
                    parts: vec![
                        PartialSegment {
                            uri: "https://example.com/1.mp4".into(),
                            duration_in_seconds: 5.045 / 2.0,
                            is_independent: true,
                            byte_range: Some(crate::ByteRange {
                                length_bytes: 400,
                                start_offset_bytes: None,
                            }),
                            is_gap: false,
                        },
                        PartialSegment {
                            uri: "https://example.com/1.mp4".into(),
                            duration_in_seconds: 5.045 / 2.0,
                            is_independent: false,
                            byte_range: Some(crate::ByteRange {
                                length_bytes: 400,
                                start_offset_bytes: Some(400),
                            }),
                            is_gap: false,
                        },
                    ],
                },
                MediaSegment {
                    uri: "https://example.com/2.mp4".into(),
                    duration_seconds: FloatOrInteger::Float(5.045),
                    title: "This is the second thingy!".into(),
                    byte_range_or_bitrate: Some(ByteRangeOrBitrate::Bitrate(8000)),
                    is_discontinuity: false,
                    encryption: Some(EncryptionMethod::Aes128 {
                        uri: "https://example.com/key.key".into(),
                        iv: Some(0x0F91_DC05),
                        key_format: crate::KeyFormat::Identity,
                        key_format_versions: vec![1, 7, 6],
                    }),
                    media_initialization_section: Some(MediaInitializationSection {
                        uri: "https://example.com/1.mp4".into(),
                        range: Some(crate::ByteRangeWithOffset {
                            length_bytes: 400,
                            start_offset_bytes: 0,
                        }),
                    }),
                    absolute_time: None,
                    is_gap: false,
                    parts: vec![
                        PartialSegment {
                            uri: "https://example.com/2.mp4".into(),
                            duration_in_seconds: 5.045 / 2.0,
                            is_independent: true,
                            byte_range: Some(crate::ByteRange {
                                length_bytes: 400,
                                start_offset_bytes: None,
                            }),
                            is_gap: false,
                        },
                        PartialSegment {
                            uri: "https://example.com/2.mp4".into(),
                            duration_in_seconds: 5.045 / 2.0,
                            is_independent: false,
                            byte_range: Some(crate::ByteRange {
                                length_bytes: 400,
                                start_offset_bytes: Some(400),
                            }),
                            is_gap: false,
                        },
                    ],
                },
                MediaSegment {
                    uri: "https://example.com/3.mp4".into(),
                    duration_seconds: FloatOrInteger::Float(5.045),
                    title: String::new(),
                    byte_range_or_bitrate: Some(ByteRangeOrBitrate::Bitrate(5000)),
                    is_discontinuity: false,
                    encryption: None,
                    media_initialization_section: None,
                    absolute_time: None,
                    is_gap: false,
                    parts: vec![
                        PartialSegment {
                            uri: "https://example.com/3.mp4".into(),
                            duration_in_seconds: 5.045 / 2.0,
                            is_independent: true,
                            byte_range: Some(crate::ByteRange {
                                length_bytes: 400,
                                start_offset_bytes: None,
                            }),
                            is_gap: false,
                        },
                        PartialSegment {
                            uri: "https://example.com/3.mp4".into(),
                            duration_in_seconds: 5.045 / 2.0,
                            is_independent: false,
                            byte_range: Some(crate::ByteRange {
                                length_bytes: 400,
                                start_offset_bytes: Some(400),
                            }),
                            is_gap: false,
                        },
                    ],
                },
            ],
            start_offset: Some(StartOffset {
                offset_in_seconds: 2.0,
                is_precise: false,
            }),
            variables: vec![
                crate::DefinitionType::Inline {
                    name: "cool".into(),
                    value: "foo".into(),
                },
                crate::DefinitionType::Import {
                    name: "not_cool".into(),
                },
                crate::DefinitionType::QueryParameter {
                    name: "super_cool_actually".into(),
                },
            ],
            is_independent_segments: false,
            target_duration: 5,
            first_media_sequence_number: 0,
            discontinuity_sequence_number: 12,
            finished: false,
            playlist_type: Some(crate::PlaylistType::Event),
            hold_back_seconds: None,
            part_information: Some(PartInformation {
                part_hold_back_seconds: 3.0 * 3.0,
                part_target_duration: 3.0,
            }),
            iframes_only: false,
            playlist_delta_updates_information: Some(crate::DeltaUpdateInfo {
                skip_boundary_seconds: 3.0 * 6.0,
                can_skip_dateranges: true,
            }),
            supports_blocking_playlist_reloads: true,
            metadata: MediaMetadata {
                date_ranges: vec![],
                skip: None,
                preload_hints: vec![PreloadHint {
                    hint_type: crate::PreloadHintType::Part,
                    uri: "https://example.com/4.mp4".into(),
                    start_byte_offset: 0,
                    length_in_bytes: Some(400),
                }],
                rendition_reports: vec![crate::RenditionReport {
                    uri: "/different.m3u8".into(),
                    last_sequence_number: None,
                    last_part_index: None,
                }],
            },
        };

        playlist.serialize(&mut output).unwrap();

        assert_eq!(
            String::from_utf8(output).unwrap(),
            "#EXTM3U
#EXT-X-VERSION:11
#EXT-X-DEFINE:NAME=\"cool\",VALUE=\"foo\"
#EXT-X-DEFINE:IMPORT=\"not_cool\"
#EXT-X-DEFINE:QUERYPARAM=\"super_cool_actually\"
#EXT-X-START:TIME-OFFSET=2
#EXT-X-TARGETDURATION:5
#EXT-X-DISCONTINUITY-SEQUENCE:12
#EXT-X-PLAYLIST-TYPE:EVENT
#EXT-X-PART-INF:PART-TARGET=3
#EXT-X-SERVER-CONTROL:CAN-SKIP-UNTIL=18,CAN-SKIP-DATERANGES=YES,PART-HOLD-BACK=9,CAN-BLOCK-RELOAD=YES
#EXT-X-PRELOAD-HINT:TYPE=PART,URI=\"https://example.com/4.mp4\",BYTERANGE-LENGTH=400
#EXT-X-RENDITION-REPORT:URI=\"/different.m3u8\"
#EXTINF:5.045,This is the first thingy!
#EXT-X-BITRATE:8000
#EXT-X-KEY:METHOD=AES-128,URI=\"https://example.com/key.key\",IV=0xF91DC05,KEYFORMATVERSIONS=\"1/7/6\"
#EXT-X-MAP:URI=\"https://example.com/1.mp4\",BYTERANGE=\"400@0\"
#EXT-X-PROGRAM-DATE-TIME:2010-02-19T14:54:23.031+08:00
#EXT-X-PART:URI=\"https://example.com/1.mp4\",DURATION=2.5225,INDEPENDENT=YES,BYTERANGE=\"400\"
#EXT-X-PART:URI=\"https://example.com/1.mp4\",DURATION=2.5225,BYTERANGE=\"400@400\"
https://example.com/1.mp4
#EXTINF:5.045,This is the second thingy!
#EXT-X-PART:URI=\"https://example.com/2.mp4\",DURATION=2.5225,INDEPENDENT=YES,BYTERANGE=\"400\"
#EXT-X-PART:URI=\"https://example.com/2.mp4\",DURATION=2.5225,BYTERANGE=\"400@400\"
https://example.com/2.mp4
#EXTINF:5.045
#EXT-X-BITRATE:5000
#EXT-X-KEY:METHOD=NONE
#EXT-X-PART:URI=\"https://example.com/3.mp4\",DURATION=2.5225,INDEPENDENT=YES,BYTERANGE=\"400\"
#EXT-X-PART:URI=\"https://example.com/3.mp4\",DURATION=2.5225,BYTERANGE=\"400@400\"
https://example.com/3.mp4
"
        );
    }
}
