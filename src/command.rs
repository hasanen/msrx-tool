pub enum Command {
    Reset,
    GetFirmwareVersion,
    GetDeviceModel,
    SetBCP,
    SetBPI,
    SetHiCo,
    SetLoCo,
    SetLeadingZeros,
    SetReadModeOnFormatISO,
    SetISOReadModeOn,
    SetReadModeOff,
    TurnLedAllOn,
    TurnLedRedOn,
    TurnLedGreenOn,
    TurnLedYellowOn,
    TurnLedAllOff,
}
impl Command {
    pub fn packets(&self) -> Vec<u8> {
        match self {
            Command::Reset => vec![0x1b, 0x61],
            Command::GetFirmwareVersion => vec![0x1b, 0x76],
            Command::GetDeviceModel => vec![0x1b, 0x74],
            Command::SetBCP => vec![0x1b, 0x6f],
            Command::SetBPI => vec![0x1b, 0x62],
            Command::SetHiCo => vec![0x1b, 0x78],
            Command::SetLoCo => vec![0x1b, 0x79],
            Command::SetLeadingZeros => vec![0x1b, 0x7a],
            Command::SetReadModeOnFormatISO => vec![0x1b, 0x72],
            Command::SetISOReadModeOn => vec![0x1b, 0x72],
            Command::SetReadModeOff => vec![0x1b, 0x61],
            Command::TurnLedAllOn => vec![0x1b, 0x82],
            Command::TurnLedRedOn => vec![0x1b, 0x85],
            Command::TurnLedGreenOn => vec![0x1b, 0x83],
            Command::TurnLedYellowOn => vec![0x1b, 0x84],
            Command::TurnLedAllOff => vec![0x1b, 0x81],
        }
    }

    pub fn with_payload(&self, payload: &Vec<u8>) -> Vec<u8> {
        let mut packets = self.packets().to_vec();
        packets.extend(payload);
        return packets;
    }
}
