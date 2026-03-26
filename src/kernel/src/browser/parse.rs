// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// HTML5 + CSS parsing — wraps html5ever + rust-cssparser.
// Phase 1: Type definitions and parsing pipeline interfaces.
// Recursion depth limits enforced by design.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

pub const MAX_RECURSION_DEPTH: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    Document,
    Element(String),
    Text(String),
    Comment(String),
}

#[derive(Debug, Clone)]
pub struct DomNode {
    pub node_type: NodeType,
    pub children: Vec<DomNode>,
    pub attributes: Vec<(String, String)>,
    pub depth: usize,
}

impl DomNode {
    pub fn new_element(tag: String, depth: usize) -> Self {
        Self {
            node_type: NodeType::Element(tag),
            children: Vec::new(),
            attributes: Vec::new(),
            depth,
        }
    }

    pub fn new_text(text: String, depth: usize) -> Self {
        Self {
            node_type: NodeType::Text(text),
            children: Vec::new(),
            attributes: Vec::new(),
            depth,
        }
    }

    pub fn new_document() -> Self {
        Self {
            node_type: NodeType::Document,
            children: Vec::new(),
            attributes: Vec::new(),
            depth: 0,
        }
    }

    pub fn add_child(&mut self, child: DomNode) -> Result<(), ParseError> {
        if child.depth >= MAX_RECURSION_DEPTH {
            return Err(ParseError::RecursionLimitExceeded);
        }
        self.children.push(child);
        Ok(())
    }

    pub fn tag_name(&self) -> Option<&str> {
        match &self.node_type {
            NodeType::Element(tag) => Some(tag.as_str()),
            _ => None,
        }
    }

    pub fn text_content(&self) -> Option<&str> {
        match &self.node_type {
            NodeType::Text(text) => Some(text.as_str()),
            _ => None,
        }
    }

    pub fn node_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.node_count()).sum::<usize>()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssProperty {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct CssRule {
    pub selector: String,
    pub properties: Vec<CssProperty>,
}

#[derive(Debug, Clone)]
pub struct StyleSheet {
    pub rules: Vec<CssRule>,
}

impl StyleSheet {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: CssRule) {
        self.rules.push(rule);
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

#[derive(Debug, Clone)]
pub enum ParseError {
    RecursionLimitExceeded,
    MalformedHtml(String),
    MalformedCss(String),
    UnsupportedEncoding,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::RecursionLimitExceeded => {
                write!(f, "HTML recursion depth limit ({}) exceeded", MAX_RECURSION_DEPTH)
            }
            ParseError::MalformedHtml(msg) => write!(f, "Malformed HTML: {}", msg),
            ParseError::MalformedCss(msg) => write!(f, "Malformed CSS: {}", msg),
            ParseError::UnsupportedEncoding => write!(f, "Unsupported character encoding"),
        }
    }
}

pub type ParseResult<T> = core::result::Result<T, ParseError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom_node_element() {
        let node = DomNode::new_element("div".into(), 0);
        assert_eq!(node.tag_name(), Some("div"));
        assert_eq!(node.node_count(), 1);
    }

    #[test]
    fn test_dom_node_nesting() {
        let mut parent = DomNode::new_element("div".into(), 0);
        let child = DomNode::new_text("Hello".into(), 1);
        parent.add_child(child).unwrap();
        assert_eq!(parent.node_count(), 2);
    }

    #[test]
    fn test_recursion_limit() {
        let mut parent = DomNode::new_element("div".into(), 0);
        let deep = DomNode::new_text("too deep".into(), MAX_RECURSION_DEPTH);
        assert!(parent.add_child(deep).is_err());
    }

    #[test]
    fn test_stylesheet() {
        let mut ss = StyleSheet::new();
        ss.add_rule(CssRule {
            selector: "body".into(),
            properties: alloc::vec![CssProperty {
                name: "color".into(),
                value: "red".into(),
            }],
        });
        assert_eq!(ss.rule_count(), 1);
    }
}
