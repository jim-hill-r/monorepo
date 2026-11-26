//! A small Markdown parser for the cookbook's prose content, such as
//! `content/intro.md`.
//!
//! It turns Markdown into [`Block`]s that a UI renders as its own elements,
//! so no HTML string is ever injected into the page. It supports the subset
//! the cookbook's prose uses:
//!
//! - paragraphs separated by blank lines, with soft-wrapped lines joined by
//!   a space;
//! - ATX headings (`#` to `######`);
//! - `**strong**` and `*emphasis*` (`__strong__` and `_emphasis_` too, but
//!   only when the underscores are not inside a word, so `snake_case` stays
//!   as written).
//!
//! Anything else, such as an unmatched `*`, is kept as literal text.

/// A run of inline text with its formatting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    /// Plain text.
    Text(String),
    /// Emphasised text (`*text*`), usually shown in italics.
    Emphasis(String),
    /// Strong text (`**text**`), usually shown in bold.
    Strong(String),
}

/// A block of Markdown content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// A heading; `level` is 1 to 6.
    Heading { level: u8, content: Vec<Inline> },
    /// A paragraph of inline text.
    Paragraph(Vec<Inline>),
}

/// Parses Markdown into blocks. See the module docs for the supported subset.
pub fn parse(markdown: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut paragraph: Vec<&str> = Vec::new();

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            flush_paragraph(&mut paragraph, &mut blocks);
        } else if let Some((level, text)) = heading(trimmed) {
            flush_paragraph(&mut paragraph, &mut blocks);
            blocks.push(Block::Heading {
                level,
                content: parse_inline(text),
            });
        } else {
            paragraph.push(trimmed);
        }
    }
    flush_paragraph(&mut paragraph, &mut blocks);
    blocks
}

fn flush_paragraph(lines: &mut Vec<&str>, blocks: &mut Vec<Block>) {
    if !lines.is_empty() {
        blocks.push(Block::Paragraph(parse_inline(&lines.join(" "))));
        lines.clear();
    }
}

/// Returns the level and text of an ATX heading line, if it is one.
fn heading(line: &str) -> Option<(u8, &str)> {
    let level = line.chars().take_while(|&c| c == '#').count();
    if level == 0 || level > 6 {
        return None;
    }
    let rest = &line[level..];
    if !rest.is_empty() && !rest.starts_with([' ', '\t']) {
        return None;
    }
    let text = rest.trim().trim_end_matches('#').trim_end();
    u8::try_from(level).ok().map(|level| (level, text))
}

/// Parses `**strong**` and `*emphasis*` spans in a line of text.
pub fn parse_inline(text: &str) -> Vec<Inline> {
    let mut spans = Vec::new();
    let mut plain = String::new();
    let mut rest = text;

    while let Some(start) = rest.find(['*', '_']) {
        let (before, from_marker) = rest.split_at(start);
        match delimited_span(before, from_marker) {
            Some((span, after)) => {
                plain.push_str(before);
                push_text(&mut spans, &mut plain);
                spans.push(span);
                rest = after;
            }
            None => {
                // Not a span: keep the marker character as text and move on.
                let marker_len = from_marker.chars().next().map_or(1, char::len_utf8);
                plain.push_str(&rest[..start + marker_len]);
                rest = &from_marker[marker_len..];
            }
        }
    }
    plain.push_str(rest);
    push_text(&mut spans, &mut plain);
    spans
}

fn push_text(spans: &mut Vec<Inline>, plain: &mut String) {
    if !plain.is_empty() {
        spans.push(Inline::Text(std::mem::take(plain)));
    }
}

/// Tries to read a strong or emphasis span at the start of `text`, which
/// begins with `*` or `_`. `before` is the text preceding it on the line.
/// Returns the span and the text after its closing marker.
fn delimited_span<'a>(before: &str, text: &'a str) -> Option<(Inline, &'a str)> {
    let marker = text.chars().next()?;
    let strong = text[1..].starts_with(marker);
    let delimiter = if strong { &text[..2] } else { &text[..1] };
    let inner_start = delimiter.len();

    // Underscores only count at word boundaries.
    if marker == '_' && before.chars().last().is_some_and(char::is_alphanumeric) {
        return None;
    }
    let body = &text[inner_start..];
    if body.starts_with(char::is_whitespace) || body.is_empty() {
        return None;
    }

    let mut search_from = 0;
    while let Some(offset) = body[search_from..].find(delimiter) {
        let close = search_from + offset;
        let after = &body[close + delimiter.len()..];
        let inner = &body[..close];
        let closes = !inner.is_empty()
            && !inner.ends_with(char::is_whitespace)
            // `*a**` is not emphasis closed by the first `*` of `**`.
            && !(!strong && after.starts_with(marker))
            && !(marker == '_' && after.starts_with(char::is_alphanumeric));
        if closes {
            let inner = inner.to_string();
            let span = if strong {
                Inline::Strong(inner)
            } else {
                Inline::Emphasis(inner)
            };
            return Some((span, after));
        }
        search_from = close + delimiter.len();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(s: &str) -> Inline {
        Inline::Text(s.to_string())
    }

    #[test]
    fn empty_input_has_no_blocks() {
        assert_eq!(parse(""), vec![]);
        assert_eq!(parse("\n\n  \n"), vec![]);
    }

    #[test]
    fn soft_wrapped_lines_join_into_one_paragraph() {
        let blocks = parse("First line\nsecond line.\n");
        assert_eq!(
            blocks,
            vec![Block::Paragraph(vec![text("First line second line.")])]
        );
    }

    #[test]
    fn blank_lines_separate_paragraphs() {
        let blocks = parse("One.\n\nTwo\nthree.\n\n\nFour.");
        assert_eq!(
            blocks,
            vec![
                Block::Paragraph(vec![text("One.")]),
                Block::Paragraph(vec![text("Two three.")]),
                Block::Paragraph(vec![text("Four.")]),
            ]
        );
    }

    #[test]
    fn headings_have_levels_and_end_paragraphs() {
        let blocks = parse("# Title\nIntro\n### Sub ###\nMore");
        assert_eq!(
            blocks,
            vec![
                Block::Heading {
                    level: 1,
                    content: vec![text("Title")]
                },
                Block::Paragraph(vec![text("Intro")]),
                Block::Heading {
                    level: 3,
                    content: vec![text("Sub")]
                },
                Block::Paragraph(vec![text("More")]),
            ]
        );
    }

    #[test]
    fn hashes_without_a_space_or_more_than_six_are_text() {
        assert_eq!(
            parse("#hashtag"),
            vec![Block::Paragraph(vec![text("#hashtag")])]
        );
        assert_eq!(
            parse("####### seven"),
            vec![Block::Paragraph(vec![text("####### seven")])]
        );
    }

    #[test]
    fn strong_and_emphasis_spans() {
        assert_eq!(
            parse_inline("a **bold** and *soft* word"),
            vec![
                text("a "),
                Inline::Strong("bold".to_string()),
                text(" and "),
                Inline::Emphasis("soft".to_string()),
                text(" word"),
            ]
        );
        assert_eq!(
            parse_inline("__bold__ _soft_"),
            vec![
                Inline::Strong("bold".to_string()),
                text(" "),
                Inline::Emphasis("soft".to_string()),
            ]
        );
    }

    #[test]
    fn unmatched_or_spaced_markers_stay_literal() {
        assert_eq!(parse_inline("5 * 3 = 15"), vec![text("5 * 3 = 15")]);
        assert_eq!(parse_inline("a *dangling"), vec![text("a *dangling")]);
        assert_eq!(parse_inline("**"), vec![text("**")]);
        assert_eq!(parse_inline("* not *"), vec![text("* not *")]);
    }

    #[test]
    fn underscores_inside_words_stay_literal() {
        assert_eq!(
            parse_inline("snake_case_name"),
            vec![text("snake_case_name")]
        );
    }

    #[test]
    fn non_ascii_text_around_markers() {
        assert_eq!(
            parse_inline("crème *brûlée* — yum"),
            vec![
                text("crème "),
                Inline::Emphasis("brûlée".to_string()),
                text(" — yum"),
            ]
        );
    }

    #[test]
    fn quotes_are_kept_as_written() {
        assert_eq!(
            parse_inline("\"The Book\" by me"),
            vec![text("\"The Book\" by me")]
        );
    }
}
