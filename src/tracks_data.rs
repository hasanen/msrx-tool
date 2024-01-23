use crate::data_format::DataFormat;
use crate::iso_data::IsoData;
use crate::msrx_tool_error::MsrxToolError;
use crate::track_data::TrackData;
use crate::track_status::TrackStatus;
use crate::tracks_data;

// Page 15 in "MSR605 Programmer's Manual"
const WRITE_BLOCK_START_FIELD: [u8; 2] = [0x1b, 0x73];
const WRITE_BLOCK_END_FIELD: [u8; 2] = [0x3f, 0x1c];
const TRACK_1_START_FIELD: [u8; 2] = [0x1b, 0x01];
const TRACK_2_START_FIELD: [u8; 2] = [0x1b, 0x02];
const TRACK_3_START_FIELD: [u8; 2] = [0x1b, 0x03];
const TRACK_1_START_SENTINEL: char = '%';
const TRACK2_3_START_SENTINEL: char = ';';
const TRACK_END_SENTINEL: char = '?';

const TRACK1_MAX_LENGTH: usize = 79;
const TRACK2_MAX_LENGTH: usize = 40;
const TRACK3_MAX_LENGTH: usize = 107;

pub const TRACK1_SUPPORTED_ASCII: &str =
    " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_";
pub const TRACK2_3_SUPPORTED_ASCII: &str = "0123456789:;<=>?";

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
        let splits: Vec<&str> = text.split(*separator).collect();

        //Validation should be done in TrackData and not here, but due
        // to time constraint, no time to refactor or do it properly
        let track1_data = Self::validate_track(
            1,
            splits.get(0),
            TRACK1_MAX_LENGTH,
            TRACK1_SUPPORTED_ASCII,
            TRACK_1_START_SENTINEL,
            TRACK_END_SENTINEL,
        )?;
        let track2_data = Self::validate_track(
            2,
            splits.get(1),
            TRACK2_MAX_LENGTH,
            TRACK2_3_SUPPORTED_ASCII,
            TRACK2_3_START_SENTINEL,
            TRACK_END_SENTINEL,
        )?;
        let track3_data = Self::validate_track(
            3,
            splits.get(2),
            TRACK3_MAX_LENGTH,
            TRACK2_3_SUPPORTED_ASCII,
            TRACK2_3_START_SENTINEL,
            TRACK_END_SENTINEL,
        )?;

        let tracks_data = TracksData {
            track1: TrackData {
                data: track1_data,
                format: DataFormat::Iso,
            },
            track2: TrackData {
                data: track2_data,
                format: DataFormat::Iso,
            },
            track3: TrackData {
                data: track3_data,
                format: DataFormat::Iso,
            },
            status: TrackStatus::ParsedFromInput,
        };

        Ok(tracks_data)
    }

    /// Converts the data to a data block as it's defined in the manual
    pub fn to_data_block(&self) -> Result<Vec<u8>, MsrxToolError> {
        let card_data = TRACK_1_START_FIELD
            .to_vec()
            .into_iter()
            .chain(self.track1.as_packets().clone())
            .chain(TRACK_2_START_FIELD.to_vec())
            .chain(self.track2.as_packets().clone())
            .chain(TRACK_3_START_FIELD.to_vec())
            .chain(self.track3.as_packets().clone())
            .collect::<Vec<u8>>();

        let data_block = WRITE_BLOCK_START_FIELD
            .to_vec()
            .into_iter()
            .chain(card_data.clone())
            .chain(WRITE_BLOCK_END_FIELD.to_vec())
            .collect::<Vec<u8>>();

        Ok(data_block)
    }

    fn validate_track(
        track_number: usize,
        data: Option<&&str>,
        track_max_length: usize,
        track_supported_ascii: &str,
        start_sentinel: char,
        end_sentinel: char,
    ) -> Result<Vec<u8>, MsrxToolError> {
        let data_vec = match data {
            Some(data) => data.as_bytes().to_vec(),
            None => vec![0x00],
        };

        if data_vec.len() > track_max_length {
            Err(MsrxToolError::DataForTrackIsTooLong(
                track_number,
                data_vec.len(),
                track_max_length,
            ))
        } else if data_vec != vec![0x00] {
            if !data_vec
                .iter()
                .all(|&c| track_supported_ascii.contains(c as char))
            {
                Err(MsrxToolError::InvalidTrackData(
                    track_number,
                    track_supported_ascii.to_string(),
                ))
            } else if data_vec[0] != start_sentinel as u8 {
                return Err(MsrxToolError::InvalidStartSentinel(
                    track_number,
                    start_sentinel,
                ));
            } else if data_vec[data_vec.len() - 1] != end_sentinel as u8 {
                return Err(MsrxToolError::InvalidEndSentinel(
                    track_number,
                    end_sentinel,
                ));
            } else {
                Ok(data_vec)
            }
        } else {
            Ok(data_vec)
        }
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
        fn test_parse_text_one_track() -> Result<(), MsrxToolError> {
            let separator = '_';

            let data_to_parse =
                "%ABCDEFGHIJKLMNOPQRSTU1234567890ABCDEFGHIJKLMNOPQRSTU1234567890ABCDEFGHIJKLMN?"
                    .to_string();

            let result = TracksData::from_str(&data_to_parse, &separator)?;

            let expected_track1 = vec![
                0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e,
                0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
                0x38, 0x39, 0x30, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b,
                0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x31, 0x32, 0x33, 0x34,
                0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48,
                0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e,
            ];
            let expected_track2: Vec<u8> = vec![0x00];
            let expected_track3: Vec<u8> = vec![0x00];

            assert_eq!(expected_track1, result.track1.data);
            assert_eq!(expected_track2, result.track2.data);
            assert_eq!(expected_track3, result.track3.data);
            Ok(())
        }

        #[test]
        fn test_parse_text_two_tracks() -> Result<(), MsrxToolError> {
            let separator = '_';

            let data_to_parse = vec![
                "%ABCDEFGHIJKLMNOPQRSTU1234567890ABCDEFGHIJKLMNOPQRSTU1234567890ABCDEFGHIJKLMN?",
                ";0987654321098765432109876543210987654?",
            ]
            .join(&separator.to_string());

            let result = TracksData::from_str(&data_to_parse, &separator)?;

            let expected_track1 = vec![
                0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e,
                0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
                0x38, 0x39, 0x30, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b,
                0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x31, 0x32, 0x33, 0x34,
                0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48,
                0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e,
            ];
            let expected_track2 = vec![
                0x30, 0x39, 0x38, 0x37, 0x36, 0x35, 0x34, 0x33, 0x32, 0x31, 0x30, 0x39, 0x38, 0x37,
                0x36, 0x35, 0x34, 0x33, 0x32, 0x31, 0x30, 0x39, 0x38, 0x37, 0x36, 0x35, 0x34, 0x33,
                0x32, 0x31, 0x30, 0x39, 0x38, 0x37, 0x36, 0x35, 0x34,
            ];
            let expected_track3: Vec<u8> = vec![0x00];

            assert_eq!(expected_track1, result.track1.data);
            assert_eq!(expected_track2, result.track2.data);
            assert_eq!(expected_track3, result.track3.data);
            Ok(())
        }

        #[test]
        fn test_parse_text_three_tracks() -> Result<(), MsrxToolError> {
            let separator = '_';

            let data_to_parse = vec![
                "%ABCDEFGHIJKLMNOPQRSTU1234567890ABCDEFGHIJKLMNOPQRSTU1234567890ABCDEFGHIJKLMN?",
                ";0987654321098765432109876543210987654?",
                ";12345?",
            ]
            .join(&separator.to_string());

            let result = TracksData::from_str(&data_to_parse, &separator)?;

            let expected_track1 = vec![
                0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e,
                0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
                0x38, 0x39, 0x30, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b,
                0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x31, 0x32, 0x33, 0x34,
                0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48,
                0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e,
            ];
            let expected_track2 = vec![
                0x30, 0x39, 0x38, 0x37, 0x36, 0x35, 0x34, 0x33, 0x32, 0x31, 0x30, 0x39, 0x38, 0x37,
                0x36, 0x35, 0x34, 0x33, 0x32, 0x31, 0x30, 0x39, 0x38, 0x37, 0x36, 0x35, 0x34, 0x33,
                0x32, 0x31, 0x30, 0x39, 0x38, 0x37, 0x36, 0x35, 0x34,
            ];
            let expected_track3 = vec![0x31, 0x32, 0x33, 0x34, 0x35];

            assert_eq!(expected_track1, result.track1.data);
            assert_eq!(expected_track2, result.track2.data);
            assert_eq!(expected_track3, result.track3.data);
            Ok(())
        }

        #[test]
        fn test_from_str_validate_track1_characters_valid_chars() -> Result<(), MsrxToolError> {
            let data_to_parse = format!("%{}?_;1?_;1?", "1".repeat(30));

            let result: Result<TracksData, MsrxToolError> =
                TracksData::from_str(&data_to_parse, &'_');

            match result {
                Ok(_) => Ok(()),
                Err(e) => {
                    panic!("Expected an Ok, got Err")
                }
            }
        }

        #[test]
        fn test_from_str_validate_track1_characters_invalid_start_sentinel(
        ) -> Result<(), MsrxToolError> {
            let data_to_parse = "A?_;1?_;1?".to_string();

            let result: Result<TracksData, MsrxToolError> =
                TracksData::from_str(&data_to_parse, &'_');

            match result {
                Ok(_) => panic!("Expected an Err, got Ok"),
                Err(e) => {
                    assert_eq!(e, MsrxToolError::InvalidStartSentinel(1, '%'));

                    Ok(())
                }
            }
        }

        #[test]
        fn test_from_str_validate_track1_characters_invalid_end_sentinel(
        ) -> Result<(), MsrxToolError> {
            let data_to_parse = "%A_;1?_;1?".to_string();

            let result: Result<TracksData, MsrxToolError> =
                TracksData::from_str(&data_to_parse, &'_');

            match result {
                Ok(_) => panic!("Expected an Err, got Ok"),
                Err(e) => {
                    assert_eq!(e, MsrxToolError::InvalidEndSentinel(1, '?'));

                    Ok(())
                }
            }
        }

        #[test]
        fn test_from_str_validate_track1_characters_invalid_chars() -> Result<(), MsrxToolError> {
            let data_to_parse = format!("%{}?_;1?_;1?", "{".repeat(30));

            let result: Result<TracksData, MsrxToolError> =
                TracksData::from_str(&data_to_parse, &'_');

            match result {
                Ok(_) => panic!("Expected an Err, got Ok"),
                Err(e) => {
                    assert_eq!(
                        e,
                        MsrxToolError::InvalidTrackData(
                            1,
                            tracks_data::TRACK1_SUPPORTED_ASCII.to_string()
                        )
                    );

                    Ok(())
                }
            }
        }

        #[test]
        fn test_from_str_validate_track1_length() -> Result<(), MsrxToolError> {
            let data_to_parse = "A".repeat(80);

            let result: Result<TracksData, MsrxToolError> =
                TracksData::from_str(&data_to_parse, &'_');

            match result {
                Ok(_) => panic!("Expected an Err, got Ok"),
                Err(e) => {
                    assert_eq!(e, MsrxToolError::DataForTrackIsTooLong(1, 80, 79));

                    Ok(())
                }
            }
        }

        #[test]
        fn test_from_str_validate_track2_characters_valid_chars() -> Result<(), MsrxToolError> {
            let data_to_parse = format!("%A?_;{}?_;1?", "1".repeat(30));

            let result: Result<TracksData, MsrxToolError> =
                TracksData::from_str(&data_to_parse, &'_');

            match result {
                Ok(_) => Ok(()),
                Err(e) => {
                    panic!("Expected an Ok, got Err")
                }
            }
        }

        #[test]
        fn test_from_str_validate_track2_characters_invalid_start_sentinel(
        ) -> Result<(), MsrxToolError> {
            let data_to_parse = "%A?_1?_;1?".to_string();

            let result: Result<TracksData, MsrxToolError> =
                TracksData::from_str(&data_to_parse, &'_');

            match result {
                Ok(_) => panic!("Expected an Err, got Ok"),
                Err(e) => {
                    assert_eq!(e, MsrxToolError::InvalidStartSentinel(2, ';'));

                    Ok(())
                }
            }
        }

        #[test]
        fn test_from_str_validate_track2_characters_invalid_end_sentinel(
        ) -> Result<(), MsrxToolError> {
            let data_to_parse = "%A?_;1_;1?".to_string();

            let result: Result<TracksData, MsrxToolError> =
                TracksData::from_str(&data_to_parse, &'_');

            match result {
                Ok(_) => panic!("Expected an Err, got Ok"),
                Err(e) => {
                    assert_eq!(e, MsrxToolError::InvalidEndSentinel(2, '?'));

                    Ok(())
                }
            }
        }

        #[test]
        fn test_from_str_validate_track2_characters_invalid_chars() -> Result<(), MsrxToolError> {
            let data_to_parse = format!("%A?_;{}?_;1?", "-".repeat(30));

            let result: Result<TracksData, MsrxToolError> =
                TracksData::from_str(&data_to_parse, &'_');

            match result {
                Ok(_) => panic!("Expected an Err, got Ok"),
                Err(e) => {
                    assert_eq!(
                        e,
                        MsrxToolError::InvalidTrackData(
                            2,
                            tracks_data::TRACK2_3_SUPPORTED_ASCII.to_string()
                        )
                    );

                    Ok(())
                }
            }
        }

        #[test]
        fn test_from_str_validate_track2_length() -> Result<(), MsrxToolError> {
            let data_to_parse = format!("%A?_;{}?_;1?", "1".repeat(39));

            let result: Result<TracksData, MsrxToolError> =
                TracksData::from_str(&data_to_parse, &'_');

            match result {
                Ok(_) => panic!("Expected an Err, got Ok"),
                Err(e) => {
                    assert_eq!(e, MsrxToolError::DataForTrackIsTooLong(2, 41, 40));

                    Ok(())
                }
            }
        }

        #[test]
        fn test_from_str_validate_track3_characters_valid_chars() -> Result<(), MsrxToolError> {
            let data_to_parse = format!("%A?_;1?_;{}?", "1".repeat(30));

            let result: Result<TracksData, MsrxToolError> =
                TracksData::from_str(&data_to_parse, &'_');

            match result {
                Ok(_) => Ok(()),
                Err(e) => {
                    panic!("Expected an Ok, got Err")
                }
            }
        }
        #[test]
        fn test_from_str_validate_track3_characters_invalid_start_sentinel(
        ) -> Result<(), MsrxToolError> {
            let data_to_parse = "%A?_;1?_1?".to_string();

            let result: Result<TracksData, MsrxToolError> =
                TracksData::from_str(&data_to_parse, &'_');

            match result {
                Ok(_) => panic!("Expected an Err, got Ok"),
                Err(e) => {
                    assert_eq!(e, MsrxToolError::InvalidStartSentinel(3, ';'));

                    Ok(())
                }
            }
        }

        #[test]
        fn test_from_str_validate_track3_characters_invalid_end_sentinel(
        ) -> Result<(), MsrxToolError> {
            let data_to_parse = "%A?_;1?_;1".to_string();

            let result: Result<TracksData, MsrxToolError> =
                TracksData::from_str(&data_to_parse, &'_');

            match result {
                Ok(_) => panic!("Expected an Err, got Ok"),
                Err(e) => {
                    assert_eq!(e, MsrxToolError::InvalidEndSentinel(3, '?'));

                    Ok(())
                }
            }
        }

        #[test]
        fn test_from_str_validate_track3_characters_invalid_chars() -> Result<(), MsrxToolError> {
            let data_to_parse = format!("%A?_;1?_;{}?", "-".repeat(30));

            let result: Result<TracksData, MsrxToolError> =
                TracksData::from_str(&data_to_parse, &'_');

            match result {
                Ok(_) => panic!("Expected an Err, got Ok"),
                Err(e) => {
                    assert_eq!(
                        e,
                        MsrxToolError::InvalidTrackData(
                            3,
                            tracks_data::TRACK2_3_SUPPORTED_ASCII.to_string()
                        )
                    );

                    Ok(())
                }
            }
        }

        #[test]
        fn test_from_str_validate_track3_length() -> Result<(), MsrxToolError> {
            let data_to_parse = format!("%A?_;1?_;{}?", "1".repeat(106));

            let result: Result<TracksData, MsrxToolError> =
                TracksData::from_str(&data_to_parse, &'_');

            match result {
                Ok(_) => panic!("Expected an Err, got Ok"),
                Err(e) => {
                    assert_eq!(e, MsrxToolError::DataForTrackIsTooLong(3, 108, 107));

                    Ok(())
                }
            }
        }
    }

    mod to_packets {
        use super::*;

        #[test]
        fn test_to_packets_one_track() -> Result<(), MsrxToolError> {
            let tracks_data = TracksData {
                track1: TrackData {
                    data: vec![0x41, 0x42, 0x43, 0x31, 0x32, 0x33],
                    format: DataFormat::Iso,
                },
                track2: TrackData {
                    data: vec![],
                    format: DataFormat::Iso,
                },
                track3: TrackData {
                    data: vec![],
                    format: DataFormat::Iso,
                },
                status: TrackStatus::ParsedFromInput,
            };

            let packets = tracks_data.to_data_block()?;

            let expected_packets =
                *b"\x1b\x73\x1b\x01\x41\x42\x43\x31\x32\x33\x1b\x02\x00\x1b\x03\x00\x3f\x1c";

            assert_eq!(&expected_packets.to_vec(), &packets);

            Ok(())
        }

        #[test]
        fn test_to_packets_one_track_middle_track() -> Result<(), MsrxToolError> {
            let tracks_data = TracksData {
                track1: TrackData {
                    data: vec![],
                    format: DataFormat::Iso,
                },
                track2: TrackData {
                    data: vec![0x41, 0x42, 0x43, 0x31, 0x32, 0x33],
                    format: DataFormat::Iso,
                },
                track3: TrackData {
                    data: vec![],
                    format: DataFormat::Iso,
                },
                status: TrackStatus::ParsedFromInput,
            };

            let packets = tracks_data.to_data_block()?;

            let expected_packets =
                *b"\x1b\x73\x1b\x01\x00\x1b\x02\x41\x42\x43\x31\x32\x33\x1b\x03\x00\x3f\x1c";

            assert_eq!(&expected_packets.to_vec(), &packets);

            Ok(())
        }

        #[test]
        fn test_to_packets_two_tracks() -> Result<(), MsrxToolError> {
            let tracks_data = TracksData {
                track1: TrackData {
                    data: vec![0x41, 0x42, 0x43, 0x31, 0x32, 0x33],
                    format: DataFormat::Iso,
                },
                track2: TrackData {
                    data: vec![0x31, 0x32, 0x33, 0x34, 0x35],
                    format: DataFormat::Iso,
                },
                track3: TrackData {
                    data: vec![],
                    format: DataFormat::Iso,
                },
                status: TrackStatus::ParsedFromInput,
            };

            let packets = tracks_data.to_data_block()?;

            let expected_packets = *b"\x1b\x73\x1b\x01\x41\x42\x43\x31\x32\x33\x1b\x02\x31\x32\x33\x34\x35\x1b\x03\x00\x3f\x1c";

            assert_eq!(&expected_packets.to_vec(), &packets);

            Ok(())
        }

        #[test]
        fn test_to_data_block_three_tracks_one_packet() -> Result<(), MsrxToolError> {
            let tracks_data = TracksData {
                track1: TrackData {
                    data: vec![0x41, 0x42, 0x43, 0x31, 0x32, 0x33],
                    format: DataFormat::Iso,
                },
                track2: TrackData {
                    data: vec![0x31, 0x32, 0x33, 0x34, 0x35],
                    format: DataFormat::Iso,
                },
                track3: TrackData {
                    data: vec![0x31, 0x32, 0x33, 0x34, 0x35],
                    format: DataFormat::Iso,
                },
                status: TrackStatus::ParsedFromInput,
            };

            let packets = tracks_data.to_data_block()?;

            let expected_packets = *b"\x1b\x73\x1b\x01\x41\x42\x43\x31\x32\x33\x1b\x02\x31\x32\x33\x34\x35\x1b\x03\x31\x32\x33\x34\x35\x3f\x1c";

            assert_eq!(&expected_packets.to_vec(), &packets);
            Ok(())
        }

        #[test]
        fn test_to_data_block_three_tracks_multiple_packets() -> Result<(), MsrxToolError> {
            let tracks_data = TracksData {
                track1: TrackData {
                    data: vec![
                        0x25, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b,
                        0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x31, 0x32,
                        0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x41, 0x42, 0x43, 0x44,
                        0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x50,
                        0x51, 0x52, 0x53, 0x54, 0x55, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
                        0x38, 0x39, 0x30, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49,
                        0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x3f,
                    ],
                    format: DataFormat::Iso,
                },
                track2: TrackData {
                    data: vec![
                        0x3b, 0x30, 0x39, 0x38, 0x37, 0x36, 0x35, 0x34, 0x33, 0x32, 0x31, 0x30,
                        0x39, 0x38, 0x37, 0x36, 0x35, 0x34, 0x33, 0x32, 0x31, 0x30, 0x39, 0x38,
                        0x37, 0x36, 0x35, 0x34, 0x33, 0x32, 0x31, 0x30, 0x39, 0x38, 0x37, 0x36,
                        0x35, 0x34, 0x3f,
                    ],
                    format: DataFormat::Iso,
                },
                track3: TrackData {
                    data: vec![0x3b, 0x31, 0x32, 0x33, 0x34, 0x35, 0x3f],
                    format: DataFormat::Iso,
                },
                status: TrackStatus::ParsedFromInput,
            };

            let packets = tracks_data.to_data_block()?;

            let expected_packet = *b"\
            \x1b\x73\x1b\x01\x25\x41\x42\x43\x44\x45\x46\x47\x48\x49\x4a\x4b\x4c\x4d\x4e\x4f\x50\x51\x52\x53\x54\x55\x31\x32\x33\x34\x35\x36\x37\x38\x39\x30\x41\x42\x43\x44\x45\x46\x47\x48\x49\x4a\x4b\x4c\x4d\x4e\x4f\x50\x51\x52\x53\x54\x55\x31\x32\x33\x34\x35\x36\
            \x37\x38\x39\x30\x41\x42\x43\x44\x45\x46\x47\x48\x49\x4a\x4b\x4c\x4d\x4e\x3f\x1b\x02\x3b\x30\x39\x38\x37\x36\x35\x34\x33\x32\x31\x30\x39\x38\x37\x36\x35\x34\x33\x32\x31\x30\x39\x38\x37\x36\x35\x34\x33\x32\x31\x30\x39\x38\x37\x36\x35\x34\x3f\x1b\x03\x3b\
            \x31\x32\x33\x34\x35\x3f\x3f\x1c";

            assert_eq!(&expected_packet.to_vec(), &packets);

            Ok(())
        }
    }
}
