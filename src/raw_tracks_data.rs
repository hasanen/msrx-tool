use crate::msrx_tool_error::MsrxToolError;
use crate::raw_data::RawData;
use crate::to_hex::ToHex;
use crate::track_data::TrackData;
use crate::track_status::TrackStatus;

#[derive(Debug)]
pub struct RawTracksData {
    pub raw_data: Vec<RawData>,
    pub track1: TrackData,
    pub track2: TrackData,
    pub track3: TrackData,
    pub status: TrackStatus,
}

impl TryFrom<Vec<RawData>> for RawTracksData {
    type Error = MsrxToolError;

    fn try_from(raw_datas: Vec<RawData>) -> Result<Self, Self::Error> {
        for raw_device_data in raw_datas.iter() {}

        let mut combined_raw_data: Vec<u8> = raw_datas[0].raw_data.to_vec();
        if raw_datas.len() > 1 {
            for raw_device_data in raw_datas[1..].iter() {
                combined_raw_data.extend_from_slice(&raw_device_data.raw_data[1..]);
            }
        }

        dbg!(&combined_raw_data.to_hex());
        if combined_raw_data[1] != 0x1b || combined_raw_data[2] != 0x73 {
            return Err(MsrxToolError::RawDataNotCardData);
        }
        let mut data_length = combined_raw_data[0];
        dbg!(&data_length);
        // if raw_device_data.is_header {
        //     data_length &= !0x80;
        // }
        // dbg!(&data_length);
        // if raw_device_data.is_last_packet {
        //     data_length &= !0x40;
        // }
        // dbg!(&data_length);
        // let raw_data = combined_raw_data[1..data_length as usize + 1].to_vec();

        let mut tracks: Vec<Vec<u8>> = vec![];
        let mut read_index = 3;
        for i in 1..=3 {
            dbg!(i);
            dbg!(read_index);
            dbg!(combined_raw_data[read_index]);
            if combined_raw_data[read_index] != 0x1b || combined_raw_data[read_index + 1] != i {
                return Err(MsrxToolError::RawDataNotCardData);
            }
            read_index += 2;
            let track_len = combined_raw_data[read_index] as usize;
            dbg!(&track_len);
            read_index += 1;
            let track_data = combined_raw_data[read_index..read_index + track_len].to_vec();
            tracks.push(track_data);
            read_index += track_len;
        }

        dbg!(read_index);
        // Confirm the ending sequence 3F 1C 1B
        if combined_raw_data[read_index] != 0x3f
            || combined_raw_data[read_index + 1] != 0x1c
            || combined_raw_data[read_index + 2] != 0x1b
        {
            return Err(MsrxToolError::RawDataNotCardData);
        }
        read_index += 3;
        dbg!(read_index);
        let status = TrackStatus::from(combined_raw_data[read_index]);
        dbg!(combined_raw_data[read_index]);
        dbg!(status);

        Ok(RawTracksData {
            raw_data: raw_datas.clone(),
            track1: tracks[0].clone().try_into()?,
            track2: tracks[1].clone().try_into()?,
            track3: tracks[2].clone().try_into()?,
            status,
        })
    }
}
impl TryFrom<RawData> for RawTracksData {
    type Error = MsrxToolError;

    fn try_from(raw_device_data: RawData) -> Result<Self, Self::Error> {
        vec![raw_device_data].try_into()
        // let value = raw_device_data.raw_data;
        // if value[1] != 0x1b || value[2] != 0x73 {
        //     return Err(MsrxToolError::RawDataNotCardData);
        // }
        // let mut data_length = value[0];
        // if raw_device_data.is_header && raw_device_data.is_last_packet {
        //     data_length &= !(0x80 | 0x40);
        // }
        // let raw_data = value[1..data_length as usize + 1].to_vec();

        // let mut read_index = 2;
        // let mut tracks: Vec<Vec<u8>> = vec![];
        // for i in 1..=3 {
        //     if raw_data[read_index] != 0x1b || raw_data[read_index + 1] != i {
        //         return Err(MsrxToolError::RawDataNotCardData);
        //     }
        //     read_index += 2;
        //     let track_len = raw_data[read_index] as usize;
        //     read_index += 1;
        //     let track_data = raw_data[read_index..read_index + track_len].to_vec();
        //     tracks.push(track_data);
        //     read_index += track_len;
        // }

        // // Confirm the ending sequence 3F 1C 1B
        // if raw_data[read_index] != 0x3f
        //     || raw_data[read_index + 1] != 0x1c
        //     || raw_data[read_index + 2] != 0x1b
        // {
        //     return Err(MsrxToolError::RawDataNotCardData);
        // }
        // read_index += 3;
        // let status = TrackStatus::from(raw_data[read_index]);

        // Ok(RawTracksData {
        //     raw_data: raw_device_data,
        //     track1: tracks[0].clone().try_into()?,
        //     track2: tracks[1].clone().try_into()?,
        //     track3: tracks[2].clone().try_into()?,
        //     status,
        // })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod raw_track_data_statuses {
        use super::*;

        #[test]
        fn test_convert_raw_data_to_raw_track_data_status_ok() -> Result<(), MsrxToolError> {
            // Track 1 and Track 2 doesn't contain ant data, Track  3 data is: "1"
            let data = *b"\xd3\x1b\x73\x1b\x01\x00\x1b\x02\x00\x1b\x03\x04\xaf\xc2\xb0\x00\x3f\x1c\x1b\x30\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
            let raw_data: RawData = data.try_into()?;
            let raw_track_data: RawTracksData = raw_data.try_into()?;

            assert_eq!(raw_track_data.status, TrackStatus::Ok);
            Ok(())
        }

        #[test]
        fn test_convert_raw_data_to_raw_track_data_status_write_or_read_error(
        ) -> Result<(), MsrxToolError> {
            // Track 1 and Track 2 doesn't contain ant data, Track  3 data is: "1"
            let data =
                *b"\xd3\x1b\x73\x1b\x01\x00\x1b\x02\x00\x1b\x03\x04\xaf\xc2\xb0\x00\x3f\x1c\x1b\x31\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";

            let raw_data: RawData = data.try_into()?;
            let raw_track_data: RawTracksData = raw_data.try_into()?;

            assert_eq!(raw_track_data.status, TrackStatus::WriteOrReadError);
            Ok(())
        }

        #[test]
        fn test_convert_raw_data_to_raw_track_data_status_command_format_error(
        ) -> Result<(), MsrxToolError> {
            // Track 1 and Track 2 doesn't contain ant data, Track  3 data is: "1"
            let data =
                *b"\xd3\x1b\x73\x1b\x01\x00\x1b\x02\x00\x1b\x03\x04\xaf\xc2\xb0\x00\x3f\x1c\x1b\x32\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
            let raw_data: RawData = data.try_into()?;
            let raw_track_data: RawTracksData = raw_data.try_into()?;

            assert_eq!(raw_track_data.status, TrackStatus::CommandFormatError);
            Ok(())
        }

        #[test]
        fn test_convert_raw_data_to_raw_track_data_status_invalid_command(
        ) -> Result<(), MsrxToolError> {
            // Track 1 and Track 2 doesn't contain ant data, Track  3 data is: "1"
            let data =
                *b"\xd3\x1b\x73\x1b\x01\x00\x1b\x02\x00\x1b\x03\x04\xaf\xc2\xb0\x00\x3f\x1c\x1b\x34\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
            let raw_data: RawData = data.try_into()?;
            let raw_track_data: RawTracksData = raw_data.try_into()?;

            assert_eq!(raw_track_data.status, TrackStatus::InvalidCommand);
            Ok(())
        }

        #[test]
        fn test_convert_raw_data_to_raw_track_data_status_invalid_card_swipe(
        ) -> Result<(), MsrxToolError> {
            // Track 1 and Track 2 doesn't contain ant data, Track  3 data is: "1"
            let data =
                *b"\xd3\x1b\x73\x1b\x01\x00\x1b\x02\x00\x1b\x03\x04\xaf\xc2\xb0\x00\x3f\x1c\x1b\x39\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
            let raw_data: RawData = data.try_into()?;
            let raw_track_data: RawTracksData = raw_data.try_into()?;

            assert_eq!(raw_track_data.status, TrackStatus::InvalidCardSwipeOnWrite);
            Ok(())
        }

        #[test]
        fn test_multiple_raw_datas_to_raw_tracks_data() -> Result<(), MsrxToolError> {
            let data1 = *b"\xbf\x1b\x73\x1b\x01\x2a\x51\x43\x67\x13\x32\x73\x0b\x2a\x6b\x1a\x46\x76\x52\x26\x16\x4f\x57\x4a\x7a\x07\x2f\x0e\x62\x37\x23\x3b\x5b\x45\x25\x64\x15\x54\x34\x75\x0d\x4c\x04\x7c\x01\x00\x00\x00\x1b\x02\x1a\x1a\x01\x13\x02\x1c\x0d\x15\x04\x19\x08\x10\x01\x13";
            let data2 = *b"\x6e\x02\x1c\x0d\x15\x04\x19\x08\x10\x1f\x04\x00\x00\x00\x1b\x03\x1a\x1a\x01\x10\x08\x19\x04\x15\x0d\x1c\x02\x13\x01\x10\x08\x19\x04\x15\x0d\x1c\x02\x13\x1f\x04\x00\x00\x00\x3f\x1c\x1b\x30\x00\x1b\x02\x1a\x1a\x01\x13\x02\x1c\x0d\x15\x04\x19\x08\x10\x01\x13";

            let raw_data1: RawData = data1.try_into()?;
            let raw_data2: RawData = data2.try_into()?;
            let raw_track_data: RawTracksData = vec![raw_data1, raw_data2].try_into()?;

            assert_eq!(raw_track_data.status, TrackStatus::Ok);

            Ok(())
        }
    }
}
