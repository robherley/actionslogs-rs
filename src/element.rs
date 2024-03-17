use serde::Serialize;

use crate::log::Line;
use crate::style::Styles;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum Element {
    // Text(content, styles)
    Text(String, Styles),
    // Link(href, children)
    Link(String, Vec<Element>),
}

// Builder contructs renderable elements from a line
struct Builder {
    // output elements
    pub elements: Vec<Element>,
    // sub elements accumulator for a link
    link_elements: Vec<Element>,
    // text accumulator
    text: String,
    // current styles
    styles: Styles,
    // if currently highlighting a word, the end index of the highlight
    end_highlight_idx: Option<usize>,
    // if currently within a link, the end index of the link
    end_link_idx: Option<usize>,
    // if currently within a link, the href of the link
    link_href: Option<String>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            link_elements: Vec::new(),
            text: String::new(),
            styles: Styles::new(),
            end_highlight_idx: None,
            end_link_idx: None,
            link_href: None,
        }
    }

    pub fn elements_from(&mut self, line: &Line) {
        for (i, ch) in line.content.char_indices() {
            let mut new_styles = self.styles.clone();

            // starting a link
            if let Some(end_idx) = line.links.get(&i) {
                self.flush();
                self.start_link(*end_idx, line.content[i..*end_idx].to_string());
            }

            // ending a link
            if let Some(end_idx) = self.end_link_idx {
                if i == end_idx {
                    self.flush();
                    self.end_link();
                }
            }

            // starting a highlight
            if let Some(end_idx) = line.highlights.get(&i) {
                new_styles.highlight = true;
                self.end_highlight_idx = Some(*end_idx);
            }

            // ending a highlight
            if let Some(end_idx) = self.end_highlight_idx {
                if i == end_idx {
                    new_styles.highlight = false;
                    self.end_highlight_idx = None;
                }
            }

            // new ansi sequences
            if let Some(ansis) = line.ansis.get(&i) {
                new_styles.apply_ansis(ansis);
            }

            // styles changed, flush the current text and append a new element
            if new_styles != self.styles {
                self.flush();
                self.styles = new_styles;
            }

            self.text.push(ch);
        }

        self.flush();
        if self.is_in_link() {
            self.end_link();
        }
    }

    // appends a new element with the current text accumulator and styles if text is not empty
    fn flush(&mut self) {
        if self.text.is_empty() {
            return;
        }

        let element = Element::Text(self.text.clone(), self.styles.clone());

        if self.is_in_link() {
            self.link_elements.push(element);
        } else {
            self.elements.push(element);
        }
        self.text.clear();
    }

    fn is_in_link(&self) -> bool {
        self.end_link_idx.is_some()
    }

    fn start_link(&mut self, end_idx: usize, href: String) {
        self.end_link_idx = Some(end_idx);
        self.link_href = Some(href);
    }

    fn end_link(&mut self) {
        let link = Element::Link(self.link_href.clone().unwrap(), self.link_elements.clone());
        self.elements.push(link);
        self.link_elements.clear();
        self.end_link_idx = None;
        self.link_href = None;
    }
}

pub fn build_elements(line: &Line) -> Vec<Element> {
    let mut builder = Builder::new();
    builder.elements_from(line);
    builder.elements
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Color;

    #[test]
    fn simple() {
        let line = Line::from("foo bar");
        let elements = build_elements(&line);

        let expected = vec![Element::Text("foo bar".to_string(), Styles::new())];

        assert_eq!(elements, expected);
    }

    #[test]
    fn link() {
        let line = Line::from("foo https://reb.gg bar");
        let elements = build_elements(&line);

        let expected = vec![
            Element::Text("foo ".to_string(), Styles::new()),
            Element::Link(
                "https://reb.gg".to_string(),
                vec![Element::Text("https://reb.gg".to_string(), Styles::new())],
            ),
            Element::Text(" bar".to_string(), Styles::new()),
        ];

        assert_eq!(elements, expected);
    }

    #[test]
    fn ends_with_link() {
        let line = Line::from("foo https://reb.gg");
        let elements = build_elements(&line);

        let expected = vec![
            Element::Text("foo ".to_string(), Styles::new()),
            Element::Link(
                "https://reb.gg".to_string(),
                vec![Element::Text("https://reb.gg".to_string(), Styles::new())],
            ),
        ];

        assert_eq!(elements, expected);
    }

    #[test]
    fn highlight() {
        let mut line = Line::from("foo bar");
        line.highlight("oo");
        let elements = build_elements(&line);

        let expected = vec![
            Element::Text("f".to_string(), Styles::new()),
            Element::Text(
                "oo".to_string(),
                Styles {
                    highlight: true,
                    ..Styles::new()
                },
            ),
            Element::Text(" bar".to_string(), Styles::new()),
        ];

        assert_eq!(elements, expected);
    }

    #[test]
    fn ansis() {
        let line = Line::from("\u{1b}[36;1mbold cyan\u{1b}[0m");
        let elements = build_elements(&line);

        let expected = vec![Element::Text(
            "bold cyan".to_string(),
            Styles {
                fg: Some(Color::Bit8(6)),
                bold: true,
                ..Styles::new()
            },
        )];

        assert_eq!(elements, expected);
    }

    #[test]
    fn mixed() {
        let mut line = Line::from("do re me https://\u{1b}[31mreb.gg\u{1b}[0m fa la ti do");
        line.highlight("re");
        let elements = build_elements(&line);

        let expected = vec![
            Element::Text("do ".to_string(), Styles::new()),
            Element::Text(
                "re".to_string(),
                Styles {
                    highlight: true,
                    ..Styles::new()
                },
            ),
            Element::Text(" me ".to_string(), Styles::new()),
            Element::Link(
                "https://reb.gg".to_string(),
                vec![
                    Element::Text("https://".to_string(), Styles::new()),
                    Element::Text(
                        "re".to_string(),
                        Styles {
                            fg: Some(Color::Bit8(1)),
                            highlight: true,
                            ..Styles::new()
                        },
                    ),
                    Element::Text(
                        "b.gg".to_string(),
                        Styles {
                            fg: Some(Color::Bit8(1)),
                            ..Styles::new()
                        },
                    ),
                ],
            ),
            Element::Text(" fa la ti do".to_string(), Styles::new()),
        ];

        assert_eq!(elements, expected);
    }
}
