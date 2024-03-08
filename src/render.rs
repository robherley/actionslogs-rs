use crate::ansi::ANSISequence;
use crate::log::Line;
use std::collections::HashSet;
use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Style {
    ANSI(ANSISequence),
    Highlight,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Element {
    // Text(content, styles)
    Text(String, Vec<Style>),
    // Link(href, children)
    Link(String, Vec<Element>),
}

pub struct Renderer {
    // output elements
    elements: Vec<Element>,
    // text accumulator
    text: String,
    // current styles
    styles: HashSet<Style>,
    // marker to end highlight
    highlight_end_idx: Option<usize>,
}

impl Renderer {
    fn new() -> Self {
        Self {
            elements: Vec::new(),
            text: String::new(),
            styles: HashSet::new(),
            highlight_end_idx: None,
        }
    }

    fn render(mut self, line: Line) {
        for (i, c) in line.content.chars().enumerate() {
            let mut new_styles = self.merge_styles(&line, i);

            if let Some(end_idx) = line.highlights.get(&i) {
                new_styles.insert(Style::Highlight);
                self.highlight_end_idx = Some(*end_idx + 1);
            }

            if let Some(end_idx) = self.highlight_end_idx {
                if i == end_idx {
                    new_styles.remove(&Style::Highlight);
                    self.highlight_end_idx = None;
                }
            }

            if self.styles != new_styles {
                if !self.text.is_empty() {
                    self.elements
                        .push(Element::Text(self.text, self.styles.into_iter().collect()));
                    self.text = String::new();
                    self.styles = new_styles;
                }
            }

            self.text.push(c);
        }

        if !self.text.is_empty() {
            self.elements
                .push(Element::Text(self.text, self.styles.into_iter().collect()));
        }

        println!("{:?}", self.elements);
    }

    fn merge_styles(&self, line: &Line, i: usize) -> HashSet<Style> {
        let mut new_styles = self.styles.clone();

        if let Some(ansis) = line.ansis.get(&i) {
            for ansi in ansis {
                match ansi {
                    ANSISequence::Reset => {
                        let is_highlighted = new_styles.contains(&Style::Highlight);
                        new_styles.clear();
                        if is_highlighted {
                            new_styles.insert(Style::Highlight);
                        }
                    }
                    ANSISequence::NotBold => {
                        new_styles.remove(&Style::ANSI(ANSISequence::Bold));
                    }
                    ANSISequence::NotItalic => {
                        new_styles.remove(&Style::ANSI(ANSISequence::Italic));
                    }
                    ANSISequence::NotUnderline => {
                        new_styles.remove(&Style::ANSI(ANSISequence::Underline));
                    }
                    seq => {
                        new_styles.insert(Style::ANSI(seq.clone()));
                    }
                }
            }
        }

        new_styles
    }
}

fn render(line: Line) {
    let renderer = Renderer::new();
    renderer.render(line);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tmp() {
        let mut line = Line::from("normal [31mRed Text[0m https://reb.gg normal foo bar");
        line.highlight("Red");
        render(line);
    }
}
