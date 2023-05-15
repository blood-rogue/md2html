# md2html: A Markdown to HTML converter with extensive features

`md2html` is a markdown to html converter written in `Rust` using [`comark`](https://github.com/kivikakk/comrak) for parsing markdown.

## Features
- Generates Table of Contents along with level of heading (ex: `1`, `1.1`, `2.3.1`)
- Supports extended markdown syntax:
  - `subscript` using `~`
  - `highlighted/marked` using `==`
  - `inserted/underlined` using `++`
  - `emoticons` to `emoji` (ex: `:-)` to 😃)

- Syntax highlighting using [`syntect`](https://github.com/trishume/syntect) along with line numbering.

- Case-insensitive typographic replacements (ex: `(c)` to `©` and `(tm)`  to`™` ) _See `src/replacer.rs` for full list_
- Extended tasklist items
  - Supports `x` (`a green check mark`), `X` (`a red cross mark`), `+` (`a blue plus sign`) and `-` (`a grey minus sign`)
- Requires toml front-matter delimited with `+++` with `author`, `tags`, `avatar` and `title` fields. (As it is originally intended for blog post generation)
- Calculates total read time assuming average speed of `120 wpm`.
- Denotes external links with a icon after the link.
- Footnote return to reference location.
- Generates images with captions (`figcaption`) if title is present.
- Finds `author` details from a `authors.toml` file.
- Navbar with transitions

## Usage
``` console
$ md2html --help
Usage: md2html.exe [OPTIONS] --file-path <FILE_PATH>

Options:
  -f, --file-path <FILE_PATH>      The path to the markdown file
  -o, --out-dir <OUT_DIR>          The output directory in which to place files (generated html, logo and styles)
                                   [default: out]
  -d, --domain-name <DOMAIN_NAME>  The domain name of the blog to identify external websites [default: localhost]
  -O, --output-ast                 Output the HTML and Markdown struct debug info
  -v, --verbose                    Log events
  -s, --style-sheet <STYLE_SHEET>  Path to the stylesheet [default: ./styles.css]
  -l, --logo <LOGO>                Path to the logo file [default: ./logo.png]
  -a, --authors-db <AUTHORS_DB>    [default: ./authors.toml]
  -F, --force                      Force overwrite file to the output directory
  -h, --help                       Print help
  -V, --version                    Print version
```
**Note**: Requires `DOMAIN_NAME` to identify external urls

## Samples

Check the `sample/sample.md` and `sample/sample.html` files for simple example (`sample/sample.md` contains nearly everything currently supported by `md2html`)