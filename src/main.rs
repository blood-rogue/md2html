mod cmd;
mod highlighter;
mod html;
mod replacer;
mod utils;

use std::fs::{create_dir_all, read_to_string, write};

use chrono::Utc;
use clap::Parser;
use cmd::Command;
use colored::Colorize;
use comrak::{
    nodes::{AstNode, ListType, NodeValue, TableAlignment},
    Arena, ComrakExtensionOptions, ComrakOptions, ComrakParseOptions, ComrakRenderOptions,
};

use css_minify::optimizations::{Level, Minifier};
use html::{Meta, Tag};
use once_cell::sync::Lazy;
use syntect::{
    highlighting::{Theme, ThemeSet},
    parsing::SyntaxSet,
};
use utils::{len_to_size, must};

static THEME: Lazy<Theme> = Lazy::new(|| {
    let ts = ThemeSet::load_defaults();
    ts.themes["base16-eighties.dark"].clone()
});

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| SyntaxSet::load_defaults_newlines());

fn iter_nodes<'a>(node: &'a AstNode<'a>, state: &mut utils::State) -> Tag {
    match &node.data.borrow().value {
        NodeValue::Document => Tag::Section(
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

            Tag::Empty
        }

        NodeValue::BlockQuote => Tag::Blockquote(
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
                ListType::Bullet => Tag::Ul(Meta::new().with_children(children)),
                ListType::Ordered => Tag::Ol(Meta::new().with_children(children)),
            }
        }

        NodeValue::Item(_) => Tag::Li(
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

            Tag::Dl(Meta::new().with_children(children))
        }

        NodeValue::DescriptionItem(_) => Tag::Empty,

        NodeValue::DescriptionTerm => Tag::Dt(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::DescriptionDetails => Tag::Dd(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::CodeBlock(code_block) => {
            highlighter::highlight_code(&code_block.literal, &code_block.info, &SYNTAX_SET, &THEME)
        }

        NodeValue::HtmlBlock(html_block) => Tag::Raw(html_block.literal.clone()),

        NodeValue::Paragraph => Tag::P(
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

            children.push(Tag::A(
                Meta::new()
                    .with_child(Tag::Text("&sect;".into()))
                    .with_attrs(vec![
                        format!("href=\"#heading__{id}\""),
                        "class=\"section-logo\"".into(),
                    ]),
            ));

            let meta = Meta::new()
                .with_attr(&format!("id=\"heading__{id}\""))
                .with_children(children);

            match heading.level {
                1 => Tag::H1(meta),
                2 => Tag::H2(meta),
                3 => Tag::H3(meta),
                4 => Tag::H4(meta),
                5 => Tag::H5(meta),
                6 => Tag::H6(meta),
                _ => unreachable!(),
            }
        }

        NodeValue::ThematicBreak => Tag::Hr(Meta::default()),

        NodeValue::FootnoteDefinition(definition) => {
            let mut children = Vec::new();
            for child in node.children() {
                children.push(iter_nodes(child, state));
            }

            state.definitions.push((definition.clone(), children));
            Tag::Empty
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
                    Tag::Thead(Meta::new().with_child(header_row)),
                    Tag::Tbody(Meta::new().with_children(children)),
                ])
                .with_attr(&format!("class=\"table-{}\"", state.table_counter));

            Tag::Table(meta)
        }

        NodeValue::TableRow(header) => {
            let mut children = Vec::new();
            if *header {
                for table_head in node.children() {
                    let mut table_children = Vec::new();
                    for child in table_head.children() {
                        table_children.push(iter_nodes(child, state));
                    }
                    children.push(Tag::Th(Meta::new().with_children(table_children)))
                }
            } else {
                for child in node.children() {
                    children.push(iter_nodes(child, state));
                }
            }

            Tag::Tr(Meta::new().with_children(children))
        }

        NodeValue::TableCell => Tag::Td(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::Text(text) => {
            state.word_count += text.split_whitespace().collect::<Vec<_>>().len();

            Tag::Text(replacer::replace_emoticons(&replacer::replace_typographer(
                &text,
            )))
        }

        NodeValue::TaskItem(ch) => {
            let mut children = Vec::new();

            if let Some(ch) = ch {
                children.push(utils::char_to_taskitem(*ch))
            } else {
                children.push(Tag::Span(
                    Meta::new().with_attr(r#"class="fa-regular fa-square""#),
                ))
            }

            for paragraph in node.children() {
                for child in paragraph.children() {
                    children.push(iter_nodes(child, state));
                }
            }

            Tag::Li(
                Meta::new()
                    .with_children(children)
                    .with_attr("class=\"task-item\"".into()),
            )
        }

        NodeValue::SoftBreak => Tag::Br(Meta::default()),

        NodeValue::LineBreak => Tag::Br(Meta::default()),

        NodeValue::Code(code) => Tag::Span(
            Meta::new()
                .with_child(Tag::Text(code.literal.clone()))
                .with_attr("class=\"inline-code\"".into()),
        ),

        NodeValue::HtmlInline(html_code) => Tag::Raw(html_code.clone()),

        NodeValue::Emph => Tag::I(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::Strong => Tag::B(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::Strikethrough => Tag::S(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::Insert => Tag::U(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::Superscript => Tag::Sup(
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
                        children.push(Tag::Span(Meta::new().with_attr(
                            "class=\"fa-solid fa-up-right-from-square href-external\"".into(),
                        )))
                    }
                };
            }

            Tag::A(Meta::new().with_children(children).with_attrs(vec![
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
                Tag::Figure(Meta::new().with_children(vec![
                    Tag::Img(Meta::new().with_attrs(attrs)),
                    Tag::Figcaption(Meta::new().with_child(Tag::Text(img.title.clone()))),
                ]))
            } else {
                Tag::Img(Meta::new().with_attrs(attrs))
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

            Tag::Sup(
                Meta::new().with_child(Tag::A(
                    Meta::new()
                        .with_child(Tag::Text(format!("[{reference}{tag}]")))
                        .with_attrs(vec![
                            format!("href=\"#footnote-definition-{reference}\""),
                            format!("id=\"footnote-reference-{reference}{tag}\""),
                        ]),
                )),
            )
        }

        NodeValue::ShortCode(short_code) => {
            Tag::Span(Meta::new().with_child(Tag::Text(short_code.emoji().into())))
        }

        NodeValue::Subscript => Tag::Sub(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),

        NodeValue::Highlight => Tag::Mark(
            Meta::new().with_children(
                node.children()
                    .map(|child| iter_nodes(child, state))
                    .collect(),
            ),
        ),
    }
}

fn get_logger(verbose: bool) -> impl Fn(String) {
    let f = if verbose {
        |info: String| println!("{}", format!("[INFO]: {}", info).bright_blue())
    } else {
        |_| {}
    };

    f
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

    let logger = get_logger(cmd.verbose);

    let buf = must(read_to_string(&cmd.file_path));
    logger(format!(
        "Read ({}) markdown file \"{}\"",
        must(len_to_size(buf.len())),
        cmd.file_path.display()
    ));

    let root = comrak::parse_document(&arena, &buf, &options);
    logger("Parsed markdown file".to_string());

    let mut state = utils::State::default();
    state.domain.clone_from(&cmd.domain_name);

    let authors_db = must(std::fs::read_to_string(&cmd.authors_db));
    logger(format!(
        "Read ({} bytes) authors db file \"{}\"",
        must(len_to_size(authors_db.len())),
        &cmd.authors_db
    ));

    state.authors = must(toml::from_str(&authors_db));
    logger("Parsed authors db file".to_string());

    let html = utils::init(iter_nodes(root, &mut state), state);
    logger("Generated HTML AST".into());

    let out_dir = must(std::env::current_dir()).join(&cmd.out_dir);
    let out_path = out_dir
        .join(must(
            cmd.file_path.file_stem().ok_or_else(|| "No filename found"),
        ))
        .with_extension("html");

    if !out_path.exists() {
        must(create_dir_all(&cmd.out_dir));
        logger(format!("Created output directory \"{}\"", cmd.out_dir));
    }

    if cmd.output_ast {
        must(write(
            &out_path.with_extension("md.ast"),
            format!("{:#?}", root),
        ));
        logger(format!(
            "Written Markdown AST to \"{}\"",
            must(std::env::current_dir())
                .join(&out_path)
                .with_extension("md.ast")
                .display()
        ));

        must(write(
            &out_path.with_extension("html.ast"),
            format!("{:#?}", html),
        ));
        logger(format!(
            "Written HTML AST to \"{}\"",
            must(std::env::current_dir())
                .join(&out_path)
                .with_extension("html.ast")
                .display()
        ));
    }

    let html = html.to_html();

    must(write(&out_path, &html));

    logger(format!(
        "Written ({}) HTML to \"{}\"",
        must(len_to_size(html.len())),
        must(std::env::current_dir()).join(&out_path).display()
    ));

    let logo_path = out_dir.join("logo.png");

    if !logo_path.exists() || cmd.force {
        must(std::fs::copy(&cmd.logo, &logo_path));

        logger(format!(
            "Written ({}) logo to \"{}\"",
            must(len_to_size(must(logo_path.metadata()).len() as usize)),
            logo_path.display()
        ));
    }

    let styles_path = out_dir.join("styles.css");

    if !styles_path.exists() || cmd.force {
        let stylesheet =
            must(Minifier::default().minify(&must(read_to_string(&cmd.style_sheet)), Level::One))
                .replace(":-webkit-", "::-webkit-"); // Optimization Level::Three breaks style rules as it replaces `::-webkit-*` with `:-webkit-*`

        must(std::fs::write(&styles_path, &stylesheet));
        logger(format!(
            "Written ({}) stylesheet to \"{}\"",
            must(len_to_size(stylesheet.len())),
            styles_path.display()
        ));
    }
}
