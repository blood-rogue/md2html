use std::{
    error::Error,
    io::{BufWriter, Write},
};

#[derive(Debug, Default, Clone)]
pub struct Meta {
    attrs: Vec<String>,
    children: Vec<Element>,
}

impl Meta {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_children(&self, children: Vec<Element>) -> Self {
        Self {
            attrs: self.attrs.clone(),
            children,
        }
    }

    pub fn with_child(&self, child: Element) -> Self {
        Self {
            attrs: self.attrs.clone(),
            children: vec![child],
        }
    }

    pub fn with_attrs(&self, attrs: Vec<String>) -> Self {
        Self {
            attrs,
            children: self.children.clone(),
        }
    }

    pub fn with_attr(&self, attr: &str) -> Self {
        Self {
            attrs: Vec::from([attr.to_string()]),
            children: self.children.clone(),
        }
    }

    pub fn get_children(&self) -> Vec<Element> {
        self.children.clone()
    }
}

#[derive(Debug, Clone)]
pub enum Element {
    Doctype(Meta),
    Html(Box<Element>, Box<Element>),
    Head(Meta),
    Title(Meta),
    Link(Meta),
    Meta(Meta),
    Body(Meta),
    H1(Meta),
    H2(Meta),
    H3(Meta),
    H4(Meta),
    H5(Meta),
    H6(Meta),
    P(Meta),
    Hr(Meta),
    Pre(Meta),
    Blockquote(Meta),
    Ol(Meta),
    Ul(Meta),
    Li(Meta),
    Dl(Meta),
    Dt(Meta),
    Dd(Meta),
    Div(Meta),
    Table(Meta),
    Tr(Meta),
    Th(Meta),
    Td(Meta),
    Img(Meta),
    A(Meta),
    I(Meta),
    B(Meta),
    S(Meta),
    Sub(Meta),
    Sup(Meta),
    Code(Meta),
    Mark(Meta),
    Span(Meta),
    Br(Meta),
    Style(Meta),
    Section(Meta),
    U(Meta),
    Details(Meta),
    Summary(Meta),
    Thead(Meta),
    Tbody(Meta),

    Comment(String),

    Text(String),
    Raw(String),
    Empty,
}

impl Element {
    fn tag_name(&self) -> String {
        let tag = match self {
            Self::Head(_) => "head",
            Self::Title(_) => "title",
            Self::Link(_) => "link",
            Self::Meta(_) => "meta",
            Self::Body(_) => "body",
            Self::H1(_) => "h1",
            Self::H2(_) => "h2",
            Self::H3(_) => "h3",
            Self::H4(_) => "h4",
            Self::H5(_) => "h5",
            Self::H6(_) => "h6",
            Self::P(_) => "p",
            Self::Hr(_) => "hr",
            Self::Pre(_) => "pre",
            Self::Blockquote(_) => "blockquote",
            Self::Ol(_) => "ol",
            Self::Ul(_) => "ul",
            Self::Li(_) => "li",
            Self::Dl(_) => "dl",
            Self::Dt(_) => "dt",
            Self::Dd(_) => "dd",
            Self::Div(_) => "div",
            Self::Table(_) => "table",
            Self::Tr(_) => "tr",
            Self::Th(_) => "th",
            Self::Td(_) => "td",
            Self::Img(_) => "img",
            Self::A(_) => "a",
            Self::I(_) => "i",
            Self::B(_) => "b",
            Self::S(_) => "s",
            Self::Sub(_) => "sub",
            Self::Sup(_) => "sup",
            Self::Code(_) => "code",
            Self::Mark(_) => "mark",
            Self::Span(_) => "span",
            Self::Br(_) => "br",
            Self::Style(_) => "style",
            Self::Section(_) => "section",
            Self::U(_) => "u",
            Self::Details(_) => "details",
            Self::Summary(_) => "summary",
            Self::Thead(_) => "thead",
            Self::Tbody(_) => "tbody",

            Self::Doctype(_)
            | Self::Html(_, _)
            | Self::Comment(_)
            | Self::Text(_)
            | Self::Raw(_)
            | Self::Empty => "",
        };

        tag.to_string()
    }

    pub fn write_recursive(&self, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
        use self::Element::*;
        match self {
            Doctype(meta) => {
                writeln!(writer, "<!DOCTYPE html>")?;
                for child in &meta.children {
                    child.write_recursive(writer)?;
                }
            }

            Html(head, body) => {
                write!(writer, "<html lang=\"en\">")?;
                head.write_recursive(writer)?;
                body.write_recursive(writer)?;
                write!(writer, "</html>")?;
            }

            Head(meta) | Title(meta) | H1(meta) | H2(meta) | H3(meta) | H4(meta) | H5(meta)
            | H6(meta) | Blockquote(meta) | Body(meta) | Thead(meta) | Ol(meta) | Ul(meta)
            | Li(meta) | Dl(meta) | Dt(meta) | Dd(meta) | Div(meta) | Table(meta) | Tr(meta)
            | Th(meta) | Td(meta) | Span(meta) | Style(meta) | Section(meta) | I(meta)
            | P(meta) | Code(meta) | Pre(meta) | B(meta) | S(meta) | Sub(meta) | Sup(meta)
            | Mark(meta) | A(meta) | U(meta) | Details(meta) | Summary(meta) | Tbody(meta) => {
                write!(writer, "<{}", self.tag_name())?;
                for attr in &meta.attrs {
                    write!(writer, " {attr}")?;
                }
                write!(writer, ">")?;
                for child in &meta.children {
                    child.write_recursive(writer)?;
                }
                write!(writer, "</{}>", self.tag_name())?;
            }

            Link(meta) | Meta(meta) | Hr(meta) | Br(meta) | Img(meta) => {
                write!(writer, "<{}", self.tag_name())?;
                for attr in &meta.attrs {
                    write!(writer, " {attr}")?;
                }
                write!(writer, ">")?;
            }

            Comment(comment) => writeln!(writer, "\n<!-- {comment} -->")?,

            Text(s) | Raw(s) => write!(writer, "{s}")?,
            Empty => {}
        }

        Ok(())
    }
}

impl Element {
    pub fn to_html(&self) -> Vec<u8> {
        let mut writer = BufWriter::new(Vec::new());

        self.write_recursive(&mut writer).unwrap();

        writer.into_inner().unwrap()
    }
}
