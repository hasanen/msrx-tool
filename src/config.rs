#[derive(Debug)]
pub struct TrackConfig {
    pub bpc: u8,
    pub bpi: u8,
    pub bpi75: u8,
    pub bpi210: u8,
}
impl TrackConfig {
    pub fn bpi_packets(&self) -> Vec<u8> {
        match self.bpi {
            75 => vec![self.bpi75].clone(),
            210 => vec![self.bpi210].clone(),
            _ => panic!("Invalid BPI"),
        }
    }
}

#[derive(Debug)]
pub struct DeviceConfig {
    pub track1: TrackConfig,
    pub track2: TrackConfig,
    pub track3: TrackConfig,
    pub leading_zero210: u8,
    pub leading_zero75: u8,
    pub is_hi_co: bool,
    pub product_id: u16,
    pub vendor_id: u16,
}

impl DeviceConfig {
    pub fn msrx6() -> DeviceConfig {
        DeviceConfig {
            track1: TrackConfig {
                bpc: 7,
                bpi: 210,
                bpi75: 0xa0,
                bpi210: 0xa1,
            },
            track2: TrackConfig {
                bpc: 5,
                bpi: 75,
                bpi75: 0xc0,
                bpi210: 0xc1,
            },
            track3: TrackConfig {
                bpc: 5,
                bpi: 210,
                bpi75: 0x4b,
                bpi210: 0xd2,
            },
            leading_zero210: 61,
            leading_zero75: 22,
            is_hi_co: true,
            product_id: 0x0003,
            vendor_id: 0x0801,
        }
    }

    pub fn bpc_packets(&self) -> Vec<u8> {
        [self.track1.bpc, self.track2.bpc, self.track3.bpc]
            .iter()
            .cloned()
            .collect::<Vec<u8>>()
    }

    pub fn leading_zero_packets(&self) -> Vec<u8> {
        [self.leading_zero210, self.leading_zero75]
            .iter()
            .cloned()
            .collect::<Vec<u8>>()
    }
}
