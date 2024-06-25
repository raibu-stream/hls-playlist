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

use std::io;

use crate::{
    ByteRange, ContentSteering, DateRange, PreloadHint, RenditionPlaybackPriority, RenditionReport,
    SessionData, StreamInf,
};

use super::{MediaType, Tag};

impl Tag {
    /// Serializes the `Tag` as a extended M3U playlist tag into `output`.
    /// Guaranteed to write valid UTF-8 only.
    ///
    /// This method makes lots of small calls to write on `output`. If the implementation
    /// of write on `output` makes a syscall, like with a `TcpStream`, you should wrap it
    /// in a [`std::io::BufWriter`].
    ///
    /// # Note
    ///
    /// This method is not guaranteed to write valid M3U tags. It's your job to create
    /// valid input.
    ///
    /// # Errors
    ///
    /// May return `Err` when encountering an io error on `output`.
    pub fn serialize(&self, mut output: impl io::Write) -> io::Result<()> {
        match self {
            Self::M3u => output.write_all(b"#EXTM3U")?,
            Self::XVersion { version } => write!(output, "#EXT-X-VERSION:{version}")?,
            Self::XDefine(definition) => match definition {
                crate::DefinitionType::Inline { name, value } => {
                    write!(output, "#EXT-X-DEFINE:NAME=\"{name}\",VALUE=\"{value}\"")?;
                }
                crate::DefinitionType::Import { name } => {
                    write!(output, "#EXT-X-DEFINE:IMPORT=\"{name}\"")?;
                }
                crate::DefinitionType::QueryParameter { name } => {
                    write!(output, "#EXT-X-DEFINE:QUERYPARAM=\"{name}\"")?;
                }
            },
            Self::XStart {
                offset_seconds,
                is_precise,
            } => {
                write!(output, "#EXT-X-START:TIME-OFFSET={offset_seconds}")?;
                if *is_precise {
                    write!(output, ",PRECISE=YES")?;
                }
            }
            Self::XIndependentSegments => output.write_all(b"#EXT-X-INDEPENDENT-SEGMENTS")?,
            Self::Inf {
                duration_seconds,
                title,
            } => {
                match duration_seconds {
                    crate::FloatOrInteger::Float(float) => write!(output, "#EXTINF:{float}")?,
                    crate::FloatOrInteger::Integer(integer) => {
                        write!(output, "#EXTINF:{integer}")?;
                    }
                };
                if !title.is_empty() {
                    write!(output, ",{title}")?;
                }
            }
            Self::XByterange(byte_range) => {
                write!(output, "#EXT-X-BYTERANGE:")?;
                byte_range.serialize(&mut output)?;
            }
            Self::XDiscontinuity => write!(output, "#EXT-X-DISCONTINUITY")?,
            Self::XKey(method) => {
                write!(output, "#EXT-X-KEY:")?;

                if let Some(method) = method {
                    method.serialize(&mut output)?;
                } else {
                    write!(output, "METHOD=NONE")?;
                }
            }
            Self::XMap { uri, range } => {
                write!(output, "#EXT-X-MAP:URI=\"{uri}\"")?;
                if let Some(range) = range {
                    write!(output, ",BYTERANGE=\"")?;
                    range.serialize(&mut output)?;
                    write!(output, "\"")?;
                }
            }
            Self::XProgramDateTime(time) => {
                write!(output, "#EXT-X-PROGRAM-DATE-TIME:{}", time.to_rfc3339())?;
            }
            Self::XGap => write!(output, "#EXT-X-GAP")?,
            Self::XBitrate { kbps } => write!(output, "#EXT-X-BITRATE:{kbps}")?,
            Self::XPart {
                uri,
                duration_seconds,
                is_independent,
                byte_range,
                is_gap,
            } => Self::serialize_x_part(
                &mut output,
                uri,
                *duration_seconds,
                *is_independent,
                byte_range,
                *is_gap,
            )?,
            Self::XTargetDuration {
                target_duration_seconds,
            } => write!(output, "#EXT-X-TARGETDURATION:{target_duration_seconds}")?,
            Self::XMediaSequence { sequence_number } => {
                write!(output, "#EXT-X-MEDIA-SEQUENCE:{sequence_number}")?;
            }
            Self::XDiscontinuitySequence { sequence_number } => {
                write!(output, "#EXT-X-DISCONTINUITY-SEQUENCE:{sequence_number}")?;
            }
            Self::XEndList => write!(output, "#EXT-X-ENDLIST")?,
            Self::XPlaylistType(playlist_type) => match playlist_type {
                crate::PlaylistType::Event => write!(output, "#EXT-X-PLAYLIST-TYPE:EVENT")?,
                crate::PlaylistType::Vod => write!(output, "#EXT-X-PLAYLIST-TYPE:VOD")?,
            },
            Self::XIFramesOnly => write!(output, "#EXT-X-I-FRAMES-ONLY")?,
            Self::XPartInf {
                part_target_duration_seconds,
            } => write!(
                output,
                "#EXT-X-PART-INF:PART-TARGET={part_target_duration_seconds}"
            )?,
            Self::XServerControl {
                delta_update_info,
                hold_back,
                part_hold_back,
                can_block_reload,
            } => Self::serialize_x_server_control(
                &mut output,
                delta_update_info,
                hold_back,
                part_hold_back,
                *can_block_reload,
            )?,
            Self::XMedia {
                media_type,
                group_id,
                language,
                assoc_language,
                name,
                stable_rendition_id,
                playback_priority,
                characteristics,
            } => Self::serialize_x_media(
                &mut output,
                media_type,
                group_id,
                language,
                assoc_language,
                name,
                stable_rendition_id,
                playback_priority,
                characteristics,
            )?,
            Self::XStreamInf {
                stream_inf,
                frame_rate,
                audio_group_id,
                video_group_id,
                subtitles_group_id,
                closed_captions_group_id,
                uri,
            } => Self::serialize_x_stream_inf(
                &mut output,
                stream_inf,
                frame_rate,
                audio_group_id,
                video_group_id,
                subtitles_group_id,
                closed_captions_group_id,
                uri,
            )?,
            Self::XIFrameStreamInf {
                stream_inf,
                video_group_id,
                uri,
            } => {
                Self::serialize_x_i_frame_stream_inf(&mut output, stream_inf, video_group_id, uri)?;
            }
            Self::XSessionData(session_data) => {
                Self::serialize_x_session_data(&mut output, session_data)?;
            }
            Self::XSessionKey(encryption_method) => {
                write!(output, "#EXT-X-SESSION-KEY:")?;
                encryption_method.serialize(&mut output)?;
            }
            Self::XContentSteering(content_steering) => {
                Self::serialize_x_content_steering(&mut output, content_steering)?;
            }
            Self::XDateRange(daterange) => Self::serialize_x_daterange(&mut output, daterange)?,
            Self::XSkip {
                number_of_skipped_segments,
                recently_removed_dataranges,
            } => Self::serialize_x_skip(
                &mut output,
                *number_of_skipped_segments,
                recently_removed_dataranges,
            )?,
            Self::XPreloadHint(preload_hint) => {
                Self::serialize_x_preload_hint(&mut output, preload_hint)?;
            }
            Self::XRenditionReport(report) => {
                Self::serialize_x_rendition_report(&mut output, report)?;
            }
        };

        output.write_all(b"\n")?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn serialize_x_media(
        mut output: impl io::Write,
        media_type: &MediaType,
        group_id: &String,
        language: &Option<String>,
        assoc_language: &Option<String>,
        name: &String,
        stable_rendition_id: &Option<String>,
        playback_priority: &RenditionPlaybackPriority,
        characteristics: &[String],
    ) -> io::Result<()> {
        match media_type {
            MediaType::Audio { uri, .. } => {
                write!(output, "#EXT-X-MEDIA:TYPE=AUDIO")?;

                if let Some(uri) = uri {
                    write!(output, ",URI=\"{uri}\"")?;
                }
            }
            MediaType::Video { uri } => {
                write!(output, "#EXT-X-MEDIA:TYPE=VIDEO")?;

                if let Some(uri) = uri {
                    write!(output, ",URI=\"{uri}\"")?;
                }
            }
            MediaType::Subtitles { uri, .. } => {
                write!(output, "#EXT-X-MEDIA:TYPE=SUBTITLES,URI=\"{uri}\"")?;
            }
            MediaType::ClosedCaptions { .. } => {
                write!(output, "#EXT-X-MEDIA:TYPE=CLOSED-CAPTIONS")?;
            }
        };

        write!(output, ",GROUP-ID=\"{group_id}\"")?;

        if let Some(language) = language {
            write!(output, ",LANGUAGE=\"{language}\"")?;
        }

        if let Some(assoc_language) = assoc_language {
            write!(output, ",ASSOC-LANGUAGE=\"{assoc_language}\"")?;
        }

        write!(output, ",NAME=\"{name}\"")?;

        if let Some(id) = stable_rendition_id {
            write!(output, ",STABLE-RENDITION-ID=\"{id}\"")?;
        }

        match playback_priority {
            RenditionPlaybackPriority::Default => write!(output, ",DEFAULT=YES,AUTOSELECT=YES")?,
            RenditionPlaybackPriority::AutoSelect => write!(output, ",AUTOSELECT=YES")?,
            RenditionPlaybackPriority::None => (),
        }

        if let MediaType::Subtitles { forced: true, .. } = media_type {
            write!(output, ",FORCED=YES")?;
        }

        if let MediaType::ClosedCaptions { in_stream_id } = media_type {
            match in_stream_id {
                crate::InStreamId::Cc1 => write!(output, ",INSTREAM-ID=\"CC1\"")?,
                crate::InStreamId::Cc2 => write!(output, ",INSTREAM-ID=\"CC2\"")?,
                crate::InStreamId::Cc3 => write!(output, ",INSTREAM-ID=\"CC3\"")?,
                crate::InStreamId::Cc4 => write!(output, ",INSTREAM-ID=\"CC4\"")?,
                crate::InStreamId::Service(id) => write!(output, ",INSTREAM-ID=\"SERVICE{id}\"")?,
            }
        }

        if let MediaType::Audio {
            bit_depth,
            sample_rate,
            ..
        } = media_type
        {
            if let Some(bit_depth) = bit_depth {
                write!(output, ",BIT-DEPTH={bit_depth}")?;
            }

            if let Some(sample_rate) = sample_rate {
                write!(output, ",SAMPLE-RATE={sample_rate}")?;
            }
        }

        if !characteristics.is_empty() {
            write!(output, ",CHARACTERISTICS=\"")?;

            if characteristics.len() == 1 {
                write!(output, "{}", characteristics[0])?;
            } else {
                for (i, tag) in characteristics.iter().enumerate() {
                    if i == characteristics.len() - 1 {
                        write!(output, "{tag}")?;
                    } else {
                        write!(output, "{tag},")?;
                    }
                }
            }

            write!(output, "\"")?;
        }

        if let MediaType::Audio {
            channels: Some(channels),
            ..
        } = media_type
        {
            match channels {
                crate::AudioChannelInformation::NumberOfChannelsOnly { number_of_channels } => {
                    write!(output, ",CHANNELS=\"{number_of_channels}\"")?;
                }
                crate::AudioChannelInformation::WithAudioCodingIdentifiers {
                    number_of_channels,
                    audio_coding_identifiers,
                } => {
                    write!(output, ",CHANNELS=\"{number_of_channels}/")?;

                    if audio_coding_identifiers.is_empty() {
                        write!(output, "-\"")?;
                    } else if audio_coding_identifiers.len() == 1 {
                        write!(output, "{}\"", audio_coding_identifiers[0])?;
                    } else {
                        for (i, identifier) in audio_coding_identifiers.iter().enumerate() {
                            if i == audio_coding_identifiers.len() - 1 {
                                write!(output, "{identifier}\"")?;
                            } else {
                                write!(output, "{identifier},")?;
                            }
                        }
                    }
                }
                crate::AudioChannelInformation::WithSpecialUsageIdentifiers {
                    number_of_channels,
                    audio_coding_identifiers,
                    binaural,
                    immersive,
                    downmix,
                } => {
                    write!(output, ",CHANNELS=\"{number_of_channels}/")?;

                    if audio_coding_identifiers.is_empty() {
                        write!(output, "-/")?;
                    } else if audio_coding_identifiers.len() == 1 {
                        write!(output, "{}/", audio_coding_identifiers[0])?;
                    } else {
                        for (i, identifier) in audio_coding_identifiers.iter().enumerate() {
                            if i == audio_coding_identifiers.len() - 1 {
                                write!(output, "{identifier}/")?;
                            } else {
                                write!(output, "{identifier},")?;
                            }
                        }
                    };

                    if *binaural {
                        if *immersive || *downmix {
                            write!(output, "BINAURAL,")?;
                        } else {
                            write!(output, "BINAURAL")?;
                        }
                    }
                    if *immersive {
                        if *downmix {
                            write!(output, "IMMERSIVE,")?;
                        } else {
                            write!(output, "IMMERSIVE")?;
                        }
                    }
                    if *downmix {
                        write!(output, "DOWNMIX")?;
                    }

                    write!(output, "\"")?;
                }
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn serialize_x_stream_inf(
        mut output: impl io::Write,
        stream_inf: &StreamInf,
        frame_rate: &Option<f64>,
        audio_group_id: &Option<String>,
        video_group_id: &Option<String>,
        subtitles_group_id: &Option<String>,
        closed_captions_group_id: &Option<String>,
        uri: &String,
    ) -> io::Result<()> {
        write!(output, "#EXT-X-STREAM-INF:")?;
        stream_inf.serialize(&mut output)?;

        if let Some(frame_rate) = frame_rate {
            write!(output, ",FRAME-RATE={frame_rate:.3}")?;
        }

        if let Some(id) = audio_group_id {
            write!(output, ",AUDIO=\"{id}\"")?;
        }
        if let Some(id) = video_group_id {
            write!(output, ",VIDEO=\"{id}\"")?;
        }
        if let Some(id) = subtitles_group_id {
            write!(output, ",SUBTITLES=\"{id}\"")?;
        }
        if let Some(id) = closed_captions_group_id {
            write!(output, ",CLOSED-CAPTIONS=\"{id}\"")?;
        }

        write!(output, "\n{uri}")?;

        Ok(())
    }

    fn serialize_x_i_frame_stream_inf(
        mut output: impl io::Write,
        stream_inf: &StreamInf,
        video_group_id: &Option<String>,
        uri: &String,
    ) -> io::Result<()> {
        write!(output, "#EXT-X-I-FRAME-STREAM-INF:")?;
        stream_inf.serialize(&mut output)?;

        if let Some(id) = video_group_id {
            write!(output, ",VIDEO=\"{id}\"")?;
        }

        write!(output, ",URI=\"{uri}\"")?;

        Ok(())
    }

    fn serialize_x_session_data(
        mut output: impl io::Write,
        session_data: &SessionData,
    ) -> io::Result<()> {
        write!(
            output,
            "#EXT-X-SESSION-DATA:DATA-ID=\"{}\"",
            session_data.data_id
        )?;

        match &session_data.value {
            crate::SessionDataValue::Value { value, language } => {
                write!(output, ",VALUE=\"{value}\"")?;

                if let Some(language) = language {
                    write!(output, ",LANGUAGE=\"{language}\"")?;
                }
            }
            crate::SessionDataValue::Uri { uri, format } => {
                write!(output, ",URI=\"{uri}\"")?;

                match format {
                    crate::UriFormat::Json => write!(output, ",FORMAT=JSON")?,
                    crate::UriFormat::Raw => write!(output, ",FORMAT=RAW")?,
                }
            }
        }

        Ok(())
    }

    fn serialize_x_content_steering(
        mut output: impl io::Write,
        content_steering: &ContentSteering,
    ) -> io::Result<()> {
        write!(
            output,
            "#EXT-X-CONTENT-STEERING:SERVER-URI=\"{}\"",
            content_steering.server_uri
        )?;

        if let Some(id) = &content_steering.pathway_id {
            write!(output, ",PATHWAY-ID=\"{id}\"")?;
        }

        Ok(())
    }

    fn serialize_x_rendition_report(
        mut output: impl io::Write,
        report: &RenditionReport,
    ) -> io::Result<()> {
        let uri = &report.uri;
        write!(output, "#EXT-X-RENDITION-REPORT:URI=\"{uri}\"")?;

        if let Some(last_sequence_number) = report.last_sequence_number {
            write!(output, ",LAST-MSN={last_sequence_number}")?;
        }

        if let Some(last_part_index) = report.last_part_index {
            write!(output, ",LAST-PART={last_part_index}")?;
        }

        Ok(())
    }

    fn serialize_x_preload_hint(
        mut output: impl io::Write,
        preload_hint: &PreloadHint,
    ) -> io::Result<()> {
        let hint_type = match preload_hint.hint_type {
            crate::PreloadHintType::Part => "PART",
            crate::PreloadHintType::Map => "MAP",
        };
        write!(
            output,
            "#EXT-X-PRELOAD-HINT:TYPE={hint_type},URI=\"{}\"",
            preload_hint.uri
        )?;

        if preload_hint.start_byte_offset != 0 {
            write!(
                output,
                ",BYTERANGE-START={}",
                preload_hint.start_byte_offset
            )?;
        }

        if let Some(length) = preload_hint.length_in_bytes {
            write!(output, ",BYTERANGE-LENGTH={length}")?;
        }

        Ok(())
    }

    fn serialize_x_skip(
        mut output: impl io::Write,
        number_of_skipped_segments: u64,
        recently_removed_dataranges: &[String],
    ) -> io::Result<()> {
        write!(
            output,
            "#EXT-X-SKIP:SKIPPED-SEGMENTS={number_of_skipped_segments}"
        )?;

        if !recently_removed_dataranges.is_empty() {
            write!(output, ",RECENTLY-REMOVED-DATERANGES=\"")?;

            if recently_removed_dataranges.len() == 1 {
                write!(output, "{}", recently_removed_dataranges[0])?;
            } else {
                for (i, id) in recently_removed_dataranges.iter().enumerate() {
                    if i == recently_removed_dataranges.len() - 1 {
                        write!(output, "{id}")?;
                    } else {
                        write!(output, "{id}\t")?;
                    }
                }
            }

            write!(output, "\"")?;
        }

        Ok(())
    }

    fn serialize_x_daterange(mut output: impl io::Write, daterange: &DateRange) -> io::Result<()> {
        write!(output, "#EXT-X-DATERANGE:ID=\"{}\"", daterange.id)?;

        if let Some(class) = &daterange.class {
            write!(output, ",CLASS=\"{class}\"")?;
        }

        write!(
            output,
            ",START-DATE=\"{}\"",
            daterange.start_date.to_rfc3339()
        )?;

        if let Some(cue) = &daterange.cue {
            match cue.position {
                crate::DateRangeCuePosition::Pre if cue.once => {
                    write!(output, ",CUE=\"PRE,ONCE\"")?;
                }
                crate::DateRangeCuePosition::Post if cue.once => {
                    write!(output, ",CUE=\"POST,ONCE\"")?;
                }
                crate::DateRangeCuePosition::Neither if cue.once => {
                    write!(output, ",CUE=\"ONCE\"")?;
                }
                crate::DateRangeCuePosition::Pre => {
                    write!(output, ",CUE=\"PRE\"")?;
                }
                crate::DateRangeCuePosition::Post => {
                    write!(output, ",CUE=\"POST\"")?;
                }
                crate::DateRangeCuePosition::Neither => write!(output, ",CUE=\"\"")?,
            }
        }

        if let Some(end_date) = daterange.end_date {
            write!(output, ",END-DATE=\"{}\"", end_date.to_rfc3339())?;
        }

        if let Some(duration) = daterange.duration_seconds {
            write!(output, ",DURATION={duration}")?;
        }

        if let Some(planned_duration) = daterange.planned_duration_seconds {
            write!(output, ",PLANNED-DURATION={planned_duration}")?;
        }

        for (attribute_name, attribute_value) in &daterange.client_attributes {
            write!(output, ",X-{attribute_name}=")?;
            match attribute_value {
                crate::AttributeValue::String(string) => write!(output, "\"{string}\"")?,
                crate::AttributeValue::Bytes(bytes) => {
                    write!(output, "0x{}", hex::encode_upper(bytes.clone()))?;
                }
                crate::AttributeValue::Float(float) => write!(output, "{float}")?,
            };
        }

        if !daterange.scte35_cmd.is_empty() {
            write!(
                output,
                ",SCTE35-CMD=0x{}",
                hex::encode_upper(daterange.scte35_cmd.clone())
            )?;
        }

        if !daterange.scte35_out.is_empty() {
            write!(
                output,
                ",SCTE35-OUT=0x{}",
                hex::encode_upper(daterange.scte35_out.clone())
            )?;
        }

        if !daterange.scte35_in.is_empty() {
            write!(
                output,
                ",SCTE35-IN=0x{}",
                hex::encode_upper(daterange.scte35_in.clone())
            )?;
        }

        if daterange.end_on_next {
            write!(output, ",END-ON-NEXT=YES")?;
        }

        Ok(())
    }

    fn serialize_x_part(
        mut output: impl io::Write,
        uri: &String,
        duration_seconds: f64,
        is_independent: bool,
        byte_range: &Option<ByteRange>,
        is_gap: bool,
    ) -> io::Result<()> {
        write!(
            output,
            "#EXT-X-PART:URI=\"{uri}\",DURATION={duration_seconds}"
        )?;

        if is_independent {
            write!(output, ",INDEPENDENT=YES")?;
        }

        if let Some(byte_range) = byte_range {
            write!(output, ",BYTERANGE=\"")?;
            byte_range.serialize(&mut output)?;
            write!(output, "\"")?;
        }

        if is_gap {
            write!(output, ",GAP=YES")?;
        }

        Ok(())
    }

    fn serialize_x_server_control(
        mut output: impl io::Write,
        delta_update_info: &Option<crate::DeltaUpdateInfo>,
        hold_back: &Option<f64>,
        part_hold_back: &Option<f64>,
        can_block_reload: bool,
    ) -> io::Result<()> {
        let mut has_written_attribute = false;
        write!(output, "#EXT-X-SERVER-CONTROL:")?;

        if let Some(delta_update_info) = delta_update_info {
            has_written_attribute = true;
            write!(
                output,
                "CAN-SKIP-UNTIL={}",
                delta_update_info.skip_boundary_seconds
            )?;

            if delta_update_info.can_skip_dateranges {
                write!(output, ",CAN-SKIP-DATERANGES=YES")?;
            }
        }

        if let Some(hold_back) = hold_back {
            if has_written_attribute {
                write!(output, ",HOLD-BACK={hold_back}")?;
            } else {
                write!(output, "HOLD-BACK={hold_back}")?;
            }

            has_written_attribute = true;
        }

        if let Some(part_hold_back) = part_hold_back {
            if has_written_attribute {
                write!(output, ",PART-HOLD-BACK={part_hold_back}")?;
            } else {
                write!(output, "PART-HOLD-BACK={part_hold_back}")?;
            }

            has_written_attribute = true;
        }

        if can_block_reload {
            if has_written_attribute {
                write!(output, ",CAN-BLOCK-RELOAD=YES")?;
            } else {
                write!(output, "CAN-BLOCK-RELOAD=YES")?;
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use rstest::*;

    use crate::{
        tags::MediaType, ContentProtectionConfiguration, EncryptionMethod,
        RenditionPlaybackPriority, SupplementalCodec, VideoChannelSpecifier,
    };

    use super::*;

    #[fixture]
    pub fn output() -> Vec<u8> {
        Vec::new()
    }

    #[rstest]
    fn serialize_m3u(mut output: Vec<u8>) {
        Tag::M3u.serialize(&mut output).unwrap();
        assert_eq!(output, b"#EXTM3U\n");
    }

    #[rstest]
    fn serialize_x_version(mut output: Vec<u8>) {
        Tag::XVersion { version: 12 }
            .serialize(&mut output)
            .unwrap();
        assert_eq!(output, b"#EXT-X-VERSION:12\n");
    }

    #[rstest]
    fn serialize_x_independent_segments(mut output: Vec<u8>) {
        Tag::XIndependentSegments.serialize(&mut output).unwrap();
        assert_eq!(output, b"#EXT-X-INDEPENDENT-SEGMENTS\n");
    }

    #[rstest]
    fn serialize_x_start(mut output: Vec<u8>) {
        Tag::XStart {
            offset_seconds: -84.0,
            is_precise: false,
        }
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-START:TIME-OFFSET=-84\n");

        output.clear();
        Tag::XStart {
            offset_seconds: 5.0053,
            is_precise: true,
        }
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-START:TIME-OFFSET=5.0053,PRECISE=YES\n");
    }

    #[rstest]
    fn serialize_x_define(mut output: Vec<u8>) {
        Tag::XDefine(crate::DefinitionType::Inline {
            name: "cool-param_A0".into(),
            value: "I am so cool".into(),
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(
            output,
            b"#EXT-X-DEFINE:NAME=\"cool-param_A0\",VALUE=\"I am so cool\"\n"
        );

        output.clear();
        Tag::XDefine(crate::DefinitionType::Import {
            name: "foobar-_A0".into(),
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-DEFINE:IMPORT=\"foobar-_A0\"\n");

        output.clear();
        Tag::XDefine(crate::DefinitionType::QueryParameter {
            name: "bAz-_42".into(),
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-DEFINE:QUERYPARAM=\"bAz-_42\"\n");
    }

    #[allow(clippy::unreadable_literal)]
    #[rstest]
    fn serialize_x_target_duration(mut output: Vec<u8>) {
        Tag::XTargetDuration {
            target_duration_seconds: 6942042,
        }
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-TARGETDURATION:6942042\n");
    }

    #[rstest]
    fn serialize_x_media_sequence(mut output: Vec<u8>) {
        Tag::XMediaSequence {
            sequence_number: 42,
        }
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-MEDIA-SEQUENCE:42\n");
    }

    #[rstest]
    fn serialize_x_discontinuity_sequence(mut output: Vec<u8>) {
        Tag::XDiscontinuitySequence {
            sequence_number: 420,
        }
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-DISCONTINUITY-SEQUENCE:420\n");
    }

    #[rstest]
    fn serialize_x_endlist(mut output: Vec<u8>) {
        Tag::XEndList.serialize(&mut output).unwrap();
        assert_eq!(output, b"#EXT-X-ENDLIST\n");
    }

    #[rstest]
    fn serialize_x_playlist_type(mut output: Vec<u8>) {
        Tag::XPlaylistType(crate::PlaylistType::Event)
            .serialize(&mut output)
            .unwrap();
        assert_eq!(output, b"#EXT-X-PLAYLIST-TYPE:EVENT\n");

        output.clear();
        Tag::XPlaylistType(crate::PlaylistType::Vod)
            .serialize(&mut output)
            .unwrap();
        assert_eq!(output, b"#EXT-X-PLAYLIST-TYPE:VOD\n");
    }

    #[rstest]
    fn serialize_x_i_frames_only(mut output: Vec<u8>) {
        Tag::XIFramesOnly.serialize(&mut output).unwrap();
        assert_eq!(output, b"#EXT-X-I-FRAMES-ONLY\n");
    }

    #[rstest]
    fn serialize_x_part_inf(mut output: Vec<u8>) {
        Tag::XPartInf {
            part_target_duration_seconds: 2.5,
        }
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-PART-INF:PART-TARGET=2.5\n");
    }

    #[rstest]
    fn serialize_x_server_control(mut output: Vec<u8>) {
        Tag::XServerControl {
            delta_update_info: Some(crate::DeltaUpdateInfo {
                skip_boundary_seconds: 20.873,
                can_skip_dateranges: true,
            }),
            hold_back: Some(10.0),
            part_hold_back: Some(10.285),
            can_block_reload: true,
        }
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-SERVER-CONTROL:CAN-SKIP-UNTIL=20.873,CAN-SKIP-DATERANGES=YES,HOLD-BACK=10,PART-HOLD-BACK=10.285,CAN-BLOCK-RELOAD=YES\n");

        output.clear();
        Tag::XServerControl {
            delta_update_info: Some(crate::DeltaUpdateInfo {
                skip_boundary_seconds: 20.873,
                can_skip_dateranges: false,
            }),
            hold_back: Some(10.0),
            part_hold_back: Some(10.285),
            can_block_reload: true,
        }
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-SERVER-CONTROL:CAN-SKIP-UNTIL=20.873,HOLD-BACK=10,PART-HOLD-BACK=10.285,CAN-BLOCK-RELOAD=YES\n");

        output.clear();
        Tag::XServerControl {
            delta_update_info: None,
            hold_back: None,
            part_hold_back: None,
            can_block_reload: false,
        }
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-SERVER-CONTROL:\n");
    }

    #[rstest]
    fn serialize_inf(mut output: Vec<u8>) {
        Tag::Inf {
            duration_seconds: crate::FloatOrInteger::Float(5.340),
            title: String::new(),
        }
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXTINF:5.34\n");

        output.clear();
        Tag::Inf {
            duration_seconds: crate::FloatOrInteger::Integer(5),
            title: "super cool title".into(),
        }
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXTINF:5,super cool title\n");
    }

    #[rstest]
    fn serialize_x_byterange(mut output: Vec<u8>) {
        Tag::XByterange(crate::ByteRange {
            length_bytes: 1200,
            start_offset_bytes: None,
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-BYTERANGE:1200\n");

        output.clear();
        Tag::XByterange(crate::ByteRange {
            length_bytes: 1200,
            start_offset_bytes: Some(158),
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-BYTERANGE:1200@158\n");
    }

    #[rstest]
    fn serialize_x_discontinuity(mut output: Vec<u8>) {
        Tag::XDiscontinuity.serialize(&mut output).unwrap();
        assert_eq!(output, b"#EXT-X-DISCONTINUITY\n");
    }

    #[rstest]
    fn serialize_x_key(mut output: Vec<u8>) {
        Tag::XKey(None).serialize(&mut output).unwrap();
        assert_eq!(output, b"#EXT-X-KEY:METHOD=NONE\n");

        output.clear();
        Tag::XKey(Some(EncryptionMethod::Aes128 {
            uri: "https://example.com/foo.key".into(),
            iv: Some(0x0F91_DC05),
            key_format: crate::KeyFormat::Other("super cool key format".into()),
            key_format_versions: vec![1, 16],
        }))
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-KEY:METHOD=AES-128,URI=\"https://example.com/foo.key\",IV=0xF91DC05,KEYFORMAT=\"super cool key format\",KEYFORMATVERSIONS=\"1/16\"\n");

        output.clear();
        Tag::XKey(Some(EncryptionMethod::Aes128 {
            uri: "https://example.com/foo.key".into(),
            iv: None,
            key_format: crate::KeyFormat::Identity,
            key_format_versions: vec![],
        }))
        .serialize(&mut output)
        .unwrap();
        assert_eq!(
            output,
            b"#EXT-X-KEY:METHOD=AES-128,URI=\"https://example.com/foo.key\"\n"
        );

        output.clear();
        Tag::XKey(Some(EncryptionMethod::SampleAes {
            uri: "https://example.com/foo.key".into(),
            iv: Some(0x0F91_DC05),
            key_format_versions: vec![1, 16],
        }))
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-KEY:METHOD=SAMPLE-AES,URI=\"https://example.com/foo.key\",IV=0xF91DC05,KEYFORMATVERSIONS=\"1/16\"\n");

        output.clear();
        Tag::XKey(Some(EncryptionMethod::SampleAesCtr {
            uri: "https://example.com/foo.key".into(),
            key_format_versions: vec![1, 16],
        }))
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-KEY:METHOD=SAMPLE-AES-CTR,URI=\"https://example.com/foo.key\",KEYFORMATVERSIONS=\"1/16\"\n");
    }

    #[rstest]
    fn serialize_x_map(mut output: Vec<u8>) {
        Tag::XMap {
            uri: "https://example.com/0.mp4".into(),
            range: Some(crate::ByteRangeWithOffset {
                length_bytes: 400,
                start_offset_bytes: 0,
            }),
        }
        .serialize(&mut output)
        .unwrap();
        assert_eq!(
            output,
            b"#EXT-X-MAP:URI=\"https://example.com/0.mp4\",BYTERANGE=\"400@0\"\n"
        );

        output.clear();
        Tag::XMap {
            uri: "https://example.com/0.mp4".into(),
            range: None,
        }
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-MAP:URI=\"https://example.com/0.mp4\"\n");
    }

    #[rstest]
    fn serialize_x_program_date_time(mut output: Vec<u8>) {
        let time = chrono::DateTime::parse_from_rfc3339("2010-02-19T14:54:23.031+08:00").unwrap();
        Tag::XProgramDateTime(time).serialize(&mut output).unwrap();
        assert_eq!(
            output,
            b"#EXT-X-PROGRAM-DATE-TIME:2010-02-19T14:54:23.031+08:00\n"
        );
    }

    #[rstest]
    fn serialize_x_gap(mut output: Vec<u8>) {
        Tag::XGap.serialize(&mut output).unwrap();
        assert_eq!(output, b"#EXT-X-GAP\n");
    }

    #[rstest]
    fn serialize_x_bitrate(mut output: Vec<u8>) {
        Tag::XBitrate { kbps: 8000 }.serialize(&mut output).unwrap();
        assert_eq!(output, b"#EXT-X-BITRATE:8000\n");
    }

    #[rstest]
    fn serialize_x_part(mut output: Vec<u8>) {
        Tag::XPart {
            uri: "https://example.com/1.mp4".into(),
            duration_seconds: 2.5,
            is_independent: true,
            byte_range: Some(crate::ByteRange {
                length_bytes: 400,
                start_offset_bytes: Some(0),
            }),
            is_gap: true,
        }
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-PART:URI=\"https://example.com/1.mp4\",DURATION=2.5,INDEPENDENT=YES,BYTERANGE=\"400@0\",GAP=YES\n");

        output.clear();
        Tag::XPart {
            uri: "https://example.com/1.mp4".into(),
            duration_seconds: 2.5,
            is_independent: false,
            byte_range: Some(crate::ByteRange {
                length_bytes: 400,
                start_offset_bytes: None,
            }),
            is_gap: false,
        }
        .serialize(&mut output)
        .unwrap();
        assert_eq!(
            output,
            b"#EXT-X-PART:URI=\"https://example.com/1.mp4\",DURATION=2.5,BYTERANGE=\"400\"\n"
        );
    }

    #[rstest]
    fn serialize_x_skip(mut output: Vec<u8>) {
        Tag::XSkip {
            number_of_skipped_segments: 42,
            recently_removed_dataranges: vec![
                "This is my favorite data range".into(),
                "I hate this one though".into(),
            ],
        }
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-SKIP:SKIPPED-SEGMENTS=42,RECENTLY-REMOVED-DATERANGES=\"This is my favorite data range\tI hate this one though\"\n");

        output.clear();
        Tag::XSkip {
            number_of_skipped_segments: 68,
            recently_removed_dataranges: vec![],
        }
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-SKIP:SKIPPED-SEGMENTS=68\n");
    }

    #[rstest]
    fn serialize_x_preload_hint(mut output: Vec<u8>) {
        Tag::XPreloadHint(crate::PreloadHint {
            hint_type: crate::PreloadHintType::Part,
            uri: "https://example.com/1.mp4".into(),
            start_byte_offset: 400,
            length_in_bytes: Some(400),
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-PRELOAD-HINT:TYPE=PART,URI=\"https://example.com/1.mp4\",BYTERANGE-START=400,BYTERANGE-LENGTH=400\n");

        output.clear();
        Tag::XPreloadHint(crate::PreloadHint {
            hint_type: crate::PreloadHintType::Map,
            uri: "https://example.com/0.mp4".into(),
            start_byte_offset: 0,
            length_in_bytes: None,
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(
            output,
            b"#EXT-X-PRELOAD-HINT:TYPE=MAP,URI=\"https://example.com/0.mp4\"\n"
        );
    }

    #[rstest]
    fn serialize_x_rendition_report(mut output: Vec<u8>) {
        Tag::XRenditionReport(crate::RenditionReport {
            uri: "/2.m3u8".into(),
            last_sequence_number: Some(420),
            last_part_index: Some(1),
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(
            output,
            b"#EXT-X-RENDITION-REPORT:URI=\"/2.m3u8\",LAST-MSN=420,LAST-PART=1\n"
        );

        output.clear();
        Tag::XRenditionReport(crate::RenditionReport {
            uri: "/2.m3u8".into(),
            last_sequence_number: None,
            last_part_index: None,
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-RENDITION-REPORT:URI=\"/2.m3u8\"\n");
    }

    #[rstest]
    fn serialize_x_media(mut output: Vec<u8>) {
        let mut tag = Tag::XMedia {
            media_type: MediaType::Audio {
                uri: Some("https://example.com/1.m3u8".into()),
                channels: Some(
                    crate::AudioChannelInformation::WithSpecialUsageIdentifiers {
                        number_of_channels: 2,
                        audio_coding_identifiers: vec!["idk".into(), "This is kinda weird".into()],
                        binaural: true,
                        immersive: true,
                        downmix: true,
                    },
                ),
                bit_depth: Some(16),
                sample_rate: Some(40000),
            },
            group_id: "really cool group".into(),
            language: Some("en-US".into()),
            assoc_language: Some("de".into()),
            name: "english audio".into(),
            stable_rendition_id: Some("azBY09+/=.-_".into()),
            playback_priority: crate::RenditionPlaybackPriority::Default,
            characteristics: vec![
                "public.accessibility.describes-video".into(),
                "private.cool.example".into(),
            ],
        };
        tag.serialize(&mut output).unwrap();
        assert_eq!(output, b"#EXT-X-MEDIA:TYPE=AUDIO,URI=\"https://example.com/1.m3u8\",GROUP-ID=\"really cool group\",LANGUAGE=\"en-US\",ASSOC-LANGUAGE=\"de\",NAME=\"english audio\",STABLE-RENDITION-ID=\"azBY09+/=.-_\",DEFAULT=YES,AUTOSELECT=YES,BIT-DEPTH=16,SAMPLE-RATE=40000,CHARACTERISTICS=\"public.accessibility.describes-video,private.cool.example\",CHANNELS=\"2/idk,This is kinda weird/BINAURAL,IMMERSIVE,DOWNMIX\"\n");

        output.clear();
        if let Tag::XMedia {
            media_type: MediaType::Audio { channels, .. },
            ..
        } = &mut tag
        {
            *channels = Some(
                crate::AudioChannelInformation::WithSpecialUsageIdentifiers {
                    number_of_channels: 14,
                    audio_coding_identifiers: vec!["This is kinda weird".into()],
                    binaural: false,
                    immersive: false,
                    downmix: false,
                },
            );
        };
        tag.serialize(&mut output).unwrap();
        assert_eq!(output, b"#EXT-X-MEDIA:TYPE=AUDIO,URI=\"https://example.com/1.m3u8\",GROUP-ID=\"really cool group\",LANGUAGE=\"en-US\",ASSOC-LANGUAGE=\"de\",NAME=\"english audio\",STABLE-RENDITION-ID=\"azBY09+/=.-_\",DEFAULT=YES,AUTOSELECT=YES,BIT-DEPTH=16,SAMPLE-RATE=40000,CHARACTERISTICS=\"public.accessibility.describes-video,private.cool.example\",CHANNELS=\"14/This is kinda weird/\"\n");

        output.clear();
        if let Tag::XMedia {
            media_type:
                MediaType::Audio {
                    channels,
                    uri,
                    bit_depth,
                    sample_rate,
                },
            ..
        } = &mut tag
        {
            *channels = Some(crate::AudioChannelInformation::WithAudioCodingIdentifiers {
                number_of_channels: 6,
                audio_coding_identifiers: vec![],
            });
            *uri = None;
            *bit_depth = None;
            *sample_rate = None;
        };
        tag.serialize(&mut output).unwrap();
        assert_eq!(output, b"#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"really cool group\",LANGUAGE=\"en-US\",ASSOC-LANGUAGE=\"de\",NAME=\"english audio\",STABLE-RENDITION-ID=\"azBY09+/=.-_\",DEFAULT=YES,AUTOSELECT=YES,CHARACTERISTICS=\"public.accessibility.describes-video,private.cool.example\",CHANNELS=\"6/-\"\n");

        output.clear();
        if let Tag::XMedia {
            media_type,
            language,
            assoc_language,
            stable_rendition_id,
            playback_priority,
            characteristics,
            ..
        } = &mut tag
        {
            *media_type = MediaType::ClosedCaptions {
                in_stream_id: crate::InStreamId::Cc2,
            };
            *language = None;
            *assoc_language = None;
            *stable_rendition_id = None;
            *playback_priority = RenditionPlaybackPriority::AutoSelect;
            *characteristics = vec![];
        };
        tag.serialize(&mut output).unwrap();
        assert_eq!(output, b"#EXT-X-MEDIA:TYPE=CLOSED-CAPTIONS,GROUP-ID=\"really cool group\",NAME=\"english audio\",AUTOSELECT=YES,INSTREAM-ID=\"CC2\"\n");

        output.clear();
        if let Tag::XMedia { media_type, .. } = &mut tag {
            *media_type = MediaType::Subtitles {
                uri: "whyeven.mp4".into(),
                forced: true,
            };
        };
        tag.serialize(&mut output).unwrap();
        assert_eq!(output, b"#EXT-X-MEDIA:TYPE=SUBTITLES,URI=\"whyeven.mp4\",GROUP-ID=\"really cool group\",NAME=\"english audio\",AUTOSELECT=YES,FORCED=YES\n");

        output.clear();
        if let Tag::XMedia {
            media_type,
            playback_priority,
            ..
        } = &mut tag
        {
            *media_type = MediaType::Video { uri: None };
            *playback_priority = RenditionPlaybackPriority::None;
        };
        tag.serialize(&mut output).unwrap();
        assert_eq!(
            output,
            b"#EXT-X-MEDIA:TYPE=VIDEO,GROUP-ID=\"really cool group\",NAME=\"english audio\"\n"
        );
    }

    #[rstest]
    fn serialize_x_stream_inf(mut output: Vec<u8>) {
        Tag::XStreamInf {
            stream_inf: crate::StreamInf {
                bandwidth_bits_per_second: 82006,
                average_bandwidth_bits_per_second: Some(80000),
                score: Some(2.0),
                codecs: vec!["mp4a.40.2".into(), "avc1.4d401e".into()],
                supplemental_codecs: vec![
                    SupplementalCodec {
                        supplemental_codec: "somethin".into(),
                        compatibility_brands: vec![],
                    },
                    SupplementalCodec {
                        supplemental_codec: "dvh1.08.07".into(),
                        compatibility_brands: vec!["db4h".into(), "idk".into()],
                    },
                ],
                resolution: Some(crate::Resolution {
                    width: 1080,
                    height: 1920,
                }),
                hdcp_level: Some(crate::HdcpLevel::Type1),
                allowed_cpc: vec![
                    ContentProtectionConfiguration {
                        key_format: "com.example.drm1".into(),
                        cpc_labels: vec!["SMART-TV".into(), "PC".into()],
                    },
                    ContentProtectionConfiguration {
                        key_format: "com.example.drm2".into(),
                        cpc_labels: vec![],
                    },
                ],
                video_range: crate::VideoRange::Pq,
                required_video_layout: vec![
                    VideoChannelSpecifier::Stereo,
                    VideoChannelSpecifier::Mono,
                ],
                stable_variant_id: Some("azBY09+/=.-_".into()),
                pathway_id: Some("cool-pathway".into()),
            },
            frame_rate: Some(59.94258),
            audio_group_id: Some("great-audio".into()),
            video_group_id: Some("great-video".into()),
            subtitles_group_id: Some("great-subtitles".into()),
            closed_captions_group_id: Some("great-closed-captions".into()),
            uri: "great-playlist.m3u8".into(),
        }
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-STREAM-INF:BANDWIDTH=82006,AVERAGE-BANDWIDTH=80000,SCORE=2,CODECS=\"mp4a.40.2,avc1.4d401e\",SUPPLEMENTAL-CODECS=\"somethin,dvh1.08.07/db4h/idk\",RESOLUTION=1080x1920,HDCP-LEVEL=TYPE-1,ALLOWED-CPC=\"com.example.drm1:SMART-TV/PC,com.example.drm2:\",VIDEO-RANGE=PQ,REQ-VIDEO-LAYOUT=\"CH-STEREO,CH-MONO\",STABLE-VARIANT-ID=\"azBY09+/=.-_\",PATHWAY-ID=\"cool-pathway\",FRAME-RATE=59.943,AUDIO=\"great-audio\",VIDEO=\"great-video\",SUBTITLES=\"great-subtitles\",CLOSED-CAPTIONS=\"great-closed-captions\"\ngreat-playlist.m3u8\n");

        output.clear();
        Tag::XStreamInf {
            stream_inf: crate::StreamInf {
                bandwidth_bits_per_second: 82006,
                average_bandwidth_bits_per_second: None,
                score: None,
                codecs: vec![],
                supplemental_codecs: vec![],
                resolution: None,
                hdcp_level: None,
                allowed_cpc: vec![],
                video_range: crate::VideoRange::Sdr,
                required_video_layout: vec![VideoChannelSpecifier::Mono],
                stable_variant_id: None,
                pathway_id: None,
            },
            frame_rate: None,
            audio_group_id: None,
            video_group_id: None,
            subtitles_group_id: None,
            closed_captions_group_id: None,
            uri: "great-playlist.m3u8".into(),
        }
        .serialize(&mut output)
        .unwrap();
        assert_eq!(
            output,
            b"#EXT-X-STREAM-INF:BANDWIDTH=82006\ngreat-playlist.m3u8\n"
        );
    }

    #[rstest]
    fn serialize_x_i_frame_stream_inf(mut output: Vec<u8>) {
        Tag::XIFrameStreamInf {
            stream_inf: crate::StreamInf {
                bandwidth_bits_per_second: 82006,
                average_bandwidth_bits_per_second: Some(80000),
                score: Some(2.0),
                codecs: vec!["mp4a.40.2".into(), "avc1.4d401e".into()],
                supplemental_codecs: vec![
                    SupplementalCodec {
                        supplemental_codec: "somethin".into(),
                        compatibility_brands: vec![],
                    },
                    SupplementalCodec {
                        supplemental_codec: "dvh1.08.07".into(),
                        compatibility_brands: vec!["db4h".into(), "idk".into()],
                    },
                ],
                resolution: Some(crate::Resolution {
                    width: 1080,
                    height: 1920,
                }),
                hdcp_level: Some(crate::HdcpLevel::Type1),
                allowed_cpc: vec![
                    ContentProtectionConfiguration {
                        key_format: "com.example.drm1".into(),
                        cpc_labels: vec!["SMART-TV".into(), "PC".into()],
                    },
                    ContentProtectionConfiguration {
                        key_format: "com.example.drm2".into(),
                        cpc_labels: vec![],
                    },
                ],
                video_range: crate::VideoRange::Pq,
                required_video_layout: vec![
                    VideoChannelSpecifier::Stereo,
                    VideoChannelSpecifier::Mono,
                ],
                stable_variant_id: Some("azBY09+/=.-_".into()),
                pathway_id: Some("cool-pathway".into()),
            },
            video_group_id: Some("great-video".into()),
            uri: "https://example.com/example.m3u8".into(),
        }
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-I-FRAME-STREAM-INF:BANDWIDTH=82006,AVERAGE-BANDWIDTH=80000,SCORE=2,CODECS=\"mp4a.40.2,avc1.4d401e\",SUPPLEMENTAL-CODECS=\"somethin,dvh1.08.07/db4h/idk\",RESOLUTION=1080x1920,HDCP-LEVEL=TYPE-1,ALLOWED-CPC=\"com.example.drm1:SMART-TV/PC,com.example.drm2:\",VIDEO-RANGE=PQ,REQ-VIDEO-LAYOUT=\"CH-STEREO,CH-MONO\",STABLE-VARIANT-ID=\"azBY09+/=.-_\",PATHWAY-ID=\"cool-pathway\",VIDEO=\"great-video\",URI=\"https://example.com/example.m3u8\"\n");
    }

    #[rstest]
    fn serialize_x_session_data(mut output: Vec<u8>) {
        Tag::XSessionData(crate::SessionData {
            data_id: "com.example.movie.title".into(),
            value: crate::SessionDataValue::Value {
                value: "I'm important".into(),
                language: Some("en".into()),
            },
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-SESSION-DATA:DATA-ID=\"com.example.movie.title\",VALUE=\"I'm important\",LANGUAGE=\"en\"\n");

        output.clear();
        Tag::XSessionData(crate::SessionData {
            data_id: "com.example.movie.title".into(),
            value: crate::SessionDataValue::Value {
                value: "I'm important".into(),
                language: None,
            },
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(
            output,
            b"#EXT-X-SESSION-DATA:DATA-ID=\"com.example.movie.title\",VALUE=\"I'm important\"\n"
        );

        output.clear();
        Tag::XSessionData(crate::SessionData {
            data_id: "com.example.movie.title".into(),
            value: crate::SessionDataValue::Uri {
                uri: "/important.json".into(),
                format: crate::UriFormat::Json,
            },
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-SESSION-DATA:DATA-ID=\"com.example.movie.title\",URI=\"/important.json\",FORMAT=JSON\n");

        output.clear();
        Tag::XSessionData(crate::SessionData {
            data_id: "com.example.movie.title".into(),
            value: crate::SessionDataValue::Uri {
                uri: "/important.bin".into(),
                format: crate::UriFormat::Raw,
            },
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-SESSION-DATA:DATA-ID=\"com.example.movie.title\",URI=\"/important.bin\",FORMAT=RAW\n");
    }

    #[rstest]
    fn serialize_x_session_key(mut output: Vec<u8>) {
        output.clear();
        Tag::XSessionKey(EncryptionMethod::Aes128 {
            uri: "https://example.com/foo.key".into(),
            iv: Some(0x0F91_DC05),
            key_format: crate::KeyFormat::Other("super cool key format".into()),
            key_format_versions: vec![1, 16],
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-SESSION-KEY:METHOD=AES-128,URI=\"https://example.com/foo.key\",IV=0xF91DC05,KEYFORMAT=\"super cool key format\",KEYFORMATVERSIONS=\"1/16\"\n");

        output.clear();
        Tag::XSessionKey(EncryptionMethod::Aes128 {
            uri: "https://example.com/foo.key".into(),
            iv: None,
            key_format: crate::KeyFormat::Identity,
            key_format_versions: vec![],
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(
            output,
            b"#EXT-X-SESSION-KEY:METHOD=AES-128,URI=\"https://example.com/foo.key\"\n"
        );

        output.clear();
        Tag::XSessionKey(EncryptionMethod::SampleAes {
            uri: "https://example.com/foo.key".into(),
            iv: Some(0x0F91_DC05),
            key_format_versions: vec![1, 16],
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-SESSION-KEY:METHOD=SAMPLE-AES,URI=\"https://example.com/foo.key\",IV=0xF91DC05,KEYFORMATVERSIONS=\"1/16\"\n");

        output.clear();
        Tag::XSessionKey(EncryptionMethod::SampleAesCtr {
            uri: "https://example.com/foo.key".into(),
            key_format_versions: vec![1, 16],
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-SESSION-KEY:METHOD=SAMPLE-AES-CTR,URI=\"https://example.com/foo.key\",KEYFORMATVERSIONS=\"1/16\"\n");
    }

    #[rstest]
    fn serialize_x_content_steering(mut output: Vec<u8>) {
        Tag::XContentSteering(crate::ContentSteering {
            server_uri: "https://example.com/manifest.json".into(),
            pathway_id: Some("hi".into()),
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-CONTENT-STEERING:SERVER-URI=\"https://example.com/manifest.json\",PATHWAY-ID=\"hi\"\n");

        output.clear();
        Tag::XContentSteering(crate::ContentSteering {
            server_uri: "https://example.com/manifest.json".into(),
            pathway_id: None,
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(
            output,
            b"#EXT-X-CONTENT-STEERING:SERVER-URI=\"https://example.com/manifest.json\"\n"
        );
    }
}
