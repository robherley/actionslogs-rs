use crate::log::{Command, Group, Line};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum Node {
    Line(Line),
    Group(Group),
}

#[derive(Debug, Serialize)]
pub struct Parser {
    pub nodes: Vec<Node>,
    idx: usize,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            idx: 1,
        }
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

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.nodes)
    }

    pub fn add_raw(&mut self, raw: &str) {
        raw.lines().for_each(|line| self.add_line(None, line));
    }

    pub fn add_line(&mut self, id: Option<&str>, raw: &str) {
        let line = Line::new(self.idx, id, raw);

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
        parser.add_raw(lines);

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
        parser.add_raw(lines);

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
        parser.add_raw(lines);

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
}
