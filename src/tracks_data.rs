use crate::data_format::DataFormat;
use crate::iso_data::IsoData;
use crate::msrx_tool_error::MsrxToolError;
use crate::track_data::TrackData;
use crate::track_status::TrackStatus;

#[derive(Debug)]
pub struct TracksData {
    pub track1: TrackData,
    pub track2: TrackData,
    pub track3: TrackData,
    pub status: TrackStatus,
}

// TODO fix this
impl TryFrom<Vec<IsoData>> for TracksData {
    type Error = MsrxToolError;

    fn try_from(raw_datas: Vec<IsoData>) -> Result<Self, Self::Error> {
        let mut combined_raw_data: Vec<u8> = raw_datas[0].raw.data.to_vec();

        if raw_datas.len() > 1 {
            for raw_device_data in raw_datas[1..].iter() {
                combined_raw_data.extend_from_slice(&raw_device_data.raw.data[1..]);
            }
        }
        if combined_raw_data[1] != 0x1b || combined_raw_data[2] != 0x73 {
            return Err(MsrxToolError::RawDataNotCardData);
        }

        let mut tracks: Vec<Vec<u8>> = vec![];
        let mut track_start_index = 0;
        let mut current_track = 0;
        let mut status_char: Option<u8> = None;
        for (index, char) in combined_raw_data.iter().enumerate() {
            match char {
                0x3f => {
                    if combined_raw_data[index + 1] == 0x1c && combined_raw_data[index + 2] == 0x1b
                    {
                        tracks.push(combined_raw_data[track_start_index..index].to_vec());
                        status_char = Some(combined_raw_data[index + 3]);
                        break;
                    }
                }
                0x1b => match combined_raw_data[index + 1] {
                    1 | 2 | 3 => {
                        if current_track != combined_raw_data[index + 1] {
                            if current_track > 0 {
                                tracks.push(combined_raw_data[track_start_index..index].to_vec());
                            }
                            track_start_index = index + 2;
                            current_track = combined_raw_data[index + 1];
                        }
                    }
                    _ => { /* Only three tracks */ }
                },
                _ => { /* No other special sequences to look */ }
            }
        }
        let status = match status_char {
            Some(char) => TrackStatus::from(char),
            None => TrackStatus::Unknown,
        };

        Ok(TracksData {
            track1: TrackData {
                data: tracks[0].clone(),
                format: DataFormat::Iso,
            },
            track2: TrackData {
                data: tracks[1].clone(),
                format: DataFormat::Iso,
            },
            track3: TrackData {
                data: tracks[2].clone(),
                format: DataFormat::Iso,
            },
            status,
        })
    }
}

impl TracksData {
    pub fn from_str(text: &str, separator: &char) -> Result<Self, MsrxToolError> {
        let string = String::new();
        Ok(TracksData {
            track1: TrackData {
                data: string.clone().into_bytes(),
                format: DataFormat::Iso,
            },
            track2: TrackData {
                data: string.clone().into_bytes(),
                format: DataFormat::Iso,
            },
            track3: TrackData {
                data: string.clone().into_bytes(),
                format: DataFormat::Iso,
            },
            status: TrackStatus::ParsedFromInput,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // mod raw_track_data_statuses {
    //     use super::*;

    //     #[test]
    //     fn test_convert_raw_data_to_raw_track_data_status_ok() -> Result<(), MsrxToolError> {
    //         // Track 1 and Track 2 doesn't contain ant data, Track  3 data is: "1"
    //         let data = *b"\xd3x1b\x73\x1b\x01\x00\x1b\x02\x00\x1b\x03\x04\xaf\xc2\xb0\x00\x3f\x1c\x1b\x30\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
    //         let raw_data: DeviceData = DeviceData {
    //             raw: data.try_into()?,
    //             format: DataFormat::Iso,
    //         };
    //         let raw_track_data: TracksData = raw_data.try_into()?;

    //         assert_eq!(raw_track_data.status, TrackStatus::Ok);
    //         Ok(())
    //     }

    //     #[test]
    //     fn test_convert_raw_data_to_raw_track_data_status_write_or_read_error(
    //     ) -> Result<(), MsrxToolError> {
    //         // Track 1 and Track 2 doesn't contain ant data, Track  3 data is: "1"
    //         let data =
    //             *b"\xd3\x1b\x73\x1b\x01\x00\x1b\x02\x00\x1b\x03\x04\xaf\xc2\xb0\x00\x3f\x1c\x1b\x31\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";

    //         let raw_data: DeviceData = DeviceData {
    //             raw: data.try_into()?,
    //             format: DataFormat::Iso,
    //         };
    //         let raw_track_data: TracksData = raw_data.try_into()?;

    //         assert_eq!(raw_track_data.status, TrackStatus::WriteOrReadError);
    //         Ok(())
    //     }

    //     #[test]
    //     fn test_convert_raw_data_to_raw_track_data_status_command_format_error(
    //     ) -> Result<(), MsrxToolError> {
    //         // Track 1 and Track 2 doesn't contain ant data, Track  3 data is: "1"
    //         let data =
    //             *b"\xd3\x1b\x73\x1b\x01\x00\x1b\x02\x00\x1b\x03\x04\xaf\xc2\xb0\x00\x3f\x1c\x1b\x32\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
    //         let raw_data: DeviceData = DeviceData {
    //             raw: data.try_into()?,
    //             format: DataFormat::Iso,
    //         };
    //         let raw_track_data: TracksData = raw_data.try_into()?;

    //         assert_eq!(raw_track_data.status, TrackStatus::CommandFormatError);
    //         Ok(())
    //     }

    //     #[test]
    //     fn test_convert_raw_data_to_raw_track_data_status_invalid_command(
    //     ) -> Result<(), MsrxToolError> {
    //         // Track 1 and Track 2 doesn't contain ant data, Track  3 data is: "1"
    //         let data =
    //             *b"\xd3\x1b\x73\x1b\x01\x00\x1b\x02\x00\x1b\x03\x04\xaf\xc2\xb0\x00\x3f\x1c\x1b\x34\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
    //         let raw_data: DeviceData = DeviceData {
    //             raw: data.try_into()?,
    //             format: DataFormat::Iso,
    //         };
    //         let raw_track_data: TracksData = raw_data.try_into()?;

    //         assert_eq!(raw_track_data.status, TrackStatus::InvalidCommand);
    //         Ok(())
    //     }

    //     #[test]
    //     fn test_convert_raw_data_to_raw_track_data_status_invalid_card_swipe(
    //     ) -> Result<(), MsrxToolError> {
    //         // Track 1 and Track 2 doesn't contain ant data, Track  3 data is: "1"
    //         let data =
    //             *b"\xd3\x1b\x73\x1b\x01\x00\x1b\x02\x00\x1b\x03\x04\xaf\xc2\xb0\x00\x3f\x1c\x1b\x39\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
    //         let raw_data: DeviceData = DeviceData {
    //             raw: data.try_into()?,
    //             format: DataFormat::Iso,
    //         };
    //         let raw_track_data: TracksData = raw_data.try_into()?;

    //         assert_eq!(raw_track_data.status, TrackStatus::InvalidCardSwipeOnWrite);
    //         Ok(())
    //     }

    //     #[test]
    //     fn test_multiple_raw_datas_to_raw_tracks_data() -> Result<(), MsrxToolError> {
    //         let data1 = *b"\xbf\x1b\x73\x1b\x01\x2a\x51\x43\x67\x13\x32\x73\x0b\x2a\x6b\x1a\x46\x76\x52\x26\x16\x4f\x57\x4a\x7a\x07\x2f\x0e\x62\x37\x23\x3b\x5b\x45\x25\x64\x15\x54\x34\x75\x0d\x4c\x04\x7c\x01\x00\x00\x00\x1b\x02\x1a\x1a\x01\x13\x02\x1c\x0d\x15\x04\x19\x08\x10\x01\x13";
    //         let data2 = *b"\x6e\x02\x1c\x0d\x15\x04\x19\x08\x10\x1f\x04\x00\x00\x00\x1b\x03\x1a\x1a\x01\x10\x08\x19\x04\x15\x0d\x1c\x02\x13\x01\x10\x08\x19\x04\x15\x0d\x1c\x02\x13\x1f\x04\x00\x00\x00\x3f\x1c\x1b\x30\x00\x1b\x02\x1a\x1a\x01\x13\x02\x1c\x0d\x15\x04\x19\x08\x10\x01\x13";

    //         let raw_data1: DeviceData = DeviceData {
    //             raw: data1.try_into()?,
    //             format: DataFormat::Iso,
    //         };
    //         let raw_data2: DeviceData = DeviceData {
    //             raw: data2.try_into()?,
    //             format: DataFormat::Iso,
    //         };
    //         let raw_track_data: TracksData = vec![raw_data1, raw_data2].try_into()?;

    //         assert_eq!(raw_track_data.status, TrackStatus::Ok);

    //         Ok(())
    //     }
    // }

    mod raw_track_data_read {
        use super::*;

        #[test]
        fn test_multiple_raw_datas_to_raw_tracks_data() -> Result<(), MsrxToolError> {
            let data1 = *b"\xbf\x1b\x73\x1b\x01\x25\x41\x42\x43\x44\x45\x46\x47\x48\x49\x4a\x4b\x4c\x4d\x4e\x4f\x50\x51\x52\x53\x54\x55\x31\x32\x33\x34\x35\x36\x37\x38\x39\x30\x41\x42\x43\x44\x45\x46\x47\x48\x49\x4a\x4b\x4c\x4d\x4e\x4f\x50\x51\x52\x53\x54\x55\x31\x32\x33\x34\x35\x36";
            let data2 = *b"\x3f\x37\x38\x39\x30\x41\x42\x43\x44\x45\x46\x47\x48\x49\x4a\x4b\x4c\x4d\x4e\x3f\x1b\x02\x3b\x30\x39\x38\x37\x36\x35\x34\x33\x32\x31\x30\x39\x38\x37\x36\x35\x34\x33\x32\x31\x30\x39\x38\x37\x36\x35\x34\x33\x32\x31\x30\x39\x38\x37\x36\x35\x34\x3f\x1b\x03\x3b";
            let data3 = *b"\x4a\x31\x32\x33\x34\x35\x3f\x3f\x1c\x1b\x30\x47\x48\x49\x4a\x4b\x4c\x4d\x4e\x3f\x1b\x02\x3b\x30\x39\x38\x37\x36\x35\x34\x33\x32\x31\x30\x39\x38\x37\x36\x35\x34\x33\x32\x31\x30\x39\x38\x37\x36\x35\x34\x33\x32\x31\x30\x39\x38\x37\x36\x35\x34\x3f\x1b\x03\x3b";

            let raw_data1: IsoData = IsoData {
                raw: data1.try_into()?,
            };
            let raw_data2: IsoData = IsoData {
                raw: data2.try_into()?,
            };
            let raw_data3: IsoData = IsoData {
                raw: data3.try_into()?,
            };
            let raw_track_data: TracksData = vec![raw_data1, raw_data2, raw_data3].try_into()?;

            assert_eq!(
                raw_track_data.track1.to_string()?,
                "%ABCDEFGHIJKLMNOPQRSTU1234567890ABCDEFGHIJKLMNOPQRSTU1234567890ABCDEFGHIJKLMN?"
            );
            assert_eq!(
                raw_track_data.track2.to_string()?,
                ";0987654321098765432109876543210987654?"
            );
            assert_eq!(raw_track_data.track3.to_string()?, ";12345?");

            Ok(())
        }
    }

    mod from_str {
        use super::*;

        #[test]
        fn test_parse_text() -> Result<(), MsrxToolError> {
            let separator = '_';

            let data_to_parse = vec![
                "%ABCDEFGHIJKLMNOPQRSTU1234567890ABCDEFGHIJKLMNOPQRSTU1234567890ABCDEFGHIJKLMN?",
                ";0987654321098765432109876543210987654?",
                ";12345?",
            ]
            .join(&separator.to_string());

            dbg!(&data_to_parse);

            let result = TracksData::from_str(&data_to_parse, &separator)?;

            let expected_track1 = vec![
                0xbf, 0x1b, 0x73, 0x1b, 0x01, 0x25, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48,
                0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x31,
                0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x41, 0x42, 0x43, 0x44, 0x45,
                0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53,
                0x54, 0x55, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36,
            ];
            let expected_track2 = vec![
                0x3f, 0x37, 0x38, 0x39, 0x30, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49,
                0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x3f, 0x1b, 0x02, 0x3b, 0x30, 0x39, 0x38, 0x37, 0x36,
                0x35, 0x34, 0x33, 0x32, 0x31, 0x30, 0x39, 0x38, 0x37, 0x36, 0x35, 0x34, 0x33, 0x32,
                0x31, 0x30, 0x39, 0x38, 0x37, 0x36, 0x35, 0x34, 0x33, 0x32, 0x31, 0x30, 0x39, 0x38,
                0x37, 0x36, 0x35, 0x34, 0x3f, 0x1b, 0x03, 0x3b,
            ];
            let expected_track3 = vec![
                0x4a, 0x31, 0x32, 0x33, 0x34, 0x35, 0x3f, 0x3f, 0x1c, 0x1b, 0x30,
            ];

            assert_eq!(expected_track1, result.track1.data);
            assert_eq!(expected_track2, result.track2.data);
            assert_eq!(expected_track3, result.track3.data);
            Ok(())
        }
    }
}
