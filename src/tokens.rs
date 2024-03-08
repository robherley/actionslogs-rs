use crate::ansi::ANSISequence;

pub enum Tokens {
    Text(String),
    ANSI(ANSISequence),
    StartLink(String),
    EndLink,
    StartHighlight,
    EndHighlight,
}
