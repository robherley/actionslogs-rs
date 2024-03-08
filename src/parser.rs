use crate::log::{Command, Group, Line};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Debug, Serialize)]
pub enum Node {
    Line(Line),
    Group(Group),
}

#[wasm_bindgen]
#[derive(Debug, Serialize)]
pub struct Parser {
    idx: usize,
    nodes: Vec<Node>,
    search: String,
}

#[wasm_bindgen]
impl Parser {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            idx: 1,
            nodes: Vec::new(),
            search: "".to_string(),
        }
    }

    fn reset(&mut self) {
        self.nodes.clear();
        self.idx = 1;
    }

    fn end_group(&mut self) {
        match self.nodes.last_mut() {
            Some(Node::Group(group)) => {
                group.ended = true;
            }
            _ => {}
        }
    }

    fn in_group(&self) -> bool {
        match self.nodes.last() {
            Some(Node::Group(group)) => !group.ended,
            _ => false,
        }
    }

    #[wasm_bindgen(js_name = stringify)]
    pub fn stringify(&self, pretty: bool) -> Result<String, JsError> {
        let serialize_fn = if pretty {
            serde_json::to_string_pretty
        } else {
            serde_json::to_string
        };

        match serialize_fn(&self.nodes) {
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
        for node in self.nodes.iter_mut() {
            match node {
                Node::Line(line) => {
                    line.highlight(&self.search);
                }
                Node::Group(group) => {
                    for line in group.children.iter_mut() {
                        line.highlight(&self.search);
                    }
                }
            }
        }
    }

    #[wasm_bindgen(js_name = getSearch)]
    pub fn get_search(&self) -> String {
        self.search.clone()
    }

    #[wasm_bindgen(js_name = getMatches)]
    pub fn get_matches(&self) -> usize {
        self.nodes
            .iter()
            .map(|node| match node {
                Node::Line(line) => line.highlights.len(),
                Node::Group(group) => group
                    .children
                    .iter()
                    .map(|line| line.highlights.len())
                    .sum(),
            })
            .sum()
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
                self.nodes.push(Node::Line(line));
            }
            Some(Command::Group) => {
                self.end_group();
                let new_group = Group::new(line);
                self.nodes.push(Node::Group(new_group));
            }
            _ => {
                if self.in_group() {
                    match self.nodes.last_mut() {
                        Some(Node::Group(group)) => {
                            group.add_line(line);
                        }
                        _ => {}
                    }
                } else {
                    self.nodes.push(Node::Line(line));
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

        assert_eq!(parser.nodes.len(), 6);

        for (i, node) in parser.nodes.iter().enumerate() {
            match node {
                Node::Line(line) => {
                    assert_eq!(line.number, i + 1);
                    assert_eq!(line.cmd, None);
                }
                _ => panic!("expected Node::Line"),
            }
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

        assert_eq!(parser.nodes.len(), 3);

        let expected_children: Vec<usize> = vec![3, 4, 1];
        for (i, node) in parser.nodes.iter().enumerate() {
            match node {
                Node::Group(group) => {
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

        assert_eq!(parser.nodes.len(), 5);

        let expected_group = vec![true, false, true, false, false];
        for (i, node) in parser.nodes.iter().enumerate() {
            match node {
                Node::Group(_) => {
                    assert!(expected_group[i], "expected Node::Group");
                }
                _ => assert!(!expected_group[i], "expected Node::Line"),
            }
        }

        // since the last two endgroups are not closing any groups, they are rendered as regular lines
        parser.nodes[3..].iter().for_each(|node| match node {
            Node::Line(line) => match line.cmd {
                Some(Command::EndGroup) => {}
                _ => panic!("expected Command::EndGroup"),
            },
            _ => panic!("expected Node::Line"),
        });
    }

    #[test]
    fn search() {
        let lines = concat!("foo\n", "bar\n", "baz\n");

        let mut parser = Parser::new();
        parser.set_raw(lines);

        let find_matches = |parser: &Parser| -> Vec<bool> {
            parser
                .nodes
                .iter()
                .map(|node| match node {
                    Node::Line(line) => !line.highlights.is_empty(),
                    _ => false,
                })
                .collect()
        };

        assert_eq!(parser.nodes.len(), 3);
        assert_eq!(find_matches(&parser), vec![false, false, false]);

        parser.set_search("bar");
        assert_eq!(parser.get_matches(), 1);
        assert_eq!(find_matches(&parser), vec![false, true, false]);
        match parser.nodes.get(1) {
            Some(Node::Line(line)) => assert_eq!(line.highlights, HashMap::from([(0, 2)])),
            _ => panic!("expected Node::Line"),
        }

        parser.add_line("", "----> bar <----");
        assert_eq!(parser.get_matches(), 2);
        assert_eq!(find_matches(&parser), vec![false, true, false, true]);
        match parser.nodes.get(3) {
            Some(Node::Line(line)) => assert_eq!(line.highlights, HashMap::from([(6, 8)])),
            _ => panic!("expected Node::Line"),
        }

        parser.add_line("", "#[group]some group");
        parser.add_line("", "baz bar baz");
        parser.add_line("", "#[endgroup]");
        assert_eq!(parser.get_matches(), 3);

        parser.set_search("");
        assert_eq!(parser.get_matches(), 0);
    }
}
