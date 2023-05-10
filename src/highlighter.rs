use crate::html::{Element, Meta};

use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, Style, Theme};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

pub fn highlight_code(code: &str, lang: &str, ps: &SyntaxSet, theme: &Theme) -> Element {
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
        ) in h.highlight_line(line, &ps).unwrap()
        {
            cur_line_children.push(Element::Span(
                Meta::new()
                    .with_child(Element::Text(
                        text.replace(">", "&gt;").replace("<", "&lt;"),
                    ))
                    .with_attr(&format!("style=\"color: #{r:02x}{g:02x}{b:02x}{a:02x};\"")),
            ))
        }

        children.push(Element::Div(Meta::new().with_children(if is_plain_text {
            vec![Element::Div(Meta::new().with_children(cur_line_children))]
        } else {
            vec![
                Element::Span(
                    Meta::new()
                        .with_children(vec![Element::Text(format!("{:>width$}.", line_number + 1))])
                        .with_attr("class=\"code-line-number\""),
                ),
                Element::Div(
                    Meta::new()
                        .with_children(cur_line_children)
                        .with_attr(&format!("style=\"padding-left: {}px\"", 25 + width * 10)),
                ),
            ]
        })))
    }

    Element::Pre(
        Meta::new()
            .with_children(vec![
                if !is_plain_text {
                    Element::Div(Meta::new().with_child(Element::Text(syntax.name.clone())))
                } else {
                    Element::Empty
                },
                Element::Code(Meta::new().with_children(children)),
            ])
            .with_attr(&format!(
                "style=\"padding: {}px 20px 20px 20px\"",
                if is_plain_text { 20 } else { 40 }
            )),
    )
}
