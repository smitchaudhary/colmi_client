use crate::error::ProtocolError;

pub const BIG_DATA_MAGIC: u8 = 0xBC;
pub const DATA_REQUEST_ID_SLEEP: u8 = 0x27;
pub const DATA_REQUEST_ID_OXYGEN: u8 = 0x2A;

pub fn make_data_request(data_id: u8) -> [u8; 6] {
    [BIG_DATA_MAGIC, data_id, 0x00, 0x00, 0xFF, 0xFF]
}

pub fn parse_big_data_header(bytes: &[u8]) -> Result<(u8, u16), ProtocolError> {
    if bytes.len() < 6 {
        return Err(ProtocolError::PacketLength);
    }
    if bytes[0] != BIG_DATA_MAGIC {
        return Err(ProtocolError::InvalidMagic {
            expected: BIG_DATA_MAGIC,
            actual: bytes[0],
        });
    }
    let data_len = u16::from_le_bytes([bytes[2], bytes[3]]);
    Ok((bytes[1], data_len))
}

pub fn sleep_phase_label(phase_type: u8) -> &'static str {
    match phase_type {
        0 => "no data",
        1 => "error",
        2 => "light",
        3 => "deep",
        4 => "REM",
        5 => "awake",
        _ => "unknown",
    }
}

#[derive(Clone, Debug)]
pub struct SleepPhase {
    pub phase_type: u8,
    pub minutes: u8,
}

#[derive(Clone, Debug)]
pub struct SleepDay {
    pub days_ago: u8,
    /// Minutes after midnight.
    pub start_minutes: u16,
    /// Minutes after midnight.
    pub end_minutes: u16,
    pub phases: Vec<SleepPhase>,
}

#[derive(Clone, Debug)]
pub struct SleepData {
    pub days: Vec<SleepDay>,
}

pub fn parse_sleep_data(bytes: &[u8]) -> Result<SleepData, ProtocolError> {
    let (id, _data_len) = parse_big_data_header(bytes)?;
    if id != DATA_REQUEST_ID_SLEEP {
        return Err(ProtocolError::CommandId {
            expected: DATA_REQUEST_ID_SLEEP,
            actual: id,
        });
    }

    let data = &bytes[6..];
    let mut days = Vec::new();

    if data.is_empty() {
        return Ok(SleepData { days });
    }
    let mut index = 1;

    while index + 1 < data.len() {
        let record_start = index - 1;
        let days_ago = data[index];
        let byte_count = data[index + 1] as usize;
        index += 2;

        let phases_start = record_start + 6;
        let phases_end = phases_start + byte_count.saturating_sub(4);
        if byte_count < 4 || phases_end > data.len() {
            break;
        }

        let start_minutes = u16::from_le_bytes([data[index], data[index + 1]]);
        let end_minutes = u16::from_le_bytes([data[index + 2], data[index + 3]]);
        index += 4;

        let mut phases = Vec::new();
        while index + 1 < data.len() && index < phases_end {
            phases.push(SleepPhase {
                phase_type: data[index],
                minutes: data[index + 1],
            });
            index += 2;
        }

        days.push(SleepDay {
            days_ago,
            start_minutes,
            end_minutes,
            phases,
        });
    }

    Ok(SleepData { days })
}

#[derive(Clone, Debug)]
pub struct OxygenSample {
    pub min: u8,
    pub max: u8,
}

#[derive(Clone, Debug)]
pub struct OxygenDay {
    pub days_ago: u8,
    /// One sample per hour, 24 samples per day.
    pub samples: Vec<OxygenSample>,
}

#[derive(Clone, Debug)]
pub struct OxygenData {
    pub days: Vec<OxygenDay>,
}

pub fn parse_oxygen_data(bytes: &[u8]) -> Result<OxygenData, ProtocolError> {
    let (id, _data_len) = parse_big_data_header(bytes)?;
    if id != DATA_REQUEST_ID_OXYGEN {
        return Err(ProtocolError::CommandId {
            expected: DATA_REQUEST_ID_OXYGEN,
            actual: id,
        });
    }

    let data = &bytes[6..];
    let mut days = Vec::new();

    // Each day block is 1 (days-ago marker) + 24 * 2 (min/max pairs)
    const DAY_BLOCK_SIZE: usize = 49;

    let mut index = 0;
    while index + DAY_BLOCK_SIZE <= data.len() {
        let days_ago = data[index];
        let mut samples = Vec::with_capacity(24);
        for i in 0..24 {
            samples.push(OxygenSample {
                min: data[index + 1 + i * 2],
                max: data[index + 2 + i * 2],
            });
        }
        days.push(OxygenDay { days_ago, samples });
        index += DAY_BLOCK_SIZE;
    }

    Ok(OxygenData { days })
}
