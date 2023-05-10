use crate::html::{Meta, Tag};
use crate::utils::must;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, Style, Theme};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

pub fn highlight_code(code: &str, lang: &str, ps: &SyntaxSet, theme: &Theme) -> Tag {
    let mut children = Vec::new();

    let mut is_plain_text = false;

    let syntax = ps.find_syntax_by_token(lang).unwrap_or_else(|| {
        is_plain_text = true;
        ps.find_syntax_plain_text()
    });

    let mut h = HighlightLines::new(syntax, theme);

    let lines = LinesWithEndings::from(code).collect::<Vec<_>>();
    let n = lines.len();

    let width = n.to_string().len();

    for (line_number, line) in lines.iter().enumerate() {
        let mut cur_line_children = Vec::new();
        for (
            Style {
                foreground: Color { r, g, b, a },
                ..
            },
            text,
        ) in must(h.highlight_line(line, &ps))
        {
            cur_line_children.push(Tag::Span(
                Meta::new()
                    .with_child(Tag::Text(text.to_string()))
                    .with_attr(&format!("style=\"color: #{r:02x}{g:02x}{b:02x}{a:02x};\"")),
            ))
        }

        children.push(Tag::Div(Meta::new().with_children(if is_plain_text {
            vec![Tag::Div(Meta::new().with_children(cur_line_children))]
        } else {
            vec![
                Tag::Span(
                    Meta::new()
                        .with_children(vec![Tag::Text(format!("{:>width$}.", line_number + 1))])
                        .with_attr("class=\"code-line-number\""),
                ),
                Tag::Div(
                    Meta::new()
                        .with_children(cur_line_children)
                        .with_attr(&format!("style=\"padding-left: {}px\"", 25 + width * 10)),
                ),
            ]
        })))
    }

    Tag::Pre(
        Meta::new()
            .with_children(vec![
                if !is_plain_text {
                    Tag::Div(Meta::new().with_child(Tag::Text(syntax.name.clone())))
                } else {
                    Tag::Empty
                },
                Tag::Code(Meta::new().with_children(children)),
            ])
            .with_attr(&format!(
                "style=\"padding: {}px 20px 20px 20px\"",
                if is_plain_text { 20 } else { 40 }
            )),
    )
}
