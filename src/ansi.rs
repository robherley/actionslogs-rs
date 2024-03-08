use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize)]
pub enum ANSISequence {
    Reset,
    Bold,
    Italic,
    Underline,
    NotBold,
    NotItalic,
    NotUnderline,
    SetFG8(u8),
    DefaultFG,
    SetBG8(u8),
    DefaultBG,
    SetFG24(u8, u8, u8),
    SetBG24(u8, u8, u8),
}

impl ANSISequence {
    fn match_seqs(mut seq: Vec<u8>) -> (Option<Self>, Vec<u8>) {
        if seq.is_empty() {
            return (None, seq);
        }

        let matched = match seq[0] {
            0 => Some((ANSISequence::Reset, 1)),
            1 => Some((ANSISequence::Bold, 1)),
            3 => Some((ANSISequence::Italic, 1)),
            4 => Some((ANSISequence::Underline, 1)),
            22 => Some((ANSISequence::NotBold, 1)),
            23 => Some((ANSISequence::NotItalic, 1)),
            24 => Some((ANSISequence::NotUnderline, 1)),
            // https://en.wikipedia.org/wiki/ANSI_escape_code#3-bit_and_4-bit
            30..=37 => Some((ANSISequence::SetFG8(seq[0] - 30), 1)), // 30-37 are the 4bit colors
            38 => match (seq.get(1), seq.get(2), seq.get(3), seq.get(4)) {
                (Some(5), Some(0..=255), None, None) => Some((ANSISequence::SetFG8(seq[2]), 3)),
                (Some(2), Some(0..=255), Some(0..=255), Some(0..=255)) => {
                    Some((ANSISequence::SetFG24(seq[2], seq[3], seq[4]), 5))
                }
                _ => None,
            },
            39 => Some((ANSISequence::DefaultFG, 1)),
            40..=47 => Some((ANSISequence::SetBG8(seq[0] - 40), 1)), // 40-47 are the 4bit colors
            48 => match (seq.get(1), seq.get(2), seq.get(3), seq.get(4)) {
                (Some(5), Some(0..=255), None, None) => Some((ANSISequence::SetBG8(seq[2]), 3)),
                (Some(2), Some(0..=255), Some(0..=255), Some(0..=255)) => {
                    Some((ANSISequence::SetBG24(seq[2], seq[3], seq[4]), 5))
                }
                _ => None,
            },
            49 => Some((ANSISequence::DefaultBG, 1)),
            90..=97 => Some((ANSISequence::SetFG8(seq[0] - 90 + 8), 1)), // 90-97 are the 4bit high intensity
            100..=107 => Some((ANSISequence::SetBG8(seq[0] - 100 + 8), 1)), // 100-107 are the 4bit high intensity
            _ => None,
        };

        match matched {
            Some((ansi, len)) => (Some(ansi), seq.split_off(len)),
            None => (None, seq),
        }
    }

    pub fn from(seq: String) -> Option<Vec<Self>> {
        let mut possible_seqs: Vec<u8> = seq
            .split(';')
            .map(|n| n.parse::<u8>())
            .collect::<Result<Vec<u8>, _>>()
            .ok()?;

        let mut seqs = Vec::new();
        while !possible_seqs.is_empty() {
            let (matched, rest) = ANSISequence::match_seqs(possible_seqs);
            match matched {
                Some(seq) => seqs.push(seq),
                // if any part of the sequence fails to match treat the whole thing as invalid
                None => return None,
            }
            possible_seqs = rest;
        }

        Some(seqs)
    }
}

pub fn extract_ansi(raw: String) -> (String, HashMap<usize, Vec<ANSISequence>>) {
    let mut scrubbed = String::new();
    scrubbed.reserve(raw.len());
    let mut ansi_map: HashMap<usize, Vec<ANSISequence>> = HashMap::new();

    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        match (ch, chars.peek()) {
            // Matches start of ESC[<seq>m
            ('\x1b', Some('[')) => {
                chars.next();
                let mut acc = String::new();
                let mut seqs: Option<Vec<ANSISequence>> = None;

                // Read until we find 'm' or run out of chars
                loop {
                    match chars.next() {
                        Some('m') => {
                            seqs = ANSISequence::from(acc.clone());
                            acc.push('m');
                            break;
                        }
                        Some(ch) => {
                            acc.push(ch);
                        }
                        None => {
                            break;
                        }
                    }
                }

                match seqs {
                    // Found a valid sequence, push & mark the index
                    Some(seqs) => match ansi_map.get_mut(&scrubbed.len()) {
                        Some(existing) => existing.extend(seqs),
                        None => {
                            ansi_map.insert(scrubbed.len(), seqs);
                        }
                    },
                    // Nothing found just push what we've seen
                    None => {
                        scrubbed.push_str("\x1b[");
                        scrubbed.push_str(&acc);
                    }
                }
            }
            // No match, just push the char
            (_, _) => {
                scrubbed.push(ch);
            }
        }
    }

    (scrubbed, ansi_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset() {
        let raw = "\u{1b}[0mreset\u{1b}[0m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("reset"),
            HashMap::from([
                (0, vec![ANSISequence::Reset]),
                (5, vec![ANSISequence::Reset]),
            ]),
        );
        assert_eq!(want.0, got.0);
        assert_eq!(want.1, got.1);
    }

    #[test]
    fn bold() {
        let raw = "\u{1b}[1mbold\u{1b}[22m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("bold"),
            HashMap::from([
                (0, vec![ANSISequence::Bold]),
                (4, vec![ANSISequence::NotBold]),
            ]),
        );
        assert_eq!(want.0, got.0);
        assert_eq!(want.1, got.1);
    }

    #[test]
    fn italic() {
        let raw = "\u{1b}[3mitalic\u{1b}[23m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("italic"),
            HashMap::from([
                (0, vec![ANSISequence::Italic]),
                (6, vec![ANSISequence::NotItalic]),
            ]),
        );
        assert_eq!(want.0, got.0);
        assert_eq!(want.1, got.1);
    }

    #[test]
    fn underline() {
        let raw = "\u{1b}[4munderline\u{1b}[24m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("underline"),
            HashMap::from([
                (0, vec![ANSISequence::Underline]),
                (9, vec![ANSISequence::NotUnderline]),
            ]),
        );
        assert_eq!(want.0, got.0);
        assert_eq!(want.1, got.1);
    }

    #[test]
    fn color_4bit_fg() {
        let raw = "\u{1b}[30m\u{1b}[31m\u{1b}[32m\u{1b}[33m\u{1b}[34m\u{1b}[35m\u{1b}[36m\u{1b}[37m4bit-colors\u{1b}[39m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("4bit-colors"),
            HashMap::from([
                (
                    0,
                    vec![
                        ANSISequence::SetFG8(0),
                        ANSISequence::SetFG8(1),
                        ANSISequence::SetFG8(2),
                        ANSISequence::SetFG8(3),
                        ANSISequence::SetFG8(4),
                        ANSISequence::SetFG8(5),
                        ANSISequence::SetFG8(6),
                        ANSISequence::SetFG8(7),
                    ],
                ),
                (11, vec![ANSISequence::DefaultFG]),
            ]),
        );
        assert_eq!(want.0, got.0);
        assert_eq!(want.1, got.1);
    }

    #[test]
    fn color_4bit_bg() {
        let raw = "\u{1b}[40m\u{1b}[41m\u{1b}[42m\u{1b}[43m\u{1b}[44m\u{1b}[45m\u{1b}[46m\u{1b}[47m4bit-colors\u{1b}[49m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("4bit-colors"),
            HashMap::from([
                (
                    0,
                    vec![
                        ANSISequence::SetBG8(0),
                        ANSISequence::SetBG8(1),
                        ANSISequence::SetBG8(2),
                        ANSISequence::SetBG8(3),
                        ANSISequence::SetBG8(4),
                        ANSISequence::SetBG8(5),
                        ANSISequence::SetBG8(6),
                        ANSISequence::SetBG8(7),
                    ],
                ),
                (11, vec![ANSISequence::DefaultBG]),
            ]),
        );

        assert_eq!(want.0, got.0);
        assert_eq!(want.1, got.1);
    }

    #[test]
    fn color_4bit_hi_fg() {
        let raw = "\u{1b}[90m\u{1b}[91m\u{1b}[92m\u{1b}[93m\u{1b}[94m\u{1b}[95m\u{1b}[96m\u{1b}[97m4bit-colors high intensity\u{1b}[39m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("4bit-colors high intensity"),
            HashMap::from([
                (
                    0,
                    vec![
                        ANSISequence::SetFG8(8),
                        ANSISequence::SetFG8(9),
                        ANSISequence::SetFG8(10),
                        ANSISequence::SetFG8(11),
                        ANSISequence::SetFG8(12),
                        ANSISequence::SetFG8(13),
                        ANSISequence::SetFG8(14),
                        ANSISequence::SetFG8(15),
                    ],
                ),
                (26, vec![ANSISequence::DefaultFG]),
            ]),
        );

        assert_eq!(want.0, got.0);
        assert_eq!(want.1, got.1);
    }

    #[test]
    fn color_4bit_hi_bg() {
        let raw = "\u{1b}[100m\u{1b}[101m\u{1b}[102m\u{1b}[103m\u{1b}[104m\u{1b}[105m\u{1b}[106m\u{1b}[107m4bit-colors high intensity\u{1b}[49m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("4bit-colors high intensity"),
            HashMap::from([
                (
                    0,
                    vec![
                        ANSISequence::SetBG8(8),
                        ANSISequence::SetBG8(9),
                        ANSISequence::SetBG8(10),
                        ANSISequence::SetBG8(11),
                        ANSISequence::SetBG8(12),
                        ANSISequence::SetBG8(13),
                        ANSISequence::SetBG8(14),
                        ANSISequence::SetBG8(15),
                    ],
                ),
                (26, vec![ANSISequence::DefaultBG]),
            ]),
        );

        assert_eq!(want.0, got.0);
        assert_eq!(want.1, got.1);
    }

    #[test]
    fn color_8bit_fg() {
        let raw = "\u{1b}[38;5;111m8-bit\u{1b}[0m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("8-bit"),
            HashMap::from([
                (0, vec![ANSISequence::SetFG8(111)]),
                (5, vec![ANSISequence::Reset]),
            ]),
        );

        assert_eq!(want.0, got.0);
        assert_eq!(want.1, got.1);
    }

    #[test]
    fn color_8bit_bg() {
        let raw = "\u{1b}[48;5;111m8-bit\u{1b}[0m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("8-bit"),
            HashMap::from([
                (0, vec![ANSISequence::SetBG8(111)]),
                (5, vec![ANSISequence::Reset]),
            ]),
        );

        assert_eq!(want.0, got.0);
        assert_eq!(want.1, got.1);
    }

    #[test]
    fn color_8bit_invalid() {
        let raw = "\u{1b}[38;5;256m\u{1b}[48;5;256minvalid";
        let got = extract_ansi(raw.to_string());
        assert_eq!(raw, got.0);
        assert!(got.1.is_empty());
    }

    #[test]
    fn color_24bit_fg() {
        let raw = "\u{1b}[38;2;100;110;111m24-bit\u{1b}[0m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("24-bit"),
            HashMap::from([
                (0, vec![ANSISequence::SetFG24(100, 110, 111)]),
                (6, vec![ANSISequence::Reset]),
            ]),
        );

        assert_eq!(want.0, got.0);
        assert_eq!(want.1, got.1);
    }

    #[test]
    fn color_24bit_bg() {
        let raw = "\u{1b}[48;2;100;110;111m24-bit\u{1b}[0m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("24-bit"),
            HashMap::from([
                (0, vec![ANSISequence::SetBG24(100, 110, 111)]),
                (6, vec![ANSISequence::Reset]),
            ]),
        );

        assert_eq!(want.0, got.0);
        assert_eq!(want.1, got.1);
    }

    #[test]
    fn color_24bit_invalid() {
        let raw = "\u{1b}[38;2;256;100;100m\u{1b}[48;2;256;100;100minvalid";
        let got = extract_ansi(raw.to_string());
        assert_eq!(raw, got.0);
        assert!(got.1.is_empty());
    }

    #[test]
    fn invalid_junk() {
        let raw = "\u{1b}[1337minvalid\u{1b}[1337;1337;1337;1337mwithout an m:\u{1b}[0";
        let got = extract_ansi(raw.to_string());
        assert_eq!(raw, got.0);
        assert!(got.1.is_empty());
    }

    #[test]
    fn multi_seq() {
        let raw = "\u{1b}[36;1mbold cyan\u{1b}[0m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("bold cyan"),
            HashMap::from([
                (0, vec![ANSISequence::SetFG8(6), ANSISequence::Bold]),
                (9, vec![ANSISequence::Reset]),
            ]),
        );

        assert_eq!(want.0, got.0);
        assert_eq!(want.1, got.1);
    }
}
