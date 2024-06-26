use crate::line::{Command, Line};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Serialize)]
pub struct Parser {
    idx: usize,
    lines: Vec<Line>,
    search: String,
}

#[wasm_bindgen]
impl Parser {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            idx: 1,
            lines: Vec::new(),
            search: "".to_string(),
        }
    }

    fn reset(&mut self) {
        self.lines.clear();
        self.idx = 1;
    }

    fn end_group(&mut self) {
        if let Some(line) = self.lines.last_mut() {
            line.end_group();
        }
    }

    fn in_group(&self) -> bool {
        if let Some(line) = self.lines.last() {
            if let Some(group) = &line.group {
                return !group.ended;
            }
        }

        false
    }

    #[wasm_bindgen(js_name = stringify)]
    pub fn stringify(&self, pretty: bool) -> Result<String, JsError> {
        let serialize_fn = if pretty {
            serde_json::to_string_pretty
        } else {
            serde_json::to_string
        };

        match serialize_fn(&self.lines) {
            Ok(json) => Ok(json),
            Err(err) => Err(JsError::new(&format!("{:?}", err))),
        }
    }

    #[wasm_bindgen(js_name = setRaw)]
    pub fn set_raw(&mut self, raw: &str) {
        self.reset();
        raw.lines().for_each(|line| self.add_line("", line));
    }

    #[wasm_bindgen(js_name = setSearch)]
    pub fn set_search(&mut self, search: &str) {
        self.search = search.to_lowercase();
        for line in self.lines.iter_mut() {
            line.highlight(&self.search);
        }
    }

    #[wasm_bindgen(js_name = getMatches)]
    pub fn matches(&self) -> usize {
        self.lines.iter().map(|line| line.matches()).sum()
    }

    #[wasm_bindgen(js_name = addLine)]
    pub fn add_line(&mut self, id: &str, raw: &str) {
        let id = if id.is_empty() { None } else { Some(id) };
        let mut line = Line::new(self.idx, id, raw);

        if !self.search.is_empty() {
            line.highlight(&self.search);
        }

        match line.cmd {
            Some(Command::EndGroup) => {
                if self.in_group() {
                    self.end_group();
                    // don't add endgroup lines when they properly close a group
                    return;
                }

                // otherwise treat endgroup as a regular line
                self.lines.push(line);
            }
            Some(Command::Group) => {
                self.end_group();
                line.start_group();
                self.lines.push(line);
            }
            _ => {
                if self.in_group() {
                    if let Some(last_line) = self.lines.last_mut() {
                        last_line.add_child(line);
                    }
                } else {
                    self.lines.push(line);
                }
            }
        }

        self.idx += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::vec;

    #[test]
    fn normal() {
        let lines = concat!(
        "2024-01-15T00:14:43.5805748Z Requested labels: ubuntu-latest\n",
        "2024-01-15T00:14:43.5806028Z Job defined at: robherley/tmp/.github/workflows/blank.yml@refs/heads/main\n",
        "2024-01-15T00:14:43.5806125Z Waiting for a runner to pick up this job...\n",
        "2024-01-15T00:14:44.2854453Z Job is waiting for a hosted runner to come online.\n",
        "2024-01-15T00:14:46.5843551Z Job is about to start running on the hosted runner: GitHub Actions 2 (hosted)\n",
        "2024-01-15T00:14:49.2802822Z Current runner version: '2.311.0'\n",
        );

        let mut parser = Parser::new();
        parser.set_raw(lines);

        assert_eq!(parser.lines.len(), 6);

        for (i, line) in parser.lines.iter().enumerate() {
            assert_eq!(line.number, i + 1);
            assert_eq!(line.cmd, None);
        }
    }

    #[test]
    fn with_groups() {
        let lines = concat!(
            "2024-01-15T00:14:49.2830954Z ##[group]Operating System\n",
            "2024-01-15T00:14:49.2831846Z Ubuntu\n",
            "2024-01-15T00:14:49.2832204Z 22.04.3\n",
            "2024-01-15T00:14:49.2832638Z LTS\n",
            "2024-01-15T00:14:49.2833085Z ##[endgroup]\n",
            "2024-01-15T00:14:49.2833509Z ##[group]Runner Image\n",
            "2024-01-15T00:14:49.2834023Z Image: ubuntu-22.04\n",
            "2024-01-15T00:14:49.2834552Z Version: 20240107.1.0\n",
            "2024-01-15T00:14:49.2835705Z Included Software: https://github.com/actions/runner-images/blob/ubuntu22/20240107.1/images/ubuntu/Ubuntu2204-Readme.md\n",
            "2024-01-15T00:14:49.2837409Z Image Release: https://github.com/actions/runner-images/releases/tag/ubuntu22%2F20240107.1\n",
            "2024-01-15T00:14:49.2838476Z ##[endgroup]\n",
            "2024-01-15T00:14:49.2838965Z ##[group]Runner Image Provisioner\n",
            "2024-01-15T00:14:49.2839497Z 2.0.321.1\n",
            "2024-01-15T00:14:49.2839965Z ##[endgroup]\n",
        );

        let mut parser = Parser::new();
        parser.set_raw(lines);

        assert_eq!(parser.lines.len(), 3);

        let expected_children: Vec<usize> = vec![3, 4, 1];
        for (i, line) in parser.lines.iter().enumerate() {
            match &line.group {
                Some(group) => {
                    assert_eq!(group.children.len(), expected_children[i]);
                }
                _ => panic!("expected Node::Group"),
            }
        }
    }

    #[test]
    fn weird_endgroup_behavior() {
        let lines = concat!(
            "##[group]start group\n",
            "inside group\n",
            "##[endgroup]\n",
            "outside group\n",
            "##[group]start another group\n",
            "inside another group\n",
            "##[endgroup]\n",
            "##[endgroup]\n",
            "##[endgroup]\n",
        );

        let mut parser = Parser::new();
        parser.set_raw(lines);

        assert_eq!(parser.lines.len(), 5);

        let expected_group = vec![true, false, true, false, false];
        for (i, line) in parser.lines.iter().enumerate() {
            match line.group {
                Some(_) => {
                    assert!(expected_group[i], "expected group");
                }
                _ => assert!(!expected_group[i], "did not expect group"),
            }
        }

        // since the last two endgroups are not closing any groups, they are rendered as regular lines
        parser.lines[3..].iter().for_each(|line| match line.cmd {
            Some(Command::EndGroup) => {}
            _ => panic!("expected Command::EndGroup"),
        });
    }

    #[test]
    fn search() {
        let lines = concat!("foo\n", "bar\n", "baz\n");

        let mut parser = Parser::new();
        parser.set_raw(lines);

        let find_matches = |parser: &Parser| -> Vec<bool> {
            parser.lines.iter().map(|line| line.matches() > 0).collect()
        };

        assert_eq!(parser.lines.len(), 3);
        assert_eq!(find_matches(&parser), vec![false, false, false]);

        parser.set_search("bar");
        assert_eq!(parser.matches(), 1);
        assert_eq!(find_matches(&parser), vec![false, true, false]);
        match parser.lines.get(1) {
            Some(line) => assert_eq!(line.highlights, HashMap::from([(0, 3)])),
            _ => panic!("expected line at index 1"),
        }

        parser.add_line("", "----> bar <----");
        assert_eq!(parser.matches(), 2);
        assert_eq!(find_matches(&parser), vec![false, true, false, true]);
        match parser.lines.get(3) {
            Some(line) => assert_eq!(line.highlights, HashMap::from([(6, 9)])),
            _ => panic!("expected line at index 3"),
        }

        parser.add_line("", "#[group]some group");
        parser.add_line("", "baz bar baz");
        parser.add_line("", "#[endgroup]");
        assert_eq!(parser.matches(), 3);

        parser.set_search("");
        assert_eq!(parser.matches(), 0);
    }
}
