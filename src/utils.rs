use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{AddAssign, Mul},
    process::exit,
};

use chrono::{DateTime, Utc};
use colored::Colorize;
use fancy_regex::Regex;
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::html::{Meta, Tag};

static NON_ASCII_CHAR: Lazy<Regex> = Lazy::new(|| must(Regex::new("[^a-z0-9 _]+")));

#[derive(Deserialize, Default, Clone)]
pub struct FrontMatter {
    title: String,
    tags: Vec<String>,
    author: String,
    avatar: String,
}

#[derive(Default)]
pub struct State {
    pub table_counter: usize,
    pub front_matter: Option<FrontMatter>,
    pub footnote_counter: HashMap<String, usize>,
    pub date: DateTime<Utc>,
    pub definitions: Vec<(String, Vec<Tag>)>,
    pub styles: Vec<String>,
    pub word_count: usize,
    pub headings: Vec<(u8, String, String)>,
    pub domain: String,
}

fn remove_diacritics(string: &str) -> String {
    let chars = string.chars();
    chars.fold(String::with_capacity(string.len()), |mut acc, current| {
        match current {
            'A' | 'Ⓐ' | 'Ａ' | 'À' | 'Á' | 'Â' | 'Ầ' | 'Ấ' | 'Ẫ' | 'Ẩ' | 'Ã' | 'Ā' | 'Ă' | 'Ằ'
            | 'Ắ' | 'Ẵ' | 'Ẳ' | 'Ȧ' | 'Ǡ' | 'Ä' | 'Ǟ' | 'Ả' | 'Å' | 'Ǻ' | 'Ǎ' | 'Ȁ' | 'Ȃ' | 'Ạ'
            | 'Ậ' | 'Ặ' | 'Ḁ' | 'Ą' | 'Ⱥ' | 'Ɐ' => acc.push('A'),
            'Ꜳ' => acc.push_str("AA"),
            'Æ' | 'Ǽ' | 'Ǣ' => acc.push_str("A"),
            'Ꜵ' => acc.push_str("AO"),
            'Ꜷ' => acc.push_str("AU"),
            'Ꜹ' | 'Ꜻ' => acc.push_str("AV"),
            'Ꜽ' => acc.push_str("AY"),
            'B' | 'Ⓑ' | 'Ｂ' | 'Ḃ' | 'Ḅ' | 'Ḇ' | 'Ƀ' | 'Ƃ' | 'Ɓ' => acc.push('B'),
            'C' | 'Ⓒ' | 'Ｃ' | 'Ć' | 'Ĉ' | 'Ċ' | 'Č' | 'Ç' | 'Ḉ' | 'Ƈ' | 'Ȼ' | 'Ꜿ' => {
                acc.push('C')
            }
            'D' | 'Ⓓ' | 'Ｄ' | 'Ḋ' | 'Ď' | 'Ḍ' | 'Ḑ' | 'Ḓ' | 'Ḏ' | 'Đ' | 'Ƌ' | 'Ɗ' | 'Ɖ' | 'Ꝺ' => {
                acc.push('D')
            }
            'Ǳ' | 'Ǆ' => acc.push_str("DZ"),
            'ǲ' | 'ǅ' => acc.push_str("Dz"),
            'E' | 'Ⓔ' | 'Ｅ' | 'È' | 'É' | 'Ê' | 'Ề' | 'Ế' | 'Ễ' | 'Ể' | 'Ẽ' | 'Ē' | 'Ḕ' | 'Ḗ'
            | 'Ĕ' | 'Ė' | 'Ë' | 'Ẻ' | 'Ě' | 'Ȅ' | 'Ȇ' | 'Ẹ' | 'Ệ' | 'Ȩ' | 'Ḝ' | 'Ę' | 'Ḙ' | 'Ḛ'
            | 'Ɛ' | 'Ǝ' => acc.push('E'),
            'F' | 'Ⓕ' | 'Ｆ' | 'Ḟ' | 'Ƒ' | 'Ꝼ' => acc.push('F'),
            'G' | 'Ⓖ' | 'Ｇ' | 'Ǵ' | 'Ĝ' | 'Ḡ' | 'Ğ' | 'Ġ' | 'Ǧ' | 'Ģ' | 'Ǥ' | 'Ɠ' | 'Ꞡ' | 'Ᵹ'
            | 'Ꝿ' => acc.push('G'),
            'H' | 'Ⓗ' | 'Ｈ' | 'Ĥ' | 'Ḣ' | 'Ḧ' | 'Ȟ' | 'Ḥ' | 'Ḩ' | 'Ḫ' | 'Ħ' | 'Ⱨ' | 'Ⱶ' | 'Ɥ' => {
                acc.push('H')
            }
            'I' | 'Ⓘ' | 'Ｉ' | 'Ì' | 'Í' | 'Î' | 'Ĩ' | 'Ī' | 'Ĭ' | 'İ' | 'Ï' | 'Ḯ' | 'Ỉ' | 'Ǐ'
            | 'Ȉ' | 'Ȋ' | 'Ị' | 'Į' | 'Ḭ' | 'Ɨ' => acc.push('I'),
            'J' | 'Ⓙ' | 'Ｊ' | 'Ĵ' | 'Ɉ' => acc.push('J'),
            'K' | 'Ⓚ' | 'Ｋ' | 'Ḱ' | 'Ǩ' | 'Ḳ' | 'Ķ' | 'Ḵ' | 'Ƙ' | 'Ⱪ' | 'Ꝁ' | 'Ꝃ' | 'Ꝅ' | 'Ꞣ' => {
                acc.push('K')
            }
            'L' | 'Ⓛ' | 'Ｌ' | 'Ŀ' | 'Ĺ' | 'Ľ' | 'Ḷ' | 'Ḹ' | 'Ļ' | 'Ḽ' | 'Ḻ' | 'Ł' | 'Ƚ' | 'Ɫ'
            | 'Ⱡ' | 'Ꝉ' | 'Ꝇ' | 'Ꞁ' => acc.push('L'),
            'Ǉ' => acc.push_str("LJ"),
            'ǈ' => acc.push_str("Lj"),
            'M' | 'Ⓜ' | 'Ｍ' | 'Ḿ' | 'Ṁ' | 'Ṃ' | 'Ɱ' | 'Ɯ' => acc.push('M'),
            'N' | 'Ⓝ' | 'Ｎ' | 'Ǹ' | 'Ń' | 'Ñ' | 'Ṅ' | 'Ň' | 'Ṇ' | 'Ņ' | 'Ṋ' | 'Ṉ' | 'Ƞ' | 'Ɲ'
            | 'Ꞑ' | 'Ꞥ' => acc.push('N'),
            'Ǌ' => acc.push_str("NJ"),
            'ǋ' => acc.push_str("Nj"),
            'O' | 'Ⓞ' | 'Ｏ' | 'Ò' | 'Ó' | 'Ô' | 'Ồ' | 'Ố' | 'Ỗ' | 'Ổ' | 'Õ' | 'Ṍ' | 'Ȭ' | 'Ṏ'
            | 'Ō' | 'Ṑ' | 'Ṓ' | 'Ŏ' | 'Ȯ' | 'Ȱ' | 'Ö' | 'Ȫ' | 'Ỏ' | 'Ő' | 'Ǒ' | 'Ȍ' | 'Ȏ' | 'Ơ'
            | 'Ờ' | 'Ớ' | 'Ỡ' | 'Ở' | 'Ợ' | 'Ọ' | 'Ộ' | 'Ǫ' | 'Ǭ' | 'Ø' | 'Ǿ' | 'Ɔ' | 'Ɵ' | 'Ꝋ'
            | 'Ꝍ' => acc.push('O'),
            'Ƣ' => acc.push_str("OI"),
            'Ꝏ' => acc.push_str("OO"),
            'Ȣ' => acc.push_str("OU"),
            '\u{008C}' | 'Œ' => acc.push_str("OE"),
            '\u{009C}' | 'œ' => acc.push_str("oe"),
            'P' | 'Ⓟ' | 'Ｐ' | 'Ṕ' | 'Ṗ' | 'Ƥ' | 'Ᵽ' | 'Ꝑ' | 'Ꝓ' | 'Ꝕ' => {
                acc.push('P')
            }
            'Q' | 'Ⓠ' | 'Ｑ' | 'Ꝗ' | 'Ꝙ' | 'Ɋ' => acc.push('Q'),
            'R' | 'Ⓡ' | 'Ｒ' | 'Ŕ' | 'Ṙ' | 'Ř' | 'Ȑ' | 'Ȓ' | 'Ṛ' | 'Ṝ' | 'Ŗ' | 'Ṟ' | 'Ɍ' | 'Ɽ'
            | 'Ꝛ' | 'Ꞧ' | 'Ꞃ' => acc.push('R'),
            'S' | 'Ⓢ' | 'Ｓ' | 'ẞ' | 'Ś' | 'Ṥ' | 'Ŝ' | 'Ṡ' | 'Š' | 'Ṧ' | 'Ṣ' | 'Ṩ' | 'Ș' | 'Ş'
            | 'Ȿ' | 'Ꞩ' | 'Ꞅ' => acc.push('S'),
            'T' | 'Ⓣ' | 'Ｔ' | 'Ṫ' | 'Ť' | 'Ṭ' | 'Ț' | 'Ţ' | 'Ṱ' | 'Ṯ' | 'Ŧ' | 'Ƭ' | 'Ʈ' | 'Ⱦ'
            | 'Ꞇ' => acc.push('T'),
            'Ꜩ' => acc.push_str("TZ"),
            'U' | 'Ⓤ' | 'Ｕ' | 'Ù' | 'Ú' | 'Û' | 'Ũ' | 'Ṹ' | 'Ū' | 'Ṻ' | 'Ŭ' | 'Ü' | 'Ǜ' | 'Ǘ'
            | 'Ǖ' | 'Ǚ' | 'Ủ' | 'Ů' | 'Ű' | 'Ǔ' | 'Ȕ' | 'Ȗ' | 'Ư' | 'Ừ' | 'Ứ' | 'Ữ' | 'Ử' | 'Ự'
            | 'Ụ' | 'Ṳ' | 'Ų' | 'Ṷ' | 'Ṵ' | 'Ʉ' => acc.push('U'),
            'V' | 'Ⓥ' | 'Ｖ' | 'Ṽ' | 'Ṿ' | 'Ʋ' | 'Ꝟ' | 'Ʌ' => acc.push('V'),
            'Ꝡ' => acc.push_str("VY"),
            'W' | 'Ⓦ' | 'Ｗ' | 'Ẁ' | 'Ẃ' | 'Ŵ' | 'Ẇ' | 'Ẅ' | 'Ẉ' | 'Ⱳ' => {
                acc.push('W')
            }
            'X' | 'Ⓧ' | 'Ｘ' | 'Ẋ' | 'Ẍ' => acc.push('X'),
            'Y' | 'Ⓨ' | 'Ｙ' | 'Ỳ' | 'Ý' | 'Ŷ' | 'Ỹ' | 'Ȳ' | 'Ẏ' | 'Ÿ' | 'Ỷ' | 'Ỵ' | 'Ƴ' | 'Ɏ'
            | 'Ỿ' => acc.push('Y'),
            'Z' | 'Ⓩ' | 'Ｚ' | 'Ź' | 'Ẑ' | 'Ż' | 'Ž' | 'Ẓ' | 'Ẕ' | 'Ƶ' | 'Ȥ' | 'Ɀ' | 'Ⱬ' | 'Ꝣ' => {
                acc.push('Z')
            }
            'a' | 'ⓐ' | 'ａ' | 'ẚ' | 'à' | 'á' | 'â' | 'ầ' | 'ấ' | 'ẫ' | 'ẩ' | 'ã' | 'ā' | 'ă'
            | 'ằ' | 'ắ' | 'ẵ' | 'ẳ' | 'ȧ' | 'ǡ' | 'ä' | 'ǟ' | 'ả' | 'å' | 'ǻ' | 'ǎ' | 'ȁ' | 'ȃ'
            | 'ạ' | 'ậ' | 'ặ' | 'ḁ' | 'ą' | 'ⱥ' | 'ɐ' => acc.push('a'),
            'ꜳ' => acc.push_str("aa"),
            'æ' | 'ǽ' | 'ǣ' => acc.push('a'),
            'ꜵ' => acc.push_str("ao"),
            'ꜷ' => acc.push_str("au"),
            'ꜹ' | 'ꜻ' => acc.push_str("av"),
            'ꜽ' => acc.push_str("ay"),
            'b' | 'ⓑ' | 'ｂ' | 'ḃ' | 'ḅ' | 'ḇ' | 'ƀ' | 'ƃ' | 'ɓ' | 'þ' => {
                acc.push('b')
            }
            'c' | 'ⓒ' | 'ｃ' | 'ć' | 'ĉ' | 'ċ' | 'č' | 'ç' | 'ḉ' | 'ƈ' | 'ȼ' | 'ꜿ' | 'ↄ' => {
                acc.push('c')
            }
            'd' | 'ⓓ' | 'ｄ' | 'ḋ' | 'ď' | 'ḍ' | 'ḑ' | 'ḓ' | 'ḏ' | 'đ' | 'ƌ' | 'ɖ' | 'ɗ' | 'ꝺ' => {
                acc.push('d')
            }
            'ǳ' | 'ǆ' => acc.push_str("dz"),
            'e' | 'ⓔ' | 'ｅ' | 'è' | 'é' | 'ê' | 'ề' | 'ế' | 'ễ' | 'ể' | 'ẽ' | 'ē' | 'ḕ' | 'ḗ'
            | 'ĕ' | 'ė' | 'ë' | 'ẻ' | 'ě' | 'ȅ' | 'ȇ' | 'ẹ' | 'ệ' | 'ȩ' | 'ḝ' | 'ę' | 'ḙ' | 'ḛ'
            | 'ɇ' | 'ɛ' | 'ǝ' => acc.push('e'),
            'f' | 'ⓕ' | 'ｆ' | 'ḟ' | 'ƒ' | 'ꝼ' => acc.push('f'),
            'g' | 'ⓖ' | 'ｇ' | 'ǵ' | 'ĝ' | 'ḡ' | 'ğ' | 'ġ' | 'ǧ' | 'ģ' | 'ǥ' | 'ɠ' | 'ꞡ' | 'ᵹ'
            | 'ꝿ' => acc.push('g'),
            'h' | 'ⓗ' | 'ｈ' | 'ĥ' | 'ḣ' | 'ḧ' | 'ȟ' | 'ḥ' | 'ḩ' | 'ḫ' | 'ẖ' | 'ħ' | 'ⱨ' | 'ⱶ'
            | 'ɥ' => acc.push('h'),
            'ƕ' => acc.push_str("hv"),
            'i' | 'ⓘ' | 'ｉ' | 'ì' | 'í' | 'î' | 'ĩ' | 'ī' | 'ĭ' | 'ï' | 'ḯ' | 'ỉ' | 'ǐ' | 'ȉ'
            | 'ȋ' | 'ị' | 'į' | 'ḭ' | 'ɨ' | 'ı' => acc.push('i'),
            'j' | 'ⓙ' | 'ｊ' | 'ĵ' | 'ǰ' | 'ɉ' => acc.push('j'),
            'k' | 'ⓚ' | 'ｋ' | 'ḱ' | 'ǩ' | 'ḳ' | 'ķ' | 'ḵ' | 'ƙ' | 'ⱪ' | 'ꝁ' | 'ꝃ' | 'ꝅ' | 'ꞣ' => {
                acc.push('k')
            }
            'l' | 'ⓛ' | 'ｌ' | 'ŀ' | 'ĺ' | 'ľ' | 'ḷ' | 'ḹ' | 'ļ' | 'ḽ' | 'ḻ' | 'ſ' | 'ł' | 'ƚ'
            | 'ɫ' | 'ⱡ' | 'ꝉ' | 'ꞁ' | 'ꝇ' => acc.push('l'),
            'ǉ' => acc.push_str("lj"),
            'm' | 'ⓜ' | 'ｍ' | 'ḿ' | 'ṁ' | 'ṃ' | 'ɱ' | 'ɯ' => acc.push('m'),
            'n' | 'ⓝ' | 'ｎ' | 'ǹ' | 'ń' | 'ñ' | 'ṅ' | 'ň' | 'ṇ' | 'ņ' | 'ṋ' | 'ṉ' | 'ƞ' | 'ɲ'
            | 'ŉ' | 'ꞑ' | 'ꞥ' => acc.push('n'),
            'ǌ' => acc.push_str("nj"),
            'o' | 'ⓞ' | 'ｏ' | 'ò' | 'ó' | 'ô' | 'ồ' | 'ố' | 'ỗ' | 'ổ' | 'õ' | 'ṍ' | 'ȭ' | 'ṏ'
            | 'ō' | 'ṑ' | 'ṓ' | 'ŏ' | 'ȯ' | 'ȱ' | 'ö' | 'ȫ' | 'ỏ' | 'ő' | 'ǒ' | 'ȍ' | 'ȏ' | 'ơ'
            | 'ờ' | 'ớ' | 'ỡ' | 'ở' | 'ợ' | 'ọ' | 'ộ' | 'ǫ' | 'ǭ' | 'ø' | 'ǿ' | 'ɔ' | 'ꝋ' | 'ꝍ'
            | 'ɵ' => acc.push('o'),
            'ƣ' => acc.push_str("oi"),
            'ȣ' => acc.push_str("ou"),
            'ꝏ' => acc.push_str("oo"),
            'p' | 'ⓟ' | 'ｐ' | 'ṕ' | 'ṗ' | 'ƥ' | 'ᵽ' | 'ꝑ' | 'ꝓ' | 'ꝕ' => {
                acc.push('p')
            }
            'q' | 'ⓠ' | 'ｑ' | 'ɋ' | 'ꝗ' | 'ꝙ' => acc.push('q'),
            'r' | 'ⓡ' | 'ｒ' | 'ŕ' | 'ṙ' | 'ř' | 'ȑ' | 'ȓ' | 'ṛ' | 'ṝ' | 'ŗ' | 'ṟ' | 'ɍ' | 'ɽ'
            | 'ꝛ' | 'ꞧ' | 'ꞃ' => acc.push('r'),
            's' | 'ⓢ' | 'ｓ' | 'ß' | 'ś' | 'ṥ' | 'ŝ' | 'ṡ' | 'š' | 'ṧ' | 'ṣ' | 'ṩ' | 'ș' | 'ş'
            | 'ȿ' | 'ꞩ' | 'ꞅ' | 'ẛ' => acc.push('s'),
            't' | 'ⓣ' | 'ｔ' | 'ṫ' | 'ẗ' | 'ť' | 'ṭ' | 'ț' | 'ţ' | 'ṱ' | 'ṯ' | 'ŧ' | 'ƭ' | 'ʈ'
            | 'ⱦ' | 'ꞇ' => acc.push('t'),
            'ꜩ' => acc.push_str("tz"),
            'u' | 'ⓤ' | 'ｕ' | 'ù' | 'ú' | 'û' | 'ũ' | 'ṹ' | 'ū' | 'ṻ' | 'ŭ' | 'ü' | 'ǜ' | 'ǘ'
            | 'ǖ' | 'ǚ' | 'ủ' | 'ů' | 'ű' | 'ǔ' | 'ȕ' | 'ȗ' | 'ư' | 'ừ' | 'ứ' | 'ữ' | 'ử' | 'ự'
            | 'ụ' | 'ṳ' | 'ų' | 'ṷ' | 'ṵ' | 'ʉ' => acc.push('u'),
            'v' | 'ⓥ' | 'ｖ' | 'ṽ' | 'ṿ' | 'ʋ' | 'ꝟ' | 'ʌ' => acc.push('v'),
            'ꝡ' => acc.push_str("vy"),
            'w' | 'ⓦ' | 'ｗ' | 'ẁ' | 'ẃ' | 'ŵ' | 'ẇ' | 'ẅ' | 'ẘ' | 'ẉ' | 'ⱳ' => {
                acc.push('w')
            }
            'x' | 'ⓧ' | 'ｘ' | 'ẋ' | 'ẍ' => acc.push('x'),
            'y' | 'ⓨ' | 'ｙ' | 'ỳ' | 'ý' | 'ŷ' | 'ỹ' | 'ȳ' | 'ẏ' | 'ÿ' | 'ỷ' | 'ẙ' | 'ỵ' | 'ƴ'
            | 'ɏ' | 'ỿ' => acc.push('y'),
            'z' | 'ⓩ' | 'ｚ' | 'ź' | 'ẑ' | 'ż' | 'ž' | 'ẓ' | 'ẕ' | 'ƶ' | 'ȥ' | 'ɀ' | 'ⱬ' | 'ꝣ' => {
                acc.push('z')
            }
            '·' | '/' | '_' | ',' | ':' | ';' | ' ' | '\n' | '\t' | '\r' => acc.push('-'),
            _ => acc.push(current),
        }
        acc
    })
}

pub fn heading_to_slug(elements: &[Tag]) -> (String, String) {
    use Tag::*;
    let mut text = String::new();

    fn iterate(s: &mut String, child: &Tag) {
        match child {
            Text(text) => s.add_assign(&text),
            H1(meta) | H2(meta) | H3(meta) | H4(meta) | H5(meta) | H6(meta) | Blockquote(meta)
            | Body(meta) | Ol(meta) | Ul(meta) | Li(meta) | Dl(meta) | Dt(meta) | Dd(meta)
            | Div(meta) | Table(meta) | Tr(meta) | Th(meta) | Td(meta) | Span(meta)
            | Section(meta) | I(meta) | P(meta) | Code(meta) | Pre(meta) | B(meta) | S(meta)
            | Sub(meta) | Sup(meta) | Mark(meta) | A(meta) | U(meta) | Link(meta) | Meta(meta)
            | Hr(meta) | Br(meta) | Img(meta) => {
                for child in meta.get_children() {
                    iterate(s, &child)
                }
            }

            _ => {}
        }
    }

    for element in elements {
        iterate(&mut text, element);
    }

    let title = text.clone();

    let text = NON_ASCII_CHAR
        .replace_all(&remove_diacritics(&text).to_lowercase(), "-")
        .trim_matches('-')
        .to_string();

    (text, title)
}

pub fn init(section: Tag, state: State) -> Tag {
    let mut footnotes = Vec::with_capacity(state.definitions.len());

    let front_matter = must(state.front_matter.ok_or_else(|| "Missing front-matter"));

    for (definition, meta) in state.definitions {
        let mut references = meta;
        for i in 0..state.footnote_counter[&definition] {
            references.push(Tag::A(
                Meta::new()
                    .with_child(Tag::Text("↩".into()))
                    .with_attr(&format!(
                        "href=\"#footnote-reference-{definition}{}\"",
                        if i > 0 { format!(":{i}") } else { "".into() }
                    )),
            ))
        }
        footnotes.insert(
            must(definition.parse::<usize>()) - 1,
            Tag::Li(
                Meta::new().with_child(Tag::Div(
                    Meta::new()
                        .with_children(references)
                        .with_attr(&format!("id=\"footnote-definition-{definition}\"")),
                )),
            ),
        )
    }

    let mut tags = Vec::new();
    for tag in front_matter.tags {
        tags.push(Tag::A(
            Meta::new()
                .with_child(Tag::Text(format!("#{tag}")))
                .with_attrs(vec![
                    format!("href=\"{}/tags/{tag}\"", state.domain),
                    "class=\"tag\"".to_string(),
                ]),
        ))
    }

    let mut heading_levels = [0; 6];
    let mut format_heading = |depth: u8| {
        heading_levels[(depth - 1) as usize] += 1;

        for i in depth..5 {
            heading_levels[i as usize] = 0;
        }

        match depth {
            1 => format!("{}", heading_levels[0]),
            2 => format!("{}.{}", heading_levels[0], heading_levels[1]),
            3 => format!(
                "{}.{}.{}",
                heading_levels[0], heading_levels[1], heading_levels[2]
            ),
            4 => format!(
                "{}.{}.{}.{}",
                heading_levels[0], heading_levels[1], heading_levels[2], heading_levels[3]
            ),
            5 => format!(
                "{}.{}.{}.{}.{}",
                heading_levels[0],
                heading_levels[1],
                heading_levels[2],
                heading_levels[3],
                heading_levels[4]
            ),
            6 => format!(
                "{}.{}.{}.{}.{}.{}",
                heading_levels[0],
                heading_levels[1],
                heading_levels[2],
                heading_levels[3],
                heading_levels[4],
                heading_levels[5]
            ),
            _ => unreachable!(),
        }
    };

    let toc = Tag::Div(
        Meta::new().with_children(
            state
                .headings
                .iter()
                .map(|(depth, id, title)| {
                    Tag::P(
                        Meta::new()
                            .with_child(Tag::A(
                                Meta::new()
                                    .with_child(Tag::Text(format!(
                                        "{}. {title}",
                                        format_heading(*depth)
                                    )))
                                    .with_attr(&format!("href=\"#heading__{id}\"")),
                            ))
                            .with_attr(&format!("style=\"padding-left: {}px\"", depth.mul(20))),
                    )
                })
                .collect(),
        ),
    );

    let families = ["Roboto", "Jetbrains Mono", "Open Sans"]
        .iter()
        .map(|font| font.replace(" ", "+"))
        .collect::<Vec<_>>()
        .join("&family=");

    let minified = must(css_minify::optimizations::Minifier::default().minify(
        &state.styles.join(""),
        css_minify::optimizations::Level::Three,
    ))
    .replace(":-webkit-scrollbar", "::-webkit-scrollbar"); // This breaks custom scroll bar styling as it replaces `::` with `:` which breaks `-webkit-scrollbar`

    let head = Tag::Head(Meta::new().with_children(Vec::from([
        Tag::Meta(Meta::new().with_attr("charset=\"utf-8\"")),
        Tag::Meta(Meta::new().with_attrs(vec![
            "name=\"viewport\"".to_string(),
            "content=\"width=device-width, initial-scale=1\"".to_string(),
        ])),
        Tag::Meta(Meta::new().with_attrs(vec![
            "property=\"og:title\"".to_string(),
            format!("content=\"{}\"", front_matter.title),
        ])),
        Tag::Link(Meta::new().with_attrs(vec![
            "rel=\"stylesheet\"".to_string(),
            format!(
                "href=\"https://fonts.googleapis.com/css2?family={}\"",
                families
            ),
        ])),
        Tag::Link(Meta::new().with_attrs(vec![
            "rel=\"stylesheet\"".to_string(),
            "href=\"https://unpkg.com/@fortawesome/fontawesome-free/css/all.min.css\"".to_string(),
        ])),
        Tag::Title(Meta::new().with_child(Tag::Text(front_matter.title.clone()))),
        Tag::Style(minified),
    ])));

    let body = Tag::Body(
        Meta::new().with_children(Vec::from([
            Tag::Comment("META_CONTAINER_START".to_string()),
            Tag::H1(Meta::new().with_child(Tag::Text(front_matter.title.clone()))),
            Tag::Div(Meta::new().with_children(tags)),
            Tag::Div(
                Meta::new()
                    .with_children(vec![
                        Tag::Img(Meta::new().with_attrs(vec![
                            format!("src=\"{}\"", front_matter.avatar),
                            format!("alt=\"{}\"", front_matter.author),
                        ])),
                        Tag::Span(
                            Meta::new().with_child(Tag::A(
                                Meta::new()
                                    .with_child(Tag::Text(front_matter.author.clone()))
                                    .with_attr(&format!(
                                        "href=\"{}/authors/{}\"",
                                        state.domain, front_matter.author
                                    )),
                            )),
                        ),
                        Tag::Span(Meta::new().with_child(Tag::Text(format!(
                            "{} min read &bull; {}",
                            state.word_count / 120,
                            state.date.format("%e %B, %Y")
                        )))),
                    ])
                    .with_attr("class=\"meta-container\""),
            ),
            Tag::Comment("META_CONTAINER_END".to_string()),
            Tag::Comment("TOC_START".to_string()),
            Tag::Details(Meta::new().with_children(vec![
                Tag::Summary(Meta::new().with_child(Tag::Span(
                    Meta::new().with_child(Tag::Text("Table of Contents".into())),
                ))),
                toc,
            ])),
            Tag::Comment("TOC_END".to_string()),
            Tag::Comment("BLOG_SECTION_START".to_string()),
            section,
            Tag::Comment("BLOG_SECTION_END".to_string()),
            Tag::Comment("FOOTNOTES_START".to_string()),
            Tag::Section(Meta::new().with_children(vec![
                Tag::Hr(Meta::default()),
                Tag::Ol(Meta::new().with_children(footnotes)),
            ])),
            Tag::Comment("FOOTNOTES_END".to_string()),
        ])),
    );

    Tag::Doctype(Meta::new().with_children(Vec::from([
        Tag::Comment(format!(
            "Generated using `md2html` by `Blood Rogue (github.com/blood-rogue)` on {}.",
            state.date.format("%d/%m/%Y %H:%M:%S")
        )),
        Tag::Html(Box::new(head), Box::new(body)),
    ])))
}

pub fn char_to_taskitem(ch: char) -> Tag {
    let (icon, color) = match ch {
        'x' => ("square-check", "limegreen"),
        '-' => ("square-minus", "grey"),
        '+' => ("square-plus", "deepskyblue"),
        'X' => ("square-xmark", "red"),
        _ => ("", "white"),
    };

    Tag::Span(Meta::new().with_attrs(vec![
        format!("class=\"fa-solid fa-{icon}\""),
        format!("style=\"color: {color}\""),
    ]))
}

pub fn must<T, E: Debug>(res: Result<T, E>) -> T {
    match res {
        Ok(t) => t,
        Err(e) => {
            println!("{}", format!("[ERROR]: {:#?}", e).bright_red());
            exit(0);
        }
    }
}
