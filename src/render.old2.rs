use crate::ansi::ANSISequence;
use crate::log::Line;
use std::collections::HashSet;
use std::iter::Enumerate;
use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Style {
    ANSI(ANSISequence),
    Highlight,
}

#[derive(Clone, Debug)]
pub struct Element {
    href: String,
    styles: HashSet<ANSISequence>,
    text: String,
    highlighted: bool,
    children: Vec<Element>,
}

impl Element {
    pub fn new() -> Self {
        Self {
            href: String::new(),
            styles: HashSet::new(),
            text: String::new(),
            highlighted: false,
            children: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty() && self.children.is_empty()
    }
}

pub struct Renderer<'a> {
    line: &'a Line,
    ansis: HashSet<ANSISequence>,
    pub elements: Vec<Element>,
}

impl<'a> Renderer<'a> {
    pub fn new(line: &'a Line) -> Self {
        let mut renderer = Self {
            line,
            ansis: HashSet::new(),
            elements: Vec::new(),
        };

        renderer.elements = renderer.render(0..line.content.len());
        renderer
    }

    fn render(&mut self, range: Range<usize>) -> Vec<Element> {
        println!("rendering: {:?}", range);

        let mut elements = Vec::new();
        let mut current: Element = Element::new();
        let end = range.end;

        if range.start >= end {
            return elements;
        }

        let mut iter = self.line.content[range].chars().enumerate();
        while let Some((i, c)) = iter.next() {
            if let Some(end_idx) = self.starts_link(i) {
                println!("link: {}..{}", i, end_idx);
                if !current.is_empty() {
                    elements.push(current);
                }
                current = Element::new();
                current.href = self.line.content[i..end_idx].to_string();
                current.children = self.render(i..end_idx);
                let mut rest = self.render(end_idx..end);
                elements.append(rest.as_mut());
                return elements;
            } else if let Some(end_idx) = self.starts_highlight(i) {
                println!("highlight: {}..{}", i, end_idx);
                if !current.is_empty() {
                    elements.push(current);
                }
                current = Element::new();
                current.highlighted = true;
                current.children = self.render(i..end_idx);
                let mut rest = self.render(end_idx..end);
                elements.append(rest.as_mut());
                return elements;
            } else {
                current.text.push(c);
            }
        }

        if !current.is_empty() {
            elements.push(current);
        }

        elements
    }

    fn starts_highlight(&self, i: usize) -> Option<usize> {
        match self.line.highlights.get(&i) {
            Some(end_idx) => Some(*end_idx + 1),
            None => None,
        }
    }

    fn starts_link(&self, i: usize) -> Option<usize> {
        match self.line.links.get(&i) {
            Some(end_idx) => Some(*end_idx + 1),
            None => None,
        }
    }

    fn merge_ansi(&self, i: usize) -> HashSet<ANSISequence> {
        let mut new_ansi = self.ansis.clone();

        if let Some(ansis) = self.line.ansis.get(&i) {
            for ansi in ansis {
                match ansi {
                    ANSISequence::Reset => {
                        new_ansi.clear();
                    }
                    ANSISequence::NotBold => {
                        new_ansi.remove(&ANSISequence::Bold);
                    }
                    ANSISequence::NotItalic => {
                        new_ansi.remove(&ANSISequence::Italic);
                    }
                    ANSISequence::NotUnderline => {
                        new_ansi.remove(&ANSISequence::Underline);
                    }
                    seq => {
                        new_ansi.insert(seq.clone());
                    }
                }
            }
        }

        new_ansi
    }
}

fn render(line: Line) {
    let renderer = Renderer::new(&line);
    println!("{:?}", renderer.elements);
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

    // with ansi

    // with highlight

    // with link

    // with ansi and highlight (does not reset)

    // with ansi and link
}
