use crate::msrx_tool_error::MsrxToolError;
use crate::raw_device_data::RawDeviceData;
use crate::track_data::TrackData;
use crate::track_status::TrackStatus;

#[derive(Debug)]
pub struct RawTracksData {
    pub raw_device_data: RawDeviceData,
    pub track1: TrackData,
    pub track2: TrackData,
    pub track3: TrackData,
    pub status: TrackStatus,
}
impl RawTracksData {
    pub fn is_header(&self) -> bool {
        self.raw_device_data.is_header
    }
    pub fn is_last_packet(&self) -> bool {
        self.raw_device_data.is_last_packet
    }
}
impl TryFrom<RawDeviceData> for RawTracksData {
    type Error = MsrxToolError;

    fn try_from(raw_device_data: RawDeviceData) -> Result<Self, Self::Error> {
        let value = raw_device_data.raw_data;
        if value[1] != 0x1b || value[2] != 0x73 {
            return Err(MsrxToolError::RawDataNotCardData);
        }
        let mut data_length = value[0];
        if raw_device_data.is_header && raw_device_data.is_last_packet {
            data_length &= !(0x80 | 0x40);
        }
        let raw_data = value[1..data_length as usize + 1].to_vec();

        dbg!(&raw_data);

        let mut read_index = 2;
        let mut tracks: Vec<Vec<u8>> = vec![];
        for i in 1..=3 {
            if raw_data[read_index] != 0x1b || raw_data[read_index + 1] != i {
                return Err(MsrxToolError::RawDataNotCardData);
            }
            read_index += 2;
            let track_len = raw_data[read_index] as usize;
            read_index += 1;
            let track_data = raw_data[read_index..read_index + track_len].to_vec();
            tracks.push(track_data);
            read_index += track_len;
        }

        // Confirm the ending sequence 3F 1C 1B
        if raw_data[read_index] != 0x3f
            || raw_data[read_index + 1] != 0x1c
            || raw_data[read_index + 2] != 0x1b
        {
            return Err(MsrxToolError::RawDataNotCardData);
        }
        read_index += 3;
        let status = TrackStatus::from(raw_data[read_index]);

        Ok(RawTracksData {
            raw_device_data,
            track1: tracks[0].clone().try_into()?,
            track2: tracks[1].clone().try_into()?,
            track3: tracks[2].clone().try_into()?,
            status,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_raw_data_to_raw_track_data_is_header() -> Result<(), MsrxToolError> {
        // Track 1 and Track 2 doesn't contain ant data, Track  3 data is: "1"
        let data = *b"\xd3\x1b\x73\x1b\x01\x00\x1b\x02\x00\x1b\x03\x04\xaf\xc2\xb0\x00\x3f\x1c\x1b\x30\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let raw_data: RawDeviceData = data.try_into()?;
        let raw_track_data: RawTracksData = raw_data.try_into()?;

        assert_eq!(raw_track_data.is_header(), true);
        Ok(())
    }

    #[test]
    fn test_convert_raw_data_to_raw_track_data_is_last_packet() -> Result<(), MsrxToolError> {
        // Track 1 and Track 2 doesn't contain ant data, Track  3 data is: "1"
        let data = *b"\xd3\x1b\x73\x1b\x01\x00\x1b\x02\x00\x1b\x03\x04\xaf\xc2\xb0\x00\x3f\x1c\x1b\x30\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let raw_data: RawDeviceData = data.try_into()?;
        let raw_track_data: RawTracksData = raw_data.try_into()?;

        assert_eq!(raw_track_data.is_last_packet(), true);
        Ok(())
    }
    mod raw_track_data_statuses {
        use super::*;

        #[test]
        fn test_convert_raw_data_to_raw_track_data_status_ok() -> Result<(), MsrxToolError> {
            // Track 1 and Track 2 doesn't contain ant data, Track  3 data is: "1"
            let data = *b"\xd3\x1b\x73\x1b\x01\x00\x1b\x02\x00\x1b\x03\x04\xaf\xc2\xb0\x00\x3f\x1c\x1b\x30\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
            let raw_data: RawDeviceData = data.try_into()?;
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

            let raw_data: RawDeviceData = data.try_into()?;
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
            let raw_data: RawDeviceData = data.try_into()?;
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
            let raw_data: RawDeviceData = data.try_into()?;
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
            let raw_data: RawDeviceData = data.try_into()?;
            let raw_track_data: RawTracksData = raw_data.try_into()?;

            assert_eq!(raw_track_data.status, TrackStatus::InvalidCardSwipeOnWrite);
            Ok(())
        }
    }
}
