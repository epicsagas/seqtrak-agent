//! SEQTRAK static protocol tables: channels, CC map, banks, SysEx addresses.
//! Source: Yamaha SEQTRAK Data List for OS V2.00 (YJ-D0, 01/2026), available from Yamaha.

/// Fixed track channels. The Data List: Basic Channel 1-11, no channel change.
pub const TRACKS: [(&str, u8); 11] = [
    ("kick", 0),
    ("snare", 1),
    ("clap", 2),
    ("hat1", 3),
    ("hat2", 4),
    ("perc1", 5),
    ("perc2", 6),
    ("synth1", 7),
    ("synth2", 8),
    ("dx", 9),
    ("sampler", 10),
];

pub fn track_channel(name: &str) -> Option<u8> {
    let lower = name.to_ascii_lowercase();
    TRACKS
        .iter()
        .find(|(n, _)| *n == lower)
        .map(|(_, ch)| *ch)
        .or_else(|| {
            // allow "1".."11" as 1-based channel numbers
            lower
                .parse::<u8>()
                .ok()
                .filter(|c| (1..=11).contains(c))
                .map(|c| c - 1)
        })
}

/// Named CC map. Model-specific numbers per Data List p.113-114.
pub const CC_TABLE: [(&str, u8); 29] = [
    ("volume", 7),
    ("pan", 10),
    ("eq_high", 20),
    ("eq_low", 21),
    ("mute", 23),
    ("solo", 24),
    ("drum_pitch", 25),
    ("mono_poly", 26),
    ("arp_template", 27),
    ("arp_gate", 28),
    ("arp_speed", 29),
    ("portamento_time", 5),
    ("portamento_switch", 65),
    ("filter_resonance", 71),
    ("eg_attack", 73),
    ("filter_cutoff", 74),
    ("eg_decay_release", 75),
    ("reverb_send", 91),
    ("delay_send", 94),
    ("fx_master_1", 102),
    ("fx_master_2", 103),
    ("fx_master_3", 104),
    ("fx_single_1", 107),
    ("fx_single_2", 108),
    ("fx_single_3", 109),
    ("fm_algorithm", 116),
    ("fm_mod_amount", 117),
    ("fm_mod_frequency", 118),
    ("fm_mod_feedback", 119),
];

pub fn cc_number(name: &str) -> Option<u8> {
    if let Ok(n) = name.parse::<u8>() {
        return Some(n);
    }
    CC_TABLE.iter().find(|(n, _)| *n == name).map(|(_, c)| *c)
}

pub fn cc_name(num: u8) -> &'static str {
    CC_TABLE
        .iter()
        .find(|(_, c)| *c == num)
        .map(|(n, _)| *n)
        .unwrap_or("unknown")
}

/// SysEx top-level address nibbles.
pub mod addr {
    /// Parameter Change: F0 43 1n 7F 1C 0C ah am al dd... F7
    pub const MODEL_ID: u8 = 0x0C;
    /// Group ID High/Low pair sent before Model ID.
    pub const GROUP_HI: u8 = 0x1C;

    pub const BULK_HEADER: (u8, u8, u8) = (0x11, 0x00, 0x00);
    pub const BULK_FOOTER: (u8, u8, u8) = (0x12, 0x00, 0x00);
    pub const SYSTEM_GENERAL: (u8, u8, u8) = (0x00, 0x00, 0x00);
    pub const FORMAT_VERSION: (u8, u8, u8) = (0x00, 0x7F, 0x00);
    /// Project Common blocks: Send Reverb .. Project Track, Chord notes.
    pub const SEND_REVERB: (u8, u8, u8) = (0x30, 0x41, 0x00);
    pub const SEND_DELAY: (u8, u8, u8) = (0x30, 0x42, 0x00);
    pub const MASTER_EQ: (u8, u8, u8) = (0x30, 0x47, 0x00);
    /// Project Track General for part p: 30 5p 00
    pub const fn project_track(part: u8) -> (u8, u8, u8) {
        (0x30, 0x50 + part, 0x00)
    }
    /// Part Common General for part p: 31 1p 00
    pub const fn part_general(part: u8) -> (u8, u8, u8) {
        (0x31, 0x10 + part, 0x00)
    }
    /// Part name for part p: 31 0p 00 (7-bit encoded UTF-8)
    pub const fn part_name(part: u8) -> (u8, u8, u8) {
        (0x31, part, 0x00)
    }
    /// Part Insertion A: 31 2p 00 / Insertion B: 31 3p 00
    pub const fn part_ins_a(part: u8) -> (u8, u8, u8) {
        (0x31, 0x20 + part, 0x00)
    }
    pub const fn part_ins_b(part: u8) -> (u8, u8, u8) {
        (0x31, 0x30 + part, 0x00)
    }
    /// Sound edit buffer LFO for part p: 31 4p 00
    pub const fn sound_lfo(part: u8) -> (u8, u8, u8) {
        (0x31, 0x40 + part, 0x00)
    }
}

/// Bank select targets. Data List p.117-118.
#[derive(Debug, Clone, Copy)]
pub enum Bank {
    /// Project User 1-8 (LSB 0-7). Load executes on Program Change, channel ignored.
    Project {
        user: u8,
    },
    /// Sound Preset 1-32 (LSB 0-31) / User 1-16 (LSB 32-47)
    SoundPreset {
        lsb: u8,
    },
    SoundUser {
        lsb: u8,
    },
    /// Sampler Element Preset/User (LSB 0-11)
    SamplerElement {
        lsb: u8,
    },
    /// DrumKit for drum track part 0-6 (MSB 32-38, LSB = 2p where p = part)
    DrumKit {
        part: u8,
    },
}

impl Bank {
    /// Returns (msb, lsb) or None if out of range.
    pub fn msb_lsb(self) -> Option<(u8, u8)> {
        match self {
            Bank::Project { user } if user <= 7 => Some((64, user)),
            Bank::SoundPreset { lsb } if lsb <= 31 => Some((63, lsb)),
            Bank::SoundUser { lsb } if (32..=47).contains(&lsb) => Some((63, lsb)),
            Bank::SamplerElement { lsb } if lsb <= 11 => Some((62, lsb)),
            Bank::DrumKit { part } if part <= 6 => Some((32 + part, part * 2)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_lookup() {
        assert_eq!(track_channel("kick"), Some(0));
        assert_eq!(track_channel("synth1"), Some(7));
        assert_eq!(track_channel("dx"), Some(9));
        assert_eq!(track_channel("11"), Some(10));
        assert_eq!(track_channel("bogus"), None);
    }

    #[test]
    fn cc_lookup() {
        assert_eq!(cc_number("volume"), Some(7));
        assert_eq!(cc_number("fx_master_1"), Some(102));
        assert_eq!(cc_number("fx_single_1"), Some(107));
        assert_eq!(cc_number("fm_mod_frequency"), Some(118));
        assert_eq!(cc_number("74"), Some(74));
        assert_eq!(cc_name(24), "solo");
        assert_eq!(cc_name(102), "fx_master_1");
    }

    #[test]
    fn banks() {
        assert_eq!(Bank::Project { user: 0 }.msb_lsb(), Some((64, 0)));
        assert_eq!(Bank::Project { user: 8 }.msb_lsb(), None);
        assert_eq!(Bank::SoundUser { lsb: 32 }.msb_lsb(), Some((63, 32)));
        assert_eq!(Bank::DrumKit { part: 1 }.msb_lsb(), Some((33, 2)));
    }
}
