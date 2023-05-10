mod cmd;
mod highlighter;
mod html;
mod replacer;
mod utils;

use std::{fs::File, io::Read, path::PathBuf};

use chrono::Utc;
use clap::Parser;
use cmd::Command;
use comrak::{
    nodes::{AstNode, ListType, NodeValue, TableAlignment},
    Arena, ComrakExtensionOptions, ComrakOptions, ComrakParseOptions, ComrakRenderOptions,
};

use html::{Element, Meta};
use once_cell::sync::Lazy;
use syntect::{
    highlighting::{Theme, ThemeSet},
    parsing::SyntaxSet,
};
use utils::must;

static THEME: Lazy<Theme> = Lazy::new(|| {
    let ts = ThemeSet::load_defaults();
    ts.themes["base16-eighties.dark"].clone()
});

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| SyntaxSet::load_defaults_newlines());

fn iter_nodes<'a>(node: &'a AstNode<'a>, state: &mut utils::State) -> Element {
    match &node.data.borrow().value {
        NodeValue::Document => Element::Section(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::FrontMatter(front_matter) => {
            let trimmed = front_matter.trim_matches(|c: char| c == '+' || c.is_whitespace());
            state.front_matter = Some(must(toml::from_str::<utils::FrontMatter>(trimmed)));
            state.date = Utc::now();

            Element::Empty
        }

        NodeValue::BlockQuote => Element::Blockquote(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::List(list) => {
            let children = node
                .children()
                .map(|child| iter_nodes(child, state))
                .collect();

            match list.list_type {
                ListType::Bullet => Element::Ul(Meta::new().with_children(children)),
                ListType::Ordered => Element::Ol(Meta::new().with_children(children)),
            }
        }

        NodeValue::Item(_) => Element::Li(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::DescriptionList => {
            let mut children = Vec::new();
            for item in node.children() {
                for child in item.children() {
                    children.push(iter_nodes(child, state));
                }
            }

            Element::Dl(Meta::new().with_children(children))
        }

        NodeValue::DescriptionItem(_) => Element::Empty,

        NodeValue::DescriptionTerm => Element::Dt(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::DescriptionDetails => Element::Dd(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::CodeBlock(code_block) => {
            highlighter::highlight_code(&code_block.literal, &code_block.info, &SYNTAX_SET, &THEME)
        }

        NodeValue::HtmlBlock(html_block) => Element::Raw(html_block.literal.clone()),

        NodeValue::Paragraph => Element::P(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::Heading(heading) => {
            let mut children = node
                .children()
                .map(|child| iter_nodes(child, state))
                .collect::<Vec<_>>();

            let (id, title) = utils::heading_to_slug(&children);

            state.headings.push((heading.level, id.clone(), title));

            children.push(Element::A(
                Meta::new()
                    .with_child(Element::Text("&nbsp;&sect;".into()))
                    .with_attrs(vec![
                        format!("href=\"#heading__{id}\""),
                        "class=\"section-logo\"".into(),
                    ]),
            ));

            let meta = Meta::new()
                .with_attr(&format!("id=\"heading__{id}\""))
                .with_children(children);

            match heading.level {
                1 => Element::H1(meta),
                2 => Element::H2(meta),
                3 => Element::H3(meta),
                4 => Element::H4(meta),
                5 => Element::H5(meta),
                6 => Element::H6(meta),
                _ => unreachable!(),
            }
        }

        NodeValue::ThematicBreak => Element::Hr(Meta::default()),

        NodeValue::FootnoteDefinition(definition) => {
            let mut children = Vec::new();
            for child in node.children() {
                children.push(iter_nodes(child, state));
            }

            state.definitions.push((definition.clone(), children));
            Element::Empty
        }

        NodeValue::Table(alignments) => {
            let mut children = Vec::new();

            state.table_counter += 1;

            fn alignment_to_str(alignment: &TableAlignment) -> String {
                (match alignment {
                    TableAlignment::Center => "center",
                    TableAlignment::Left => "left",
                    TableAlignment::None => "initial",
                    TableAlignment::Right => "right",
                })
                .into()
            }

            alignments.iter().enumerate().for_each(|(i, alignment)| {
                state.styles.push(format!(
                    ".table-{0} td:nth-child({1}), .table-{0} th:nth-child({1}) {{ text-align: {2} }}",
                    state.table_counter,
                    i + 1,
                    alignment_to_str(alignment)
                ));
            });

            let table_rows = node.children().collect::<Vec<_>>();

            let header_row = iter_nodes(table_rows[0], state);

            for &child in &table_rows[1..] {
                let el = iter_nodes(child, state);
                children.push(el);
            }

            let meta = Meta::new()
                .with_children(vec![
                    Element::Thead(Meta::new().with_child(header_row)),
                    Element::Tbody(Meta::new().with_children(children)),
                ])
                .with_attr(&format!("class=\"table-{}\"", state.table_counter));

            Element::Table(meta)
        }

        NodeValue::TableRow(header) => {
            let mut children = Vec::new();
            if *header {
                for table_head in node.children() {
                    let mut table_children = Vec::new();
                    for child in table_head.children() {
                        table_children.push(iter_nodes(child, state));
                    }
                    children.push(Element::Th(Meta::new().with_children(table_children)))
                }
            } else {
                for child in node.children() {
                    children.push(iter_nodes(child, state));
                }
            }

            Element::Tr(Meta::new().with_children(children))
        }

        NodeValue::TableCell => Element::Td(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::Text(text) => {
            state.word_count += text.split_whitespace().collect::<Vec<_>>().len();

            Element::Text(replacer::replace_emoticons(&replacer::replace_typographer(
                &text,
            )))
        }

        NodeValue::TaskItem(ch) => {
            let mut children = Vec::new();

            if let Some(ch) = ch {
                children.push(utils::char_to_taskitem(*ch))
            } else {
                children.push(Element::Span(
                    Meta::new().with_attr(r#"class="fa-regular fa-square""#),
                ))
            }

            for paragraph in node.children() {
                for child in paragraph.children() {
                    children.push(iter_nodes(child, state));
                }
            }

            Element::Li(
                Meta::new()
                    .with_children(children)
                    .with_attr("class=\"task-item\"".into()),
            )
        }

        NodeValue::SoftBreak => Element::Br(Meta::default()),

        NodeValue::LineBreak => Element::Br(Meta::default()),

        NodeValue::Code(code) => Element::Span(
            Meta::new()
                .with_child(Element::Text(code.literal.clone()))
                .with_attr("class=\"inline-code\"".into()),
        ),

        NodeValue::HtmlInline(html_code) => Element::Raw(html_code.clone()),

        NodeValue::Emph => Element::I(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::Strong => Element::B(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::Strikethrough => Element::S(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::Insert => Element::U(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::Superscript => Element::Sup(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::Link(link) => {
            let mut children = node
                .children()
                .map(|child| iter_nodes(child, state))
                .collect::<Vec<_>>();

            if let Ok(href) = url::Url::parse(&link.url) {
                if let Some(domain) = href.domain() {
                    if domain != state.domain.as_str() {
                        children.push(Element::Span(Meta::new().with_attr(
                            "class=\"fa-solid fa-up-right-from-square href-external\"".into(),
                        )))
                    }
                };
            }

            Element::A(Meta::new().with_children(children).with_attrs(vec![
                format!("href=\"{}\"", link.url),
                format!("title=\"{}\"", link.title),
                "target=\"_blank\"".into(),
                "rel=\"noreferrer\"".into(),
            ]))
        }

        NodeValue::Image(img) => {
            let mut attrs = vec![format!("src=\"{}\"", img.url)];
            if let NodeValue::Text(text) =
                &node.children().collect::<Vec<_>>()[0].data.borrow().value
            {
                attrs.push(format!("alt=\"{}\"", text))
            }

            if !img.title.is_empty() {
                attrs.push(format!("title=\"{}\"", img.title));
                Element::Figure(Meta::new().with_children(vec![
                    Element::Img(Meta::new().with_attrs(attrs)),
                    Element::Figcaption(Meta::new().with_child(Element::Text(img.title.clone()))),
                ]))
            } else {
                Element::Img(Meta::new().with_attrs(attrs))
            }
        }

        NodeValue::FootnoteReference(reference) => {
            let mut tag = String::new();
            state
                .footnote_counter
                .entry(reference.clone())
                .and_modify(|counter| {
                    tag = format!(":{}", counter);
                    *counter += 1;
                })
                .or_insert(1);

            Element::Sup(
                Meta::new().with_child(Element::A(
                    Meta::new()
                        .with_child(Element::Text(format!("[{reference}{tag}]")))
                        .with_attrs(vec![
                            format!("href=\"#footnote-definition-{reference}\""),
                            format!("id=\"footnote-reference-{reference}{tag}\""),
                        ]),
                )),
            )
        }

        NodeValue::ShortCode(short_code) => {
            Element::Span(Meta::new().with_child(Element::Text(short_code.emoji().into())))
        }

        NodeValue::Subscript => Element::Sub(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::Highlight => Element::Mark(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),
    }
}

fn main() {
    let arena = Arena::new();

    let cmd = Command::parse();

    let options = ComrakOptions {
        extension: ComrakExtensionOptions {
            strikethrough: true,
            tagfilter: true,
            table: true,
            autolink: true,
            tasklist: true,
            superscript: true,
            header_ids: None,
            footnotes: true,
            description_lists: true,
            front_matter_delimiter: Some(String::from("+++")),
            shortcodes: true,
            subscript: true,
            highlight: true,
            insert: true,
        },

        parse: ComrakParseOptions {
            default_info_string: Some(String::from("txt")),
            relaxed_tasklist_matching: true,
            smart: true,
        },

        render: ComrakRenderOptions {
            escape: true,
            ..ComrakRenderOptions::default()
        },
    };

    let mut file = must(File::open(&cmd.file_path));

    let mut buf = String::new();
    must(file.read_to_string(&mut buf));

    let root = comrak::parse_document(&arena, &buf, &options);

    let mut state = utils::State::default();
    state.domain.clone_from(&cmd.domain_name);

    state.styles.push(include_str!("styles.css").to_string());

    let html = utils::init(iter_nodes(root, &mut state), state);

    let out_dir = PathBuf::from(&cmd.out_dir);
    let out_path = out_dir
        .join(must(
            cmd.file_path.file_stem().ok_or_else(|| "No filename found"),
        ))
        .with_extension("html");

    if !out_path.exists() {
        must(std::fs::create_dir_all(&cmd.out_dir));
    }

    must(std::fs::write(&out_path, html.to_html()));

    println!(
        "Written output to \"{}\"",
        must(std::env::current_dir()).join(&out_path).display()
    );

    if cmd.output_ast {
        must(std::fs::write(
            &out_path.with_extension("md.ast"),
            format!("{:#?}", root),
        ));
        must(std::fs::write(
            &out_path.with_extension("html.ast"),
            format!("{:#?}", html),
        ));
    }
}
