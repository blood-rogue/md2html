use std::collections::HashMap;

use toml;

fn main() {
    let author_file = std::fs::read_to_string("authors.toml").unwrap();
    let data: HashMap<String, (String, String)> = toml::from_str(&author_file).unwrap();

    let mut out = br#"use phf::phf_map;

pub struct Author(pub &'static str, pub &'static str);

pub const AUTHORS: phf::Map<&'static str, Author> = phf_map! {
"#
    .to_vec();

    for (author, (name, avatar)) in data {
        out.extend(
            format!(
                "    \"{}\" => Author(\"{}\", \"{}\"),\n",
                author, name, avatar
            )
            .as_bytes(),
        )
    }

    out.extend(b"};\n");

    std::fs::write(
        "src/authors.rs",
        format!("{}", String::from_utf8(out).unwrap()),
    )
    .unwrap();
}
