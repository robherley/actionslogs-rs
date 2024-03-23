use serde::ser::{SerializeTuple, Serializer};
use serde::Serialize;

use crate::ansi::ANSISequence;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    Bit8(u8),
    Bit24(u8, u8, u8),
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Color::Bit8(value) => serializer.serialize_u8(value),
            Color::Bit24(r, g, b) => {
                let mut tuple = serializer.serialize_tuple(3)?;
                tuple.serialize_element(&r)?;
                tuple.serialize_element(&g)?;
                tuple.serialize_element(&b)?;
                tuple.end()
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Styles {
    #[serde(rename = "b", skip_serializing_if = "std::ops::Not::not")]
    pub bold: bool,
    #[serde(rename = "i", skip_serializing_if = "std::ops::Not::not")]
    pub italic: bool,
    #[serde(rename = "u", skip_serializing_if = "std::ops::Not::not")]
    pub underline: bool,
    #[serde(rename = "hl", skip_serializing_if = "std::ops::Not::not")]
    pub highlight: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fg: Option<Color>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<Color>,
}

impl Styles {
    pub fn new() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: false,
            highlight: false,
            fg: None,
            bg: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        !self.bold
            && !self.italic
            && !self.underline
            && !self.highlight
            && self.fg.is_none()
            && self.bg.is_none()
    }

    pub fn apply_ansis(&mut self, ansis: &Vec<ANSISequence>) {
        for ansi in ansis {
            self.apply_ansi(ansi);
        }
    }

    pub fn apply_ansi(&mut self, ansi: &ANSISequence) {
        match ansi {
            ANSISequence::Reset => {
                self.bold = false;
                self.italic = false;
                self.underline = false;
                self.fg = None;
                self.bg = None;
            }
            ANSISequence::Bold => self.bold = true,
            ANSISequence::Italic => self.italic = true,
            ANSISequence::Underline => self.underline = true,
            ANSISequence::NotBold => self.bold = false,
            ANSISequence::NotItalic => self.italic = false,
            ANSISequence::NotUnderline => self.underline = false,
            ANSISequence::SetFG8(color) => self.fg = Some(Color::Bit8(*color)),
            ANSISequence::DefaultFG => self.fg = None,
            ANSISequence::SetBG8(color) => self.bg = Some(Color::Bit8(*color)),
            ANSISequence::DefaultBG => self.bg = None,
            ANSISequence::SetFG24(r, g, b) => self.fg = Some(Color::Bit24(*r, *g, *b)),
            ANSISequence::SetBG24(r, g, b) => self.bg = Some(Color::Bit24(*r, *g, *b)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_empty() {
        let mut styles = Styles::new();
        assert!(styles.is_empty());
        styles.bold = true;
        assert!(!styles.is_empty());
    }

    #[test]
    fn apply_ansi() {
        let cases = vec![
            (ANSISequence::Reset, Styles::new()),
            (
                ANSISequence::Bold,
                Styles {
                    bold: true,
                    ..Styles::new()
                },
            ),
            (
                ANSISequence::Italic,
                Styles {
                    italic: true,
                    ..Styles::new()
                },
            ),
            (
                ANSISequence::Underline,
                Styles {
                    underline: true,
                    ..Styles::new()
                },
            ),
            (
                ANSISequence::NotBold,
                Styles {
                    bold: false,
                    ..Styles::new()
                },
            ),
            (
                ANSISequence::NotItalic,
                Styles {
                    italic: false,
                    ..Styles::new()
                },
            ),
            (
                ANSISequence::NotUnderline,
                Styles {
                    underline: false,
                    ..Styles::new()
                },
            ),
            (
                ANSISequence::SetFG8(1),
                Styles {
                    fg: Some(Color::Bit8(1)),
                    ..Styles::new()
                },
            ),
            (
                ANSISequence::DefaultFG,
                Styles {
                    fg: None,
                    ..Styles::new()
                },
            ),
            (
                ANSISequence::SetBG8(1),
                Styles {
                    bg: Some(Color::Bit8(1)),
                    ..Styles::new()
                },
            ),
            (
                ANSISequence::DefaultBG,
                Styles {
                    bg: None,
                    ..Styles::new()
                },
            ),
            (
                ANSISequence::SetFG24(1, 2, 3),
                Styles {
                    fg: Some(Color::Bit24(1, 2, 3)),
                    ..Styles::new()
                },
            ),
            (
                ANSISequence::SetBG24(1, 2, 3),
                Styles {
                    bg: Some(Color::Bit24(1, 2, 3)),
                    ..Styles::new()
                },
            ),
        ];

        for (ansi, expected) in cases {
            let mut styles = Styles::new();
            styles.apply_ansi(&ansi);
            assert_eq!(styles, expected);
        }
    }

    #[test]
    fn resetters() {
        let cases = vec![
            (
                ANSISequence::Reset,
                Styles {
                    bold: true,
                    italic: true,
                    underline: true,
                    fg: Some(Color::Bit8(1)),
                    bg: Some(Color::Bit8(2)),
                    ..Styles::new()
                },
            ),
            (
                ANSISequence::NotBold,
                Styles {
                    bold: true,
                    ..Styles::new()
                },
            ),
            (
                ANSISequence::NotItalic,
                Styles {
                    italic: true,
                    ..Styles::new()
                },
            ),
            (
                ANSISequence::NotUnderline,
                Styles {
                    underline: true,
                    ..Styles::new()
                },
            ),
            (
                ANSISequence::DefaultFG,
                Styles {
                    fg: Some(Color::Bit8(1)),
                    ..Styles::new()
                },
            ),
            (
                ANSISequence::DefaultBG,
                Styles {
                    bg: Some(Color::Bit8(1)),
                    ..Styles::new()
                },
            ),
        ];

        for (ansi, before) in cases {
            let mut styles = before.clone();
            styles.apply_ansi(&ansi);
            assert_eq!(styles, Styles::new());
        }
    }

    #[test]
    fn does_not_reset_highlight() {
        let mut styles = Styles::new();
        styles.highlight = true;
        styles.apply_ansi(&ANSISequence::Reset);
        assert_eq!(
            styles,
            Styles {
                highlight: true,
                ..Styles::new()
            }
        );
    }

    #[test]
    fn serialize() {
        let cases = vec![
            (Styles::new(), r#"{}"#),
            (
                Styles {
                    bold: true,
                    italic: true,
                    underline: true,
                    highlight: true,
                    fg: Some(Color::Bit8(1)),
                    bg: Some(Color::Bit24(1, 2, 3)),
                },
                r#"{"b":true,"i":true,"u":true,"hl":true,"fg":1,"bg":[1,2,3]}"#,
            ),
        ];

        for (styles, expected) in cases {
            let serialized = serde_json::to_string(&styles).unwrap();
            assert_eq!(serialized, expected);
        }
    }
}
