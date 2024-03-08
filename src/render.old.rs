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
    end_highlight_idx: Option<usize>,
}

fn render(line: Line) -> Vec<Element> {
    let mut elements: Vec<Element> = Vec::new();

    let mut text = String::new();
    let mut styles = HashSet::new();
    let mut end_highlight_idx: Option<usize> = None;

    for (i, c) in line.content.chars().enumerate() {
        let mut new_styles = styles.clone();

        if let Some(end_idx) = line.highlights.get(&i) {
            new_styles.insert(Style::Highlight);
            end_highlight_idx = Some(*end_idx + 1);
        } else if end_highlight_idx.is_some() && end_highlight_idx.unwrap() == i {
            new_styles.remove(&Style::Highlight);
            end_highlight_idx = None;
        }

        if let Some(ansis) = line.ansis.get(&i) {
            for ansi in ansis {
                match ansi {
                    ANSISequence::Reset => {
                        new_styles.clone().iter().for_each(|s| {
                            new_styles.remove(s);
                        });
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

        if styles != new_styles {
            if !text.is_empty() {
                elements.push(Element::Text(text, styles.into_iter().collect()));
                text = String::new();
                styles = new_styles;
            }
        }

        text.push(c);
    }

    if !text.is_empty() {
        elements.push(Element::Text(text, styles.into_iter().collect()));
    }

    println!("{:?}", elements);
    elements
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
