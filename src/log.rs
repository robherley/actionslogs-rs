use std::collections::HashMap;

use chrono::{DateTime, Utc};
use linkify::LinkFinder;
use serde::Serialize;

use crate::ansi::{extract_ansi, ANSISequence};

// https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions
#[derive(Debug, PartialEq, Serialize)]
pub enum Command {
    Command,
    Debug,
    Error,
    Info,
    Notice,
    Verbose,
    Warning,
    Group,
    EndGroup,
}

impl Command {
    fn from(value: &str) -> Option<Self> {
        match value {
            "command" => Some(Self::Command),
            "debug" => Some(Self::Debug),
            "error" => Some(Self::Error),
            "info" => Some(Self::Info),
            "notice" => Some(Self::Notice),
            "verbose" => Some(Self::Verbose),
            "warning" => Some(Self::Warning),
            "group" => Some(Self::Group),
            "endgroup" => Some(Self::EndGroup),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Line {
    pub number: usize,
    pub cmd: Option<Command>,
    pub ts: i64,
    pub content: String,
    pub links: HashMap<usize, usize>,
    pub ansis: HashMap<usize, Vec<ANSISequence>>,
    pub highlights: HashMap<usize, usize>,
}

impl Line {
    pub fn new(number: usize, id: Option<&str>, raw: &str) -> Self {
        let (ts, content) = Self::parse_ts(id, raw);
        let (cmd, content) = Self::parse_cmd(content);
        let (content, ansis) = extract_ansi(content);

        let links: HashMap<usize, usize> = LinkFinder::new()
            .kinds(&[linkify::LinkKind::Url])
            .links(&content)
            .map(|link| (link.start(), link.end()))
            .collect();

        Self {
            number,
            cmd,
            ts,
            content,
            links,
            ansis,
            highlights: HashMap::new(),
        }
    }

    pub fn highlight(&mut self, search_term: &str) {
        if search_term.is_empty() {
            self.highlights.clear();
            return;
        }

        self.highlights = self
            .content
            .to_lowercase()
            .match_indices(search_term.to_lowercase().as_str())
            .map(|(i, _)| (i, i + search_term.len() - 1))
            .collect();
    }

    fn parse_ts<'a>(id: Option<&'a str>, raw: &str) -> (i64, String) {
        // extract timestamp from beginning of line (completed logs)
        if raw.len() >= 28 {
            match &raw[..28].parse::<DateTime<Utc>>() {
                Ok(ts) => {
                    // 29 chars: skip the timestamp and the space
                    return (ts.timestamp_millis(), raw[29..].to_string());
                }
                Err(_) => {}
            }
        }

        // extract timestamp from id e.g. 1696290982067-0 (streaming logs)
        match id.and_then(|id| id.split_once('-')) {
            Some((unix_ms, _)) => match unix_ms.parse::<i64>() {
                Ok(unix_ms) => {
                    return (unix_ms, raw.to_string());
                }
                Err(_) => {}
            },
            None => {}
        }

        // otherwise default to current time
        (Utc::now().timestamp_millis(), raw.to_string())
    }

    fn parse_cmd(raw: String) -> (Option<Command>, String) {
        let start = match raw {
            ref r if r.starts_with("##[") => Some(3),
            ref r if r.starts_with("[") => Some(1),
            _ => None,
        };

        match start {
            Some(start) => match raw[start..].split_once(']') {
                Some((cmd, content)) => match Command::from(cmd) {
                    Some(cmd) => (Some(cmd), content.to_string()),
                    None => (None, raw),
                },
                None => (None, raw),
            },
            None => (None, raw),
        }
    }
}

impl From<&str> for Line {
    fn from(raw: &str) -> Self {
        Self::new(0, None, raw)
    }
}

#[derive(Debug, Serialize)]
pub struct Group {
    pub line: Line,
    pub ended: bool,
    pub children: Vec<Line>,
}

impl Group {
    pub fn new(line: Line) -> Self {
        Self {
            line,
            ended: false,
            children: Vec::new(),
        }
    }

    pub fn add_line(&mut self, line: Line) {
        self.children.push(line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands() {
        let commands = [
            ("command", Some(Command::Command)),
            ("debug", Some(Command::Debug)),
            ("error", Some(Command::Error)),
            ("info", Some(Command::Info)),
            ("notice", Some(Command::Notice)),
            ("verbose", Some(Command::Verbose)),
            ("warning", Some(Command::Warning)),
            ("group", Some(Command::Group)),
            ("endgroup", Some(Command::EndGroup)),
            ("foo", None),
        ];

        for (cmd, expected) in commands.iter() {
            let line = Line::new(1, None, format!("##[{}] with double #", cmd).as_str());
            assert_eq!(line.cmd, *expected);
        }

        for (cmd, expected) in commands.iter() {
            let line = Line::new(1, None, format!("[{}] with just [", cmd).as_str());
            assert_eq!(line.cmd, *expected);
        }
    }

    #[test]
    fn timestamps() {
        let line = Line::new(1, None, "2024-01-15T00:14:43.5805748Z foo");
        assert_eq!(line.ts, 1705277683580);

        let line = Line::new(1, Some("1705277683580-0"), "foo");
        assert_eq!(line.ts, 1705277683580);

        let line = Line::new(1, Some("foo"), "bar");
        let diff = (Utc::now().timestamp_millis() - line.ts).abs();
        assert!(diff >= 0 && diff < 1000)
    }

    #[test]
    fn ansi() {
        let line = Line::new(1, None, "\u{1b}[31mfoo\u{1b}[0m");
        assert_eq!(line.ansis.len(), 2);
        assert_eq!(line.ansis[&(0 as usize)], vec![ANSISequence::SetFG8(1)]);
        assert_eq!(line.ansis[&(3 as usize)], vec![ANSISequence::Reset]);
    }

    #[test]
    fn links() {
        let line = Line::new(1, None, "foo https://reb.gg bar");
        assert_eq!(line.links.len(), 1);
        assert_eq!(line.links[&(4 as usize)], 18);
    }

    #[test]
    fn highlights() {
        let mut line = Line::new(1, None, "foo bar baz bAr");
        line.highlight("bar");

        assert_eq!(line.highlights.len(), 2);
        assert_eq!(line.highlights[&(4 as usize)], 6);
        assert_eq!(line.highlights[&(12 as usize)], 14);

        line.highlight("BAR");

        assert_eq!(line.highlights.len(), 2);
        assert_eq!(line.highlights[&(4 as usize)], 6);
        assert_eq!(line.highlights[&(12 as usize)], 14);

        line.highlight("");

        assert_eq!(line.highlights.len(), 0);
    }
}
