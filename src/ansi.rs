use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
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
    pub fn from(seq: String) -> Option<Self> {
        let pieces: Vec<u8> = seq
            .split(';')
            .map(|n| n.parse::<u8>())
            .collect::<Result<Vec<u8>, _>>()
            .ok()?;

        match pieces.len() {
            1 => match pieces[0] {
                0 => Some(ANSISequence::Reset),
                1 => Some(ANSISequence::Bold),
                3 => Some(ANSISequence::Italic),
                4 => Some(ANSISequence::Underline),
                22 => Some(ANSISequence::NotBold),
                23 => Some(ANSISequence::NotItalic),
                24 => Some(ANSISequence::NotUnderline),
                // https://en.wikipedia.org/wiki/ANSI_escape_code#3-bit_and_4-bit
                30..=37 => Some(ANSISequence::SetFG8(pieces[0] - 30)), // 30-37 are the 4bit colors
                39 => Some(ANSISequence::DefaultFG),
                40..=47 => Some(ANSISequence::SetBG8(pieces[0] - 40)), // 40-47 are the 4bit colors
                49 => Some(ANSISequence::DefaultBG),
                90..=97 => Some(ANSISequence::SetFG8(pieces[0] - 90 + 8)), // 90-97 are the 4bit high intensity
                100..=107 => Some(ANSISequence::SetBG8(pieces[0] - 100 + 8)), // 100-107 are the 4bit high intensity
                _ => None,
            },
            // https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit
            3 => match pieces[..] {
                [38, 5, 0..=255] => Some(ANSISequence::SetFG8(pieces[2])),
                [48, 5, 0..=255] => Some(ANSISequence::SetBG8(pieces[2])),
                _ => None,
            },
            // https://en.wikipedia.org/wiki/ANSI_escape_code#24-bit
            5 => match pieces[..] {
                [38, 2, 0..=255, 0..=255, 0..=255] => {
                    Some(ANSISequence::SetFG24(pieces[2], pieces[3], pieces[4]))
                }
                [48, 2, 0..=255, 0..=255, 0..=255] => {
                    Some(ANSISequence::SetBG24(pieces[2], pieces[3], pieces[4]))
                }
                _ => None,
            },
            _ => None,
        }
    }
}

pub fn extract_ansi(raw: String) -> (String, Vec<(ANSISequence, usize)>) {
    let mut scrubbed = String::new();
    scrubbed.reserve(raw.len());
    let mut ansi_seqs = Vec::new();

    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        match (ch, chars.peek()) {
            // Matches start of ESC[<seq>m
            ('\x1b', Some('[')) => {
                chars.next();
                let mut acc = String::new();
                let mut seq: Option<ANSISequence> = None;

                // Read until we find 'm' or run out of chars
                loop {
                    match chars.next() {
                        Some('m') => {
                            seq = ANSISequence::from(acc.clone());
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

                match seq {
                    // Found a valid sequence, push & mark the index
                    Some(seq) => {
                        ansi_seqs.push((seq, scrubbed.len()));
                    }
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

    (scrubbed, ansi_seqs)
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
            vec![(ANSISequence::Reset, 0), (ANSISequence::Reset, 5)],
        );
        assert_eq!(want.0, got.0);
        assert!(want.1.iter().eq(got.1.iter()));
    }

    #[test]
    fn bold() {
        let raw = "\u{1b}[1mbold\u{1b}[22m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("bold"),
            vec![(ANSISequence::Bold, 0), (ANSISequence::NotBold, 4)],
        );
        assert_eq!(want.0, got.0);
        assert!(want.1.iter().eq(got.1.iter()));
    }

    #[test]
    fn italic() {
        let raw = "\u{1b}[3mitalic\u{1b}[23m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("italic"),
            vec![(ANSISequence::Italic, 0), (ANSISequence::NotItalic, 6)],
        );
        assert_eq!(want.0, got.0);
        assert!(want.1.iter().eq(got.1.iter()));
    }

    #[test]
    fn underline() {
        let raw = "\u{1b}[4munderline\u{1b}[24m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("underline"),
            vec![
                (ANSISequence::Underline, 0),
                (ANSISequence::NotUnderline, 9),
            ],
        );
        assert_eq!(want.0, got.0);
        assert!(want.1.iter().eq(got.1.iter()));
    }

    #[test]
    fn color_4bit_fg() {
        let raw = "\u{1b}[30m\u{1b}[31m\u{1b}[32m\u{1b}[33m\u{1b}[34m\u{1b}[35m\u{1b}[36m\u{1b}[37m4bit-colors\u{1b}[39m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("4bit-colors"),
            vec![
                (ANSISequence::SetFG8(0), 0),
                (ANSISequence::SetFG8(1), 0),
                (ANSISequence::SetFG8(2), 0),
                (ANSISequence::SetFG8(3), 0),
                (ANSISequence::SetFG8(4), 0),
                (ANSISequence::SetFG8(5), 0),
                (ANSISequence::SetFG8(6), 0),
                (ANSISequence::SetFG8(7), 0),
                (ANSISequence::DefaultFG, 11),
            ],
        );
        assert_eq!(want.0, got.0);
        assert!(want.1.iter().eq(got.1.iter()));
    }

    #[test]
    fn color_4bit_bg() {
        let raw = "\u{1b}[40m\u{1b}[41m\u{1b}[42m\u{1b}[43m\u{1b}[44m\u{1b}[45m\u{1b}[46m\u{1b}[47m4bit-colors\u{1b}[49m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("4bit-colors"),
            vec![
                (ANSISequence::SetBG8(0), 0),
                (ANSISequence::SetBG8(1), 0),
                (ANSISequence::SetBG8(2), 0),
                (ANSISequence::SetBG8(3), 0),
                (ANSISequence::SetBG8(4), 0),
                (ANSISequence::SetBG8(5), 0),
                (ANSISequence::SetBG8(6), 0),
                (ANSISequence::SetBG8(7), 0),
                (ANSISequence::DefaultBG, 11),
            ],
        );
        assert_eq!(want.0, got.0);
        assert!(want.1.iter().eq(got.1.iter()));
    }

    #[test]
    fn color_4bit_hi_fg() {
        let raw = "\u{1b}[90m\u{1b}[91m\u{1b}[92m\u{1b}[93m\u{1b}[94m\u{1b}[95m\u{1b}[96m\u{1b}[97m4bit-colors high intensity\u{1b}[39m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("4bit-colors high intensity"),
            vec![
                (ANSISequence::SetFG8(8), 0),
                (ANSISequence::SetFG8(9), 0),
                (ANSISequence::SetFG8(10), 0),
                (ANSISequence::SetFG8(11), 0),
                (ANSISequence::SetFG8(12), 0),
                (ANSISequence::SetFG8(13), 0),
                (ANSISequence::SetFG8(14), 0),
                (ANSISequence::SetFG8(15), 0),
                (ANSISequence::DefaultFG, 26),
            ],
        );
        assert_eq!(want.0, got.0);
        assert!(want.1.iter().eq(got.1.iter()));
    }

    #[test]
    fn color_4bit_hi_bg() {
        let raw = "\u{1b}[100m\u{1b}[101m\u{1b}[102m\u{1b}[103m\u{1b}[104m\u{1b}[105m\u{1b}[106m\u{1b}[107m4bit-colors high intensity\u{1b}[49m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("4bit-colors high intensity"),
            vec![
                (ANSISequence::SetBG8(8), 0),
                (ANSISequence::SetBG8(9), 0),
                (ANSISequence::SetBG8(10), 0),
                (ANSISequence::SetBG8(11), 0),
                (ANSISequence::SetBG8(12), 0),
                (ANSISequence::SetBG8(13), 0),
                (ANSISequence::SetBG8(14), 0),
                (ANSISequence::SetBG8(15), 0),
                (ANSISequence::DefaultBG, 26),
            ],
        );
        assert_eq!(want.0, got.0);
        assert!(want.1.iter().eq(got.1.iter()));
    }

    #[test]
    fn color_8bit_fg() {
        let raw = "\u{1b}[38;5;111m8-bit\u{1b}[0m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("8-bit"),
            vec![(ANSISequence::SetFG8(111), 0), (ANSISequence::Reset, 5)],
        );
        assert_eq!(want.0, got.0);
        assert!(want.1.iter().eq(got.1.iter()));
    }

    #[test]
    fn color_8bit_bg() {
        let raw = "\u{1b}[48;5;111m8-bit\u{1b}[0m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("8-bit"),
            vec![(ANSISequence::SetBG8(111), 0), (ANSISequence::Reset, 5)],
        );
        assert_eq!(want.0, got.0);
        assert!(want.1.iter().eq(got.1.iter()));
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
            vec![
                (ANSISequence::SetFG24(100, 110, 111), 0),
                (ANSISequence::Reset, 6),
            ],
        );
        assert_eq!(want.0, got.0);
        assert!(want.1.iter().eq(got.1.iter()));
    }

    #[test]
    fn color_24bit_bg() {
        let raw = "\u{1b}[48;2;100;110;111m24-bit\u{1b}[0m";
        let got = extract_ansi(raw.to_string());
        let want = (
            String::from("24-bit"),
            vec![
                (ANSISequence::SetBG24(100, 110, 111), 0),
                (ANSISequence::Reset, 6),
            ],
        );
        assert_eq!(want.0, got.0);
        assert!(want.1.iter().eq(got.1.iter()));
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
        let raw = "\u{1b}[1337minvalid\u{1b}[0;0;0;0;0;0;0;0;0;0;0;0;0mwithout an m:\u{1b}[0";
        let got = extract_ansi(raw.to_string());
        assert_eq!(raw, got.0);
        assert!(got.1.is_empty());
    }
}
