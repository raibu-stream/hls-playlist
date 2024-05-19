use std::io;

use super::Tag;

impl Tag {
    /// Serializes the `Tag` as a extended M3U playlist tag into `output`.
    /// Guaranteed to write valid UTF-8 only.
    ///
    /// This method makes lots of small calls to write on `output`. If the implementation
    /// of write on `output` makes a syscall, like with a `TcpStream`, you should wrap it
    /// in a [`std::io::BufWriter`].
    ///
    /// # Errors
    ///
    /// May return `Err` when encountering an io error on `output`.
    pub fn serialize(&self, mut output: impl io::Write) -> io::Result<()> {
        match self {
            Self::XVersion { version } => write!(output, "#EXT-X-VERSION:{version}")?,
            Self::M3u => output.write_all(b"#EXTM3U")?,
            Self::XDefine(_) => todo!(),
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
            } => todo!(),
            Self::XByterange(_) => todo!(),
            Self::XDiscontinuity => todo!(),
            Self::XKey(_) => todo!(),
            Self::XMap { uri, range } => todo!(),
            Self::XProgramDateTime(_) => todo!(),
            Self::XGap => todo!(),
            Self::XBitrate { kbps } => todo!(),
            Self::XPart {
                uri,
                duration_seconds,
                is_independent,
                byte_range,
                is_gap,
            } => todo!(),
            Self::XTargetDuration {
                target_duration_seconds,
            } => todo!(),
            Self::XMediaSequence { sequence_number } => todo!(),
            Self::XDiscontinuitySequence { sequence_number } => todo!(),
            Self::XEndList => todo!(),
            Self::XPlaylistType(_) => todo!(),
            Self::XIFramesOnly => todo!(),
            Self::XPartInf {
                part_target_duration_seconds,
            } => todo!(),
            Self::XServerControl {
                delta_update_info,
                hold_back,
                part_hold_back,
                can_block_reload,
            } => todo!(),
            Self::XMedia {
                media_type,
                group_id,
                language,
                assoc_language,
                name,
                stable_rendition_id,
                playback_priority,
                characteristics,
            } => todo!(),
            Self::XStreamInf {
                stream_inf,
                frame_rate,
                audio_group_id,
                video_group_id,
                subtitles_group_id,
                closed_captions_group_id,
                uri,
            } => todo!(),
            Self::XIFrameStreamInf {
                stream_inf,
                video_group_id,
                uri,
            } => todo!(),
            Self::XSessionData(_) => todo!(),
            Self::XSessionKey(_) => todo!(),
            Self::XContentSteering(_) => todo!(),
            Self::XDateRange(_) => todo!(),
            Self::XSkip {
                number_of_skipped_segments,
                recently_removed_dataranges,
            } => todo!(),
            Self::XPreloadHint(_) => todo!(),
            Self::XRenditionReport(_) => todo!(),
        };

        output.write_all(b"\n")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, time::Duration};

    use rstest::*;

    use crate::{
        tags::MediaType, AttributeValue, ContentProtectionConfiguration, EncryptionMethod,
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
        assert_eq!(output, b"#EXTINF:5.34,\n");

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
        assert_eq!(output, b"#EXT-X-KEY:METHOD=AES-128,URI=\"https://example.com/foo.key\",IV=0x0F91DC05,KEYFORMAT=\"super cool key format\",KEYFORMATVERSIONS=\"1/16\"\n");

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
        assert_eq!(output, b"#EXT-X-KEY:METHOD=SAMPLE-AES,URI=\"https://example.com/foo.key\",IV=0x0F91DC05,KEYFORMATVERSIONS=\"1/16\"\n");

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
            b"#EXT-X-MAP:URI=\"https://example.com/0.mp4\",BYTERANGE=400@0\n"
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
        Tag::XProgramDateTime(time.into())
            .serialize(&mut output)
            .unwrap();
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
        assert_eq!(output, b"#EXT-X-PART:URI=\"https://example.com/1.mp4\",DURATION=2.5,INDEPENDENT=YES,BYTERANGE=400@0,GAP=YES\n");

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
            b"#EXT-X-PART:URI=\"https://example.com/1.mp4\",DURATION=2.5,BYTERANGE=400\n"
        );
    }

    #[rstest]
    fn serialize_x_daterange(mut output: Vec<u8>) {
        let time = chrono::DateTime::parse_from_rfc3339("2010-02-19T14:54:23.031+08:00")
            .unwrap()
            .into();
        Tag::XDateRange(crate::DateRange {
            id: "This is my favorite data range".into(),
            class: Some("private.cool.example".into()),
            start_date: time,
            cue: Some(crate::DateRangeCue {
                once: true,
                position: crate::DateRangeCuePosition::Post,
            }),
            end_date: Some(time + Duration::from_millis(500)),
            duration_seconds: Some(0.5),
            planned_duration_seconds: Some(0.52318),
            client_attributes: HashMap::from([
                (
                    "EXAMPLE-STRING".into(),
                    AttributeValue::String("I wonder what this does!".into()),
                ),
                (
                    "EXAMPLE-BYTES".into(),
                    AttributeValue::Bytes(vec![0x63, 0x8, 0x8F]),
                ),
                ("EXAMPLE-FLOAT".into(), AttributeValue::Float(0.42)),
            ]),
            scte35_cmd: Some(vec![0x98, 0xA9, 0x1A, 0xFB, 0x81, 0x20, 0x5]),
            scte35_in: Some(vec![0x98, 0xA2, 0x72, 0x4C, 0x20, 0x5]),
            scte35_out: Some(vec![0x0]),
            end_on_next: true,
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(
            output,
            b"#EXT-X-DATERANGE:ID=\"This is my favorite data range\",CLASS=\"private.cool.example\",START-DATE=\"2010-02-19T14:54:23.031+08:00\",CUE=\"ONCE,POST\",END-DATE=\"TODO\",DURATION=0.5,PLANNED-DURATION=0.52318,X-EXAMPLE-STRING=\"I wonder what this does!\",X-EXAMPLE-BYTES=0x63088F,X-EXAMPLE-FLOAT=0.42,SCTE35-CMD=0x98A91AFB812005,SCTE35-OUT=0x0,SCTE35-IN=0x98A2724C2005,END-ON-NEXT=YES\n"
        );

        output.clear();
        Tag::XDateRange(crate::DateRange {
            id: "This is my favorite data range".into(),
            class: None,
            start_date: time,
            cue: None,
            end_date: None,
            duration_seconds: Some(0.5),
            planned_duration_seconds: None,
            client_attributes: HashMap::new(),
            scte35_cmd: None,
            scte35_in: None,
            scte35_out: None,
            end_on_next: false,
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(
            output,
            b"#EXT-X-DATERANGE:ID=\"This is my favorite data range\",START-DATE=\"2010-02-19T14:54:23.031+08:00\",DURATION=0.5\n"
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
        assert_eq!(output, b"#EXT-X-SKIP:SKIPPED-SEGMENTS=42\n");
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
            uri: Some("/2.m3u8".into()),
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
            uri: None,
            last_sequence_number: None,
            last_part_index: None,
        })
        .serialize(&mut output)
        .unwrap();
        assert_eq!(output, b"#EXT-X-RENDITION-REPORT:\n");
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
                        cpc_label: vec!["SMART-TV".into(), "PC".into()],
                    },
                    ContentProtectionConfiguration {
                        key_format: "com.example.drm2".into(),
                        cpc_label: vec![],
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
        assert_eq!(output, b"#EXT-X-STREAM-INF:BANDWIDTH=82006,AVERAGE-BANDWIDTH=80000,SCORE=2,CODECS=\"mp4a.40.2,avc1.4d401e\",SUPPLEMENTAL-CODECS=\"somethin,dvh1.08.07/db4h/idk\",RESOLUTION=1080x1920,FRAME-RATE=59.942,HDCP-LEVEL=TYPE-1,ALLOWED-CPC=\"com.example.drm1:SMART-TV/PC,com.example.drm2:\",VIDEO-RANGE=PQ,REQ-VIDEO-LAYOUT=\"CH-STEREO,CH-MONO\",STABLE-VARIANT-ID=\"azBY09+/=.-_\",AUDIO=\"great-audio\",VIDEO=\"great-video\",SUBTITLES=\"great-subtitles\",CLOSED-CAPTIONS=\"great-closed-captions\",PATHWAY-ID=\"cool-pathway\"\ngreat-playlist.m3u8\n");

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
                        cpc_label: vec!["SMART-TV".into(), "PC".into()],
                    },
                    ContentProtectionConfiguration {
                        key_format: "com.example.drm2".into(),
                        cpc_label: vec![],
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
        assert_eq!(output, b"#EXT-X-STREAM-INF:BANDWIDTH=82006,AVERAGE-BANDWIDTH=80000,SCORE=2,CODECS=\"mp4a.40.2,avc1.4d401e\",SUPPLEMENTAL-CODECS=\"somethin,dvh1.08.07/db4h/idk\",RESOLUTION=1080x1920,HDCP-LEVEL=TYPE-1,ALLOWED-CPC=\"com.example.drm1:SMART-TV/PC,com.example.drm2:\",VIDEO-RANGE=PQ,REQ-VIDEO-LAYOUT=\"CH-STEREO,CH-MONO\",STABLE-VARIANT-ID=\"azBY09+/=.-_\",VIDEO=\"great-video\",PATHWAY-ID=\"cool-pathway\"\ngreat-playlist.m3u8\n");
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
        assert_eq!(output, b"#EXT-X-SESSION-KEY:METHOD=AES-128,URI=\"https://example.com/foo.key\",IV=0x0F91DC05,KEYFORMAT=\"super cool key format\",KEYFORMATVERSIONS=\"1/16\"\n");

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
        assert_eq!(output, b"#EXT-X-SESSION-KEY:METHOD=SAMPLE-AES,URI=\"https://example.com/foo.key\",IV=0x0F91DC05,KEYFORMATVERSIONS=\"1/16\"\n");

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
