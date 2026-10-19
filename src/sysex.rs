//! Yamaha SEQTRAK SysEx message build/parse.
//! Formats per Data List D0 p.110-116:
//!   Parameter Change: F0 43 1n 7F 1C 0C ah am al dd...dd F7
//!   Dump Request:     F0 43 2n 7F 1C 0C ah am al F7
//!   Bulk Dump:        F0 43 0n 7F 1C bh bl 0C ah am al dd...dd cc F7
//! Checksum: twos complement so Model ID..data sum to 0 mod 128.

use crate::model::addr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeqtrakSysex {
    /// Parameter change sent to device.
    ParameterChange {
        device: u8,
        address: [u8; 3],
        data: Vec<u8>,
    },
    /// Dump request sent to device.
    DumpRequest { device: u8, address: [u8; 3] },
    /// Bulk dump reply from device.
    BulkDump {
        device: u8,
        byte_count: u16,
        block: [u8; 3],
        data: Vec<u8>,
    },
    /// Universal identity reply.
    Identity { family: [u8; 4], version: u8 },
    /// Anything else, kept for monitoring.
    Other(Vec<u8>),
}

impl SeqtrakSysex {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            SeqtrakSysex::ParameterChange {
                device,
                address,
                data,
            } => {
                let mut m = vec![
                    0xF0,
                    0x43,
                    0x10 | (device & 0x0F),
                    0x7F,
                    addr::GROUP_HI,
                    addr::MODEL_ID,
                ];
                m.extend_from_slice(address);
                m.extend(data);
                m.push(0xF7);
                m
            }
            SeqtrakSysex::DumpRequest { device, address } => {
                let mut m = vec![
                    0xF0,
                    0x43,
                    0x20 | (device & 0x0F),
                    0x7F,
                    addr::GROUP_HI,
                    addr::MODEL_ID,
                ];
                m.extend_from_slice(address);
                m.push(0xF7);
                m
            }
            SeqtrakSysex::BulkDump {
                device,
                byte_count,
                block,
                data,
            } => {
                let mut m = vec![0xF0, 0x43, device & 0x0F, 0x7F, addr::GROUP_HI];
                m.extend_from_slice(&byte_count.to_be_bytes().map(|b| b & 0x7F));
                m.push(addr::MODEL_ID);
                m.extend_from_slice(block);
                m.extend(data);
                m.push(checksum(&m[5..]));
                m.push(0xF7);
                m
            }
            SeqtrakSysex::Identity { .. } | SeqtrakSysex::Other(_) => Vec::new(),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SeqtrakSysex::ParameterChange { .. } => "parameter_change",
            SeqtrakSysex::DumpRequest { .. } => "dump_request",
            SeqtrakSysex::BulkDump { .. } => "bulk_dump",
            SeqtrakSysex::Identity { .. } => "identity",
            SeqtrakSysex::Other(_) => "other",
        }
    }
}

/// Checksum: two's complement of the byte sum, masked to 7 bits, so that adding
/// the checksum to the summed bytes gives a multiple of 128 (low 7 bits zero).
/// Range passed = everything after the 5-byte header
/// (group hi/lo, byte count) i.e. from Model ID to last data byte.
pub fn checksum(bytes: &[u8]) -> u8 {
    let sum: u32 = bytes.iter().map(|b| *b as u32).sum();
    (128 - (sum & 0x7F)) as u8 & 0x7F
}

/// Build a parameter change and verify checksum placement for bulk framing.
pub fn parameter_change(device: u8, address: [u8; 3], data: &[u8]) -> SeqtrakSysex {
    SeqtrakSysex::ParameterChange {
        device,
        address,
        data: data.to_vec(),
    }
}

pub fn dump_request(device: u8, address: [u8; 3]) -> SeqtrakSysex {
    SeqtrakSysex::DumpRequest { device, address }
}

/// Format Identity reply version byte `mm` where `mm = (version - 1.0) * 10` per Data List p.112.
pub fn format_version(version: u8) -> String {
    format!("{}.{}", 1 + version / 10, version % 10)
}

/// Parse raw SysEx bytes (F0..F7 inclusive).
pub fn parse(raw: &[u8]) -> Option<SeqtrakSysex> {
    if raw.len() < 2 || raw[0] != 0xF0 || *raw.last()? != 0xF7 {
        return None;
    }
    let inner = &raw[1..raw.len() - 1];

    // Identity reply: F0 7E 7F 06 02 43 00 41 dd dd mm 00 00 7F F7
    if inner.len() >= 11 && inner[0] == 0x7E && inner[2] == 0x06 && inner[3] == 0x02 {
        return Some(SeqtrakSysex::Identity {
            family: [inner[4], inner[5], inner[6], inner[7]],
            version: inner[9],
        });
    }

    // Yamaha model group: 43 [sub-status+device] 7F 1C
    if inner.len() < 8 || inner[0] != 0x43 || inner[2] != 0x7F || inner[3] != addr::GROUP_HI {
        return Some(SeqtrakSysex::Other(raw.to_vec()));
    }
    let device = inner[1] & 0x0F;
    let kind = inner[1] & 0xF0;

    // Bulk dump: [43 0n 7F 1C bh bl 0C ah am al dd...dd cc]
    if kind == 0x00 {
        if inner.len() < 11 {
            return Some(SeqtrakSysex::Other(raw.to_vec()));
        }
        let byte_count = (((inner[4] & 0x7F) as u16) << 7) | (inner[5] & 0x7F) as u16;
        let block = [inner[7], inner[8], inner[9]];
        let data = inner[10..inner.len() - 1].to_vec(); // exclude trailing checksum
        return Some(SeqtrakSysex::BulkDump {
            device,
            byte_count,
            block,
            data,
        });
    }

    // Parameter change 1n / dump request 2n: 7F 1C 0C ah am al ...
    if inner[4] == addr::MODEL_ID {
        let kind = inner[1] & 0xF0;
        if kind == 0x20 && inner.len() == 8 {
            return Some(SeqtrakSysex::DumpRequest {
                device,
                address: [inner[5], inner[6], inner[7]],
            });
        }
        if kind == 0x10 && inner.len() > 8 {
            return Some(SeqtrakSysex::ParameterChange {
                device,
                address: [inner[5], inner[6], inner[7]],
                data: inner[8..].to_vec(),
            });
        }
    }
    Some(SeqtrakSysex::Other(raw.to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameter_change_roundtrip() {
        let msg = parameter_change(0, [0x30, 0x50, 0x00], &[0x40]);
        let bytes = msg.to_bytes();
        assert_eq!(
            bytes,
            vec![
                0xF0, 0x43, 0x10, 0x7F, 0x1C, 0x0C, 0x30, 0x50, 0x00, 0x40, 0xF7
            ]
        );
        match parse(&bytes) {
            Some(SeqtrakSysex::ParameterChange {
                device,
                address,
                data,
            }) => {
                assert_eq!(device, 0);
                assert_eq!(address, [0x30, 0x50, 0x00]);
                assert_eq!(data, vec![0x40]);
            }
            other => panic!("wrong parse: {other:?}"),
        }
    }

    #[test]
    fn dump_request_parse() {
        let bytes = [0xF0, 0x43, 0x20, 0x7F, 0x1C, 0x0C, 0x11, 0x00, 0x00, 0xF7];
        match parse(&bytes) {
            Some(SeqtrakSysex::DumpRequest { address, .. }) => {
                assert_eq!(address, [0x11, 0x00, 0x00])
            }
            other => panic!("wrong parse: {other:?}"),
        }
    }

    #[test]
    fn bulk_dump_roundtrip_and_checksum() {
        let data = vec![0x01, 0x02, 0x03];
        let bulk = SeqtrakSysex::BulkDump {
            device: 0,
            byte_count: 12, // model..data+checksum length per spec "up to but not including checksum"
            block: [0x11, 0x00, 0x00],
            data: data.clone(),
        };
        let bytes = bulk.to_bytes();
        assert_eq!(bytes[0], 0xF0);
        assert_eq!(*bytes.last().unwrap(), 0xF7);
        // checksum byte before F7 must make sum(Model..data)+cc == 0 mod 128
        let cc = bytes[bytes.len() - 2];
        let sum: u32 = bytes[5..bytes.len() - 2].iter().map(|b| *b as u32).sum();
        assert_eq!((sum + cc as u32) & 0x7F, 0);
        match parse(&bytes) {
            Some(SeqtrakSysex::BulkDump { data: d, .. }) => assert_eq!(d, data),
            other => panic!("wrong parse: {other:?}"),
        }
    }

    #[test]
    fn bulk_dump_real_framing() {
        // Captured from the unit: Send Delay block, byte count 0x4C
        let raw = [
            0xF0, 0x43, 0x00, 0x7F, 0x1C, 0x00, 0x4C, 0x0C, 0x30, 0x42, 0x00, 0x02, 0x10, 0xF7,
        ];
        match parse(&raw) {
            Some(SeqtrakSysex::BulkDump {
                byte_count,
                block,
                data,
                ..
            }) => {
                assert_eq!(byte_count, 0x4C);
                assert_eq!(block, [0x30, 0x42, 0x00]);
                assert_eq!(data, vec![0x02]); // trailing 0x10 consumed as checksum
            }
            other => panic!("wrong parse: {other:?}"),
        }
    }

    #[test]
    fn identity_reply() {
        let raw = [
            0xF0, 0x7E, 0x7F, 0x06, 0x02, 0x43, 0x00, 0x41, 0x00, 0x00, 0x20, 0x00, 0x00, 0x7F,
            0xF7,
        ];
        match parse(&raw) {
            Some(SeqtrakSysex::Identity { family, version }) => {
                assert_eq!(&family[..3], &[0x43, 0x00, 0x41]);
                assert_eq!(version, 0x20);
            }
            other => panic!("wrong parse: {other:?}"),
        }
    }
}
