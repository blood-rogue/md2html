use std::collections::HashMap;

use fancy_regex::Regex;
use once_cell::sync::Lazy;

pub static TYPOGRAPHER_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(\((c|tm|r|p|C|TM|R|P)\))|(\+-|\.\.\.)"#).unwrap());

pub static EMOTICON_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?<=^|\s)(>:\(|>:\-\(|:"\)|:\-"\)|</3|<\\3|:/|:\-/|:'\(|:'\-\(|:,\(|:,\-\(|:\(|:\-\(|<3|\]:\(|\]:\-\(|o:\)|O:\)|o:\-\)|O:\-\)|0:\)|0:\-\)|:'\)|:'\-\)|:,\)|:,\-\)|:'D|:'\-D|:,D|:,\-D|:\*|:\-\*|x\-\)|X\-\)|:\||:\-\||:o|:\-o|:O|:\-O|:@|:\-@|:D|:\-D|:\)|:\-\)|\]:\)|\]:\-\)|:,'\(|:,'\-\(|;\(|;\-\(|:P|:\-P|8\-\)|B\-\)|,:\(|,:\-\(|,:\)|,:\-\)|:s|:\-S|:z|:\-Z|:\$|:\-\$|;\)|;\-\))(?=$|\s)"#).unwrap()
});

pub static TYPOGRAPHER: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("(c)", "©"),
        ("(C)", "©"),
        ("(tm)", "™"),
        ("(TM)", "™"),
        ("(r)", "®"),
        ("(R)", "®"),
        ("(p)", "℗"),
        ("(P)", "℗"),
        ("+-", "±"),
        ("...", "…"),
    ])
});

pub static EMOTICONS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        (">:(", "angry"),
        (">:-(", "angry"),
        (":\")", "blush"),
        (":-\")", "blush"),
        ("</3", "broken_heart"),
        ("<\\3", "broken_heart"),
        (":/", "confused"),
        (":-/", "confused"),
        (":'(", "cry"),
        (":'-(", "cry"),
        (":,(", "cry"),
        (":,-(", "cry"),
        (":(", "frowning"),
        (":-(", "frowning"),
        ("<3", "heart"),
        ("]:(", "imp"),
        ("]:-(", "imp"),
        ("o:)", "innocent"),
        ("O:)", "innocent"),
        ("o:-)", "innocent"),
        ("O:-)", "innocent"),
        ("0:)", "innocent"),
        ("0:-)", "innocent"),
        (":')", "joy"),
        (":'-)", "joy"),
        (":,)", "joy"),
        (":,-)", "joy"),
        (":'D", "joy"),
        (":'-D", "joy"),
        (":,D", "joy"),
        (":,-D", "joy"),
        (":*", "kissing"),
        (":-*", "kissing"),
        ("x-)", "laughing"),
        ("X-)", "laughing"),
        (":|", "neutral_face"),
        (":-|", "neutral_face"),
        (":o", "open_mouth"),
        (":-o", "open_mouth"),
        (":O", "open_mouth"),
        (":-O", "open_mouth"),
        (":@", "rage"),
        (":-@", "rage"),
        (":D", "smile"),
        (":-D", "smile"),
        (":)", "smiley"),
        (":-)", "smiley"),
        ("]:)", "smiling_imp"),
        ("]:-)", "smiling_imp"),
        (":,'(", "sob"),
        (":,'-(", "sob"),
        (";(", "sob"),
        (";-(", "sob"),
        (":P", "stuck_out_tongue"),
        (":-P", "stuck_out_tongue"),
        ("8-)", "sunglasses"),
        ("B-)", "sunglasses"),
        (",:(", "sweat"),
        (",:-(", "sweat"),
        (",:)", "sweat_smile"),
        (",:-)", "sweat_smile"),
        (":s", "unamused"),
        (":-S", "unamused"),
        (":z", "unamused"),
        (":-Z", "unamused"),
        (":$", "unamused"),
        (":-$", "unamused"),
        (";)", "wink"),
        (";-)", "wink"),
    ])
});

pub fn replace_typographer(text: &str) -> String {
    let mut offset = 0;
    let mut replaced_text = text.to_string();
    TYPOGRAPHER_REGEX.find_iter(text).for_each(|m| {
        if let Ok(m) = m {
            let typography = TYPOGRAPHER[m.as_str()];
            replaced_text.replace_range(m.start() - offset..m.end() - offset, typography);
            offset += (m.end() - m.start()) - typography.len()
        }
    });

    replaced_text
}

pub fn replace_emoticons(text: &str) -> String {
    let mut offset = 0;
    let mut replaced_text = text.to_string();

    EMOTICON_REGEX.find_iter(text).for_each(|m| {
        if let Ok(m) = m {
            let emoji = emojis::get_by_shortcode(EMOTICONS[m.as_str()])
                .unwrap()
                .as_str();
            replaced_text.replace_range(m.start() + offset..m.end() + offset, emoji);
            offset += emoji.len() - (m.end() - m.start())
        }
    });

    replaced_text
}
