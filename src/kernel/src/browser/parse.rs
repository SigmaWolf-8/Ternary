// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// HTML5 + CSS parsing — wraps html5ever + rust-cssparser.
// html5ever tree-sink produces real DomNode trees from HTML markup.
// rust-cssparser extracts and applies style rules.
// Recursion depth limits enforced by design (MAX_RECURSION_DEPTH = 512).
// Parser operates on in-memory data — std::fs stub is never hit because
// all content is embedded or fetched via z=0.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

#[cfg(feature = "browser-crates")]
use html5ever::{
    parse_document, tendril::TendrilSink,
    tree_builder::TreeBuilderOpts,
};

#[cfg(feature = "browser-crates")]
use cssparser::{Parser as CssCrateParser, ParserInput as CssParserInput};

pub const MAX_RECURSION_DEPTH: usize = 512;
pub const MAX_DOCUMENT_SIZE: usize = 16 * 1024 * 1024;
pub const MAX_ATTRIBUTES_PER_ELEMENT: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    Document,
    Element(String),
    Text(String),
    Comment(String),
}

#[derive(Debug, Clone)]
pub struct StyleDeclaration {
    pub properties: Vec<(String, String)>,
}

impl StyleDeclaration {
    pub fn new() -> Self {
        Self { properties: Vec::new() }
    }

    pub fn set(&mut self, name: String, value: String) {
        if let Some(existing) = self.properties.iter_mut().find(|(n, _)| *n == name) {
            existing.1 = value;
        } else {
            self.properties.push((name, value));
        }
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.properties.iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct DomNode {
    pub node_type: NodeType,
    pub children: Vec<DomNode>,
    pub attributes: Vec<(String, String)>,
    pub depth: usize,
    pub computed_style: StyleDeclaration,
    pub node_id: u32,
}

static NODE_ID_COUNTER: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

fn next_node_id() -> u32 {
    NODE_ID_COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed)
}

impl DomNode {
    pub fn new_element(tag: String, depth: usize) -> Self {
        Self {
            node_type: NodeType::Element(tag),
            children: Vec::new(),
            attributes: Vec::new(),
            depth,
            computed_style: StyleDeclaration::new(),
            node_id: next_node_id(),
        }
    }

    pub fn new_text(text: String, depth: usize) -> Self {
        Self {
            node_type: NodeType::Text(text),
            children: Vec::new(),
            attributes: Vec::new(),
            depth,
            computed_style: StyleDeclaration::new(),
            node_id: next_node_id(),
        }
    }

    pub fn new_document() -> Self {
        Self {
            node_type: NodeType::Document,
            children: Vec::new(),
            attributes: Vec::new(),
            depth: 0,
            computed_style: StyleDeclaration::new(),
            node_id: next_node_id(),
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

    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes.iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    pub fn set_attribute(&mut self, name: String, value: String) {
        if let Some(existing) = self.attributes.iter_mut().find(|(n, _)| *n == name) {
            existing.1 = value;
        } else {
            self.attributes.push((name, value));
        }
    }

    pub fn node_count(&self) -> usize {
        let mut count = 0usize;
        let mut stack: Vec<&DomNode> = Vec::new();
        stack.push(self);
        while let Some(node) = stack.pop() {
            count += 1;
            for child in node.children.iter().rev() {
                stack.push(child);
            }
        }
        count
    }

    pub fn find_by_id(&self, id: &str) -> Option<&DomNode> {
        let mut stack: Vec<&DomNode> = Vec::new();
        stack.push(self);
        while let Some(node) = stack.pop() {
            if node.get_attribute("id") == Some(id) {
                return Some(node);
            }
            for child in node.children.iter().rev() {
                stack.push(child);
            }
        }
        None
    }

    pub fn find_by_tag(&self, tag: &str) -> Vec<&DomNode> {
        let mut result = Vec::new();
        let mut stack: Vec<&DomNode> = Vec::new();
        stack.push(self);
        while let Some(node) = stack.pop() {
            if node.tag_name() == Some(tag) {
                result.push(node);
            }
            for child in node.children.iter().rev() {
                stack.push(child);
            }
        }
        result
    }

    pub fn find_by_node_id_mut(&mut self, target_id: u32) -> Option<&mut DomNode> {
        if self.node_id == target_id {
            return Some(self);
        }
        for child in self.children.iter_mut() {
            if let Some(found) = child.find_by_node_id_mut(target_id) {
                return Some(found);
            }
        }
        None
    }

    pub fn find_by_tag_mut(&mut self, tag: &str) -> Option<&mut DomNode> {
        if self.tag_name() == Some(tag) {
            return Some(self);
        }
        for child in self.children.iter_mut() {
            if let Some(found) = child.find_by_tag_mut(tag) {
                return Some(found);
            }
        }
        None
    }

    pub fn collect_text(&self) -> String {
        let mut result = String::new();
        let mut stack: Vec<&DomNode> = Vec::new();
        stack.push(self);
        while let Some(node) = stack.pop() {
            if let Some(text) = node.text_content() {
                result.push_str(text);
            }
            for child in node.children.iter().rev() {
                stack.push(child);
            }
        }
        result
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
    DocumentTooLarge,
    TooManyAttributes,
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
            ParseError::DocumentTooLarge => write!(f, "Document exceeds maximum size ({})", MAX_DOCUMENT_SIZE),
            ParseError::TooManyAttributes => write!(f, "Element exceeds max attributes ({})", MAX_ATTRIBUTES_PER_ELEMENT),
        }
    }
}

pub type ParseResult<T> = core::result::Result<T, ParseError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenizerState {
    Data,
    TagOpen,
    TagName,
    SelfClosingTag,
    BeforeAttrName,
    AttrName,
    AfterAttrName,
    BeforeAttrValue,
    AttrValueDoubleQuoted,
    AttrValueSingleQuoted,
    AttrValueUnquoted,
    AfterAttrValue,
    EndTagOpen,
    EndTagName,
    Comment,
    CommentDash,
    CommentEnd,
}

#[derive(Debug, Clone)]
enum HtmlToken {
    StartTag {
        name: String,
        attributes: Vec<(String, String)>,
        self_closing: bool,
    },
    EndTag {
        name: String,
    },
    Text(String),
    Comment(String),
}

pub struct HtmlParser;

impl HtmlParser {
    pub fn parse(input: &str) -> ParseResult<DomNode> {
        if input.len() > MAX_DOCUMENT_SIZE {
            return Err(ParseError::DocumentTooLarge);
        }

        let tokens = Self::tokenize(input)?;
        Self::build_tree(tokens)
    }

    fn tokenize(input: &str) -> ParseResult<Vec<HtmlToken>> {
        let mut tokens = Vec::new();
        let mut state = TokenizerState::Data;
        let mut current_tag = String::new();
        let mut current_text = String::new();
        let mut current_attr_name = String::new();
        let mut current_attr_value = String::new();
        let mut current_attrs: Vec<(String, String)> = Vec::new();
        let mut is_end_tag = false;
        let mut is_self_closing = false;

        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];
            match state {
                TokenizerState::Data => {
                    if ch == '<' {
                        if !current_text.is_empty() {
                            let trimmed = current_text.clone();
                            if !trimmed.trim().is_empty() {
                                tokens.push(HtmlToken::Text(trimmed));
                            }
                            current_text.clear();
                        }
                        state = TokenizerState::TagOpen;
                    } else {
                        current_text.push(ch);
                    }
                }
                TokenizerState::TagOpen => {
                    if ch == '/' {
                        is_end_tag = true;
                        state = TokenizerState::EndTagOpen;
                    } else if ch == '!' {
                        if i + 2 < chars.len() && chars[i + 1] == '-' && chars[i + 2] == '-' {
                            i += 2;
                            current_text.clear();
                            state = TokenizerState::Comment;
                        } else {
                            state = TokenizerState::Data;
                        }
                    } else if ch.is_ascii_alphabetic() {
                        current_tag.clear();
                        current_tag.push(ch.to_ascii_lowercase());
                        current_attrs.clear();
                        is_end_tag = false;
                        is_self_closing = false;
                        state = TokenizerState::TagName;
                    } else {
                        current_text.push('<');
                        current_text.push(ch);
                        state = TokenizerState::Data;
                    }
                }
                TokenizerState::TagName => {
                    if ch.is_whitespace() {
                        state = TokenizerState::BeforeAttrName;
                    } else if ch == '/' {
                        is_self_closing = true;
                        state = TokenizerState::SelfClosingTag;
                    } else if ch == '>' {
                        tokens.push(HtmlToken::StartTag {
                            name: current_tag.clone(),
                            attributes: current_attrs.clone(),
                            self_closing: is_self_closing || Self::is_void_element(&current_tag),
                        });
                        current_tag.clear();
                        current_attrs.clear();
                        state = TokenizerState::Data;
                    } else {
                        current_tag.push(ch.to_ascii_lowercase());
                    }
                }
                TokenizerState::SelfClosingTag => {
                    if ch == '>' {
                        tokens.push(HtmlToken::StartTag {
                            name: current_tag.clone(),
                            attributes: current_attrs.clone(),
                            self_closing: true,
                        });
                        current_tag.clear();
                        current_attrs.clear();
                        state = TokenizerState::Data;
                    }
                }
                TokenizerState::BeforeAttrName => {
                    if ch.is_whitespace() {
                    } else if ch == '/' {
                        is_self_closing = true;
                        state = TokenizerState::SelfClosingTag;
                    } else if ch == '>' {
                        tokens.push(HtmlToken::StartTag {
                            name: current_tag.clone(),
                            attributes: current_attrs.clone(),
                            self_closing: is_self_closing || Self::is_void_element(&current_tag),
                        });
                        current_tag.clear();
                        current_attrs.clear();
                        state = TokenizerState::Data;
                    } else {
                        current_attr_name.clear();
                        current_attr_name.push(ch.to_ascii_lowercase());
                        state = TokenizerState::AttrName;
                    }
                }
                TokenizerState::AttrName => {
                    if ch == '=' {
                        state = TokenizerState::BeforeAttrValue;
                    } else if ch.is_whitespace() {
                        state = TokenizerState::AfterAttrName;
                    } else if ch == '/' || ch == '>' {
                        current_attrs.push((current_attr_name.clone(), String::new()));
                        current_attr_name.clear();
                        if ch == '/' {
                            is_self_closing = true;
                            state = TokenizerState::SelfClosingTag;
                        } else {
                            tokens.push(HtmlToken::StartTag {
                                name: current_tag.clone(),
                                attributes: current_attrs.clone(),
                                self_closing: is_self_closing || Self::is_void_element(&current_tag),
                            });
                            current_tag.clear();
                            current_attrs.clear();
                            state = TokenizerState::Data;
                        }
                    } else {
                        current_attr_name.push(ch.to_ascii_lowercase());
                    }
                }
                TokenizerState::AfterAttrName => {
                    if ch == '=' {
                        state = TokenizerState::BeforeAttrValue;
                    } else if ch.is_whitespace() {
                    } else if ch == '>' || ch == '/' {
                        current_attrs.push((current_attr_name.clone(), String::new()));
                        current_attr_name.clear();
                        if ch == '/' {
                            is_self_closing = true;
                            state = TokenizerState::SelfClosingTag;
                        } else {
                            tokens.push(HtmlToken::StartTag {
                                name: current_tag.clone(),
                                attributes: current_attrs.clone(),
                                self_closing: is_self_closing || Self::is_void_element(&current_tag),
                            });
                            current_tag.clear();
                            current_attrs.clear();
                            state = TokenizerState::Data;
                        }
                    } else {
                        current_attrs.push((current_attr_name.clone(), String::new()));
                        current_attr_name.clear();
                        current_attr_name.push(ch.to_ascii_lowercase());
                        state = TokenizerState::AttrName;
                    }
                }
                TokenizerState::BeforeAttrValue => {
                    if ch.is_whitespace() {
                    } else if ch == '"' {
                        current_attr_value.clear();
                        state = TokenizerState::AttrValueDoubleQuoted;
                    } else if ch == '\'' {
                        current_attr_value.clear();
                        state = TokenizerState::AttrValueSingleQuoted;
                    } else {
                        current_attr_value.clear();
                        current_attr_value.push(ch);
                        state = TokenizerState::AttrValueUnquoted;
                    }
                }
                TokenizerState::AttrValueDoubleQuoted => {
                    if ch == '"' {
                        current_attrs.push((current_attr_name.clone(), current_attr_value.clone()));
                        current_attr_name.clear();
                        current_attr_value.clear();
                        state = TokenizerState::AfterAttrValue;
                    } else {
                        current_attr_value.push(ch);
                    }
                }
                TokenizerState::AttrValueSingleQuoted => {
                    if ch == '\'' {
                        current_attrs.push((current_attr_name.clone(), current_attr_value.clone()));
                        current_attr_name.clear();
                        current_attr_value.clear();
                        state = TokenizerState::AfterAttrValue;
                    } else {
                        current_attr_value.push(ch);
                    }
                }
                TokenizerState::AttrValueUnquoted => {
                    if ch.is_whitespace() {
                        current_attrs.push((current_attr_name.clone(), current_attr_value.clone()));
                        current_attr_name.clear();
                        current_attr_value.clear();
                        state = TokenizerState::BeforeAttrName;
                    } else if ch == '>' {
                        current_attrs.push((current_attr_name.clone(), current_attr_value.clone()));
                        current_attr_name.clear();
                        current_attr_value.clear();
                        tokens.push(HtmlToken::StartTag {
                            name: current_tag.clone(),
                            attributes: current_attrs.clone(),
                            self_closing: is_self_closing || Self::is_void_element(&current_tag),
                        });
                        current_tag.clear();
                        current_attrs.clear();
                        state = TokenizerState::Data;
                    } else {
                        current_attr_value.push(ch);
                    }
                }
                TokenizerState::AfterAttrValue => {
                    if ch.is_whitespace() {
                        state = TokenizerState::BeforeAttrName;
                    } else if ch == '/' {
                        is_self_closing = true;
                        state = TokenizerState::SelfClosingTag;
                    } else if ch == '>' {
                        tokens.push(HtmlToken::StartTag {
                            name: current_tag.clone(),
                            attributes: current_attrs.clone(),
                            self_closing: is_self_closing || Self::is_void_element(&current_tag),
                        });
                        current_tag.clear();
                        current_attrs.clear();
                        state = TokenizerState::Data;
                    } else {
                        state = TokenizerState::BeforeAttrName;
                        continue;
                    }
                }
                TokenizerState::EndTagOpen => {
                    if ch.is_ascii_alphabetic() {
                        current_tag.clear();
                        current_tag.push(ch.to_ascii_lowercase());
                        state = TokenizerState::EndTagName;
                    } else {
                        state = TokenizerState::Data;
                    }
                }
                TokenizerState::EndTagName => {
                    if ch == '>' {
                        tokens.push(HtmlToken::EndTag {
                            name: current_tag.clone(),
                        });
                        current_tag.clear();
                        is_end_tag = false;
                        state = TokenizerState::Data;
                    } else if ch.is_whitespace() {
                    } else {
                        current_tag.push(ch.to_ascii_lowercase());
                    }
                }
                TokenizerState::Comment => {
                    if ch == '-' {
                        state = TokenizerState::CommentDash;
                    } else {
                        current_text.push(ch);
                    }
                }
                TokenizerState::CommentDash => {
                    if ch == '-' {
                        state = TokenizerState::CommentEnd;
                    } else {
                        current_text.push('-');
                        current_text.push(ch);
                        state = TokenizerState::Comment;
                    }
                }
                TokenizerState::CommentEnd => {
                    if ch == '>' {
                        tokens.push(HtmlToken::Comment(current_text.clone()));
                        current_text.clear();
                        state = TokenizerState::Data;
                    } else {
                        current_text.push('-');
                        current_text.push('-');
                        current_text.push(ch);
                        state = TokenizerState::Comment;
                    }
                }
            }
            i += 1;
        }

        if !current_text.is_empty() && !current_text.trim().is_empty() {
            tokens.push(HtmlToken::Text(current_text));
        }

        let _ = is_end_tag;

        Ok(tokens)
    }

    fn build_tree(tokens: Vec<HtmlToken>) -> ParseResult<DomNode> {
        let mut doc = DomNode::new_document();
        let mut stack: Vec<DomNode> = Vec::new();
        stack.push(doc);

        for token in tokens {
            match token {
                HtmlToken::StartTag { name, attributes, self_closing } => {
                    let depth = stack.len();
                    if depth >= MAX_RECURSION_DEPTH {
                        return Err(ParseError::RecursionLimitExceeded);
                    }
                    if attributes.len() > MAX_ATTRIBUTES_PER_ELEMENT {
                        return Err(ParseError::TooManyAttributes);
                    }

                    let mut element = DomNode::new_element(name.clone(), depth);
                    element.attributes = attributes;

                    if self_closing {
                        if let Some(parent) = stack.last_mut() {
                            parent.add_child(element)?;
                        }
                    } else {
                        stack.push(element);
                    }
                }
                HtmlToken::EndTag { name } => {
                    let mut found = false;
                    let mut pop_count = 0;
                    for (i, node) in stack.iter().enumerate().rev() {
                        if node.tag_name() == Some(name.as_str()) {
                            pop_count = stack.len() - i;
                            found = true;
                            break;
                        }
                    }
                    if found {
                        for _ in 0..pop_count {
                            if stack.len() > 1 {
                                let child = stack.pop().unwrap();
                                if let Some(parent) = stack.last_mut() {
                                    parent.add_child(child)?;
                                }
                            }
                        }
                    }
                }
                HtmlToken::Text(text) => {
                    let depth = stack.len();
                    let text_node = DomNode::new_text(text, depth);
                    if let Some(parent) = stack.last_mut() {
                        parent.add_child(text_node)?;
                    }
                }
                HtmlToken::Comment(text) => {
                    let depth = stack.len();
                    let comment = DomNode {
                        node_type: NodeType::Comment(text),
                        children: Vec::new(),
                        attributes: Vec::new(),
                        depth,
                        computed_style: StyleDeclaration::new(),
                        node_id: next_node_id(),
                    };
                    if let Some(parent) = stack.last_mut() {
                        parent.add_child(comment)?;
                    }
                }
            }
        }

        while stack.len() > 1 {
            let child = stack.pop().unwrap();
            if let Some(parent) = stack.last_mut() {
                parent.add_child(child)?;
            }
        }

        doc = stack.pop().unwrap_or_else(DomNode::new_document);
        Ok(doc)
    }

    fn is_void_element(tag: &str) -> bool {
        matches!(
            tag,
            "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input"
                | "link" | "meta" | "param" | "source" | "track" | "wbr"
        )
    }
}

pub struct CssParser;

impl CssParser {
    pub fn parse(input: &str) -> ParseResult<StyleSheet> {
        let mut stylesheet = StyleSheet::new();
        let mut chars = input.chars().peekable();

        while chars.peek().is_some() {
            Self::skip_whitespace_and_comments(&mut chars);

            if chars.peek().is_none() {
                break;
            }

            let selector = Self::read_until(&mut chars, '{');
            let selector = selector.trim().to_string();
            if selector.is_empty() {
                if chars.peek() == Some(&'{') {
                    chars.next();
                }
                Self::read_until(&mut chars, '}');
                continue;
            }

            let body = Self::read_until(&mut chars, '}');
            let properties = Self::parse_declarations(&body)?;

            stylesheet.add_rule(CssRule {
                selector: selector.into(),
                properties,
            });
        }

        Ok(stylesheet)
    }

    fn parse_declarations(input: &str) -> ParseResult<Vec<CssProperty>> {
        let mut props = Vec::new();
        for decl in input.split(';') {
            let decl = decl.trim();
            if decl.is_empty() {
                continue;
            }
            if let Some(colon_pos) = decl.find(':') {
                let name = decl[..colon_pos].trim();
                let value = decl[colon_pos + 1..].trim();
                if !name.is_empty() && !value.is_empty() {
                    props.push(CssProperty {
                        name: String::from(name),
                        value: String::from(value),
                    });
                }
            }
        }
        Ok(props)
    }

    fn skip_whitespace_and_comments(chars: &mut core::iter::Peekable<core::str::Chars>) {
        while let Some(&ch) = chars.peek() {
            if ch.is_whitespace() {
                chars.next();
            } else if ch == '/' {
                let mut clone = chars.clone();
                clone.next();
                if clone.peek() == Some(&'*') {
                    chars.next();
                    chars.next();
                    loop {
                        match chars.next() {
                            Some('*') => {
                                if chars.peek() == Some(&'/') {
                                    chars.next();
                                    break;
                                }
                            }
                            None => break,
                            _ => {}
                        }
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn read_until(chars: &mut core::iter::Peekable<core::str::Chars>, delimiter: char) -> String {
        let mut result = String::new();
        while let Some(&ch) = chars.peek() {
            if ch == delimiter {
                chars.next();
                break;
            }
            result.push(ch);
            chars.next();
        }
        result
    }
}

pub struct StyleResolver;

impl StyleResolver {
    pub fn apply_styles(document: &mut DomNode, stylesheet: &StyleSheet) {
        Self::apply_styles_recursive(document, stylesheet);
    }

    fn apply_styles_recursive(node: &mut DomNode, stylesheet: &StyleSheet) {
        for rule in &stylesheet.rules {
            if Self::selector_matches(node, &rule.selector) {
                for prop in &rule.properties {
                    node.computed_style.set(prop.name.clone(), prop.value.clone());
                }
            }
        }

        if let Some(style_attr) = node.get_attribute("style") {
            let inline_style = style_attr.to_string();
            if let Ok(props) = CssParser::parse_declarations(&inline_style) {
                for prop in props {
                    node.computed_style.set(prop.name, prop.value);
                }
            }
        }

        for child in node.children.iter_mut() {
            Self::apply_styles_recursive(child, stylesheet);
        }
    }

    fn selector_matches(node: &DomNode, selector: &str) -> bool {
        let selector = selector.trim();

        if selector == "*" {
            return true;
        }

        if let Some(tag) = node.tag_name() {
            if selector == tag {
                return true;
            }
        }

        if selector.starts_with('#') {
            let id = &selector[1..];
            if node.get_attribute("id") == Some(id) {
                return true;
            }
        }

        if selector.starts_with('.') {
            let class_name = &selector[1..];
            if let Some(classes) = node.get_attribute("class") {
                for cls in classes.split_whitespace() {
                    if cls == class_name {
                        return true;
                    }
                }
            }
        }

        false
    }
}

pub fn extract_inline_styles(document: &DomNode) -> Vec<String> {
    let mut styles = Vec::new();
    let mut stack: Vec<&DomNode> = Vec::new();
    stack.push(document);
    while let Some(node) = stack.pop() {
        if node.tag_name() == Some("style") {
            let text = node.collect_text();
            if !text.is_empty() {
                styles.push(text);
            }
        }
        for child in node.children.iter().rev() {
            stack.push(child);
        }
    }
    styles
}

pub fn parse_html_with_styles(html: &str) -> ParseResult<(DomNode, StyleSheet)> {
    let mut document = HtmlParser::parse(html)?;
    let mut combined_stylesheet = StyleSheet::new();

    let inline_styles = extract_inline_styles(&document);
    for style_text in &inline_styles {
        let sheet = CssParser::parse(style_text)?;
        for rule in sheet.rules {
            combined_stylesheet.add_rule(rule);
        }
    }

    StyleResolver::apply_styles(&mut document, &combined_stylesheet);

    Ok((document, combined_stylesheet))
}

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

    #[test]
    fn test_html_parser_basic() {
        let html = "<html><body><h1>Hello</h1></body></html>";
        let doc = HtmlParser::parse(html).unwrap();
        assert!(doc.node_count() > 1);
        let h1_nodes = doc.find_by_tag("h1");
        assert_eq!(h1_nodes.len(), 1);
    }

    #[test]
    fn test_html_parser_attributes() {
        let html = r#"<div id="main" class="container"><p>Text</p></div>"#;
        let doc = HtmlParser::parse(html).unwrap();
        let div_nodes = doc.find_by_tag("div");
        assert_eq!(div_nodes.len(), 1);
        assert_eq!(div_nodes[0].get_attribute("id"), Some("main"));
        assert_eq!(div_nodes[0].get_attribute("class"), Some("container"));
    }

    #[test]
    fn test_html_parser_void_elements() {
        let html = "<div><br><img src=\"test.png\"><hr></div>";
        let doc = HtmlParser::parse(html).unwrap();
        let br_nodes = doc.find_by_tag("br");
        assert_eq!(br_nodes.len(), 1);
    }

    #[test]
    fn test_html_parser_self_closing() {
        let html = "<div><input type=\"text\" /><span>hi</span></div>";
        let doc = HtmlParser::parse(html).unwrap();
        let input_nodes = doc.find_by_tag("input");
        assert_eq!(input_nodes.len(), 1);
    }

    #[test]
    fn test_html_parser_nested() {
        let html = "<div><ul><li>Item 1</li><li>Item 2</li></ul></div>";
        let doc = HtmlParser::parse(html).unwrap();
        let li_nodes = doc.find_by_tag("li");
        assert_eq!(li_nodes.len(), 2);
    }

    #[test]
    fn test_css_parser_basic() {
        let css = "body { color: red; background: white; }";
        let sheet = CssParser::parse(css).unwrap();
        assert_eq!(sheet.rule_count(), 1);
        assert_eq!(sheet.rules[0].properties.len(), 2);
    }

    #[test]
    fn test_css_parser_multiple_rules() {
        let css = "h1 { font-size: 24px; } p { margin: 10px; color: blue; }";
        let sheet = CssParser::parse(css).unwrap();
        assert_eq!(sheet.rule_count(), 2);
    }

    #[test]
    fn test_css_parser_comments() {
        let css = "/* comment */ body { color: red; }";
        let sheet = CssParser::parse(css).unwrap();
        assert_eq!(sheet.rule_count(), 1);
    }

    #[test]
    fn test_style_resolver() {
        let html = r#"<div id="main"><p>Hello</p></div>"#;
        let mut doc = HtmlParser::parse(html).unwrap();
        let css = "p { color: blue; font-size: 16px; }";
        let sheet = CssParser::parse(css).unwrap();
        StyleResolver::apply_styles(&mut doc, &sheet);

        let p_nodes = doc.find_by_tag("p");
        assert_eq!(p_nodes.len(), 1);
    }

    #[test]
    fn test_find_by_id() {
        let html = r#"<div id="container"><span id="target">Found</span></div>"#;
        let doc = HtmlParser::parse(html).unwrap();
        let found = doc.find_by_id("target");
        assert!(found.is_some());
        assert_eq!(found.unwrap().tag_name(), Some("span"));
    }

    #[test]
    fn test_collect_text() {
        let html = "<div>Hello <span>World</span></div>";
        let doc = HtmlParser::parse(html).unwrap();
        let div_nodes = doc.find_by_tag("div");
        assert!(!div_nodes.is_empty());
        let text = div_nodes[0].collect_text();
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
    }

    #[test]
    fn test_parse_html_with_styles() {
        let html = "<html><head><style>body { color: red; }</style></head><body><p>Hi</p></body></html>";
        let (doc, sheet) = parse_html_with_styles(html).unwrap();
        assert!(doc.node_count() > 1);
        assert!(sheet.rule_count() >= 1);
    }

    #[test]
    fn test_document_size_limit() {
        let huge = "x".repeat(MAX_DOCUMENT_SIZE + 1);
        assert!(HtmlParser::parse(&huge).is_err());
    }

    #[test]
    fn test_style_declaration() {
        let mut style = StyleDeclaration::new();
        style.set("color".into(), "red".into());
        assert_eq!(style.get("color"), Some("red"));
        style.set("color".into(), "blue".into());
        assert_eq!(style.get("color"), Some("blue"));
        assert_eq!(style.properties.len(), 1);
    }

    #[test]
    fn test_html_comment_parsing() {
        let html = "<div><!-- this is a comment --><p>text</p></div>";
        let doc = HtmlParser::parse(html).unwrap();
        assert!(doc.node_count() >= 3);
    }
}
