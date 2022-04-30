// Copyright 2021 Ken Miura

use once_cell::sync::Lazy;
use regex::Regex;

pub mod email_address_validator;
pub mod password_validator;
pub mod uuid_validator;

const SYMBOL_CHAR_REGEXP: &str = r"[!-/:-@\[-`\{-~]+";
/// 記号 (ASCIIの0x21(!)から0x2f(/)、0x3a(:)から0x40(@)、0x5b([)から0x60(`)、0x7b({)から0x7e(~)) を一つ以上含むケース
pub static SYMBOL_CHAR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(SYMBOL_CHAR_REGEXP).expect("failed to compile symbol char regexp"));

const SPACE_REGEXP: &str = r"[ 　]+";
/// 半角スペース、または全角スペースを一つ以上含むケース
pub static SPACE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(SPACE_REGEXP).expect("failed to compile space regexp"));

/// 文字列が制御文字（C0制御文字、U+007F（削除文字）、C1制御文字）を含むかどうかを判定する。
/// - 制御文字を含む場合、trueを返す。そうでない場合、falseを返す。
pub fn has_control_char(s: &str) -> bool {
    let characters = s.chars().collect::<Vec<char>>();
    for c in characters {
        if c.is_control() {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use once_cell::sync::Lazy;

    use super::*;

    static CONTROL_CHAR_SET: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut set: HashSet<String> = HashSet::with_capacity(32 + 1 + 32);
        // C0制御コード
        set.insert('\u{0000}'.to_string());
        set.insert('\u{0001}'.to_string());
        set.insert('\u{0002}'.to_string());
        set.insert('\u{0003}'.to_string());
        set.insert('\u{0004}'.to_string());
        set.insert('\u{0005}'.to_string());
        set.insert('\u{0006}'.to_string());
        set.insert('\u{0007}'.to_string());
        set.insert('\u{0008}'.to_string());
        set.insert('\u{0009}'.to_string());
        set.insert('\u{000A}'.to_string());
        set.insert('\u{000B}'.to_string());
        set.insert('\u{000C}'.to_string());
        set.insert('\u{000D}'.to_string());
        set.insert('\u{000E}'.to_string());
        set.insert('\u{000F}'.to_string());
        set.insert('\u{0010}'.to_string());
        set.insert('\u{0011}'.to_string());
        set.insert('\u{0012}'.to_string());
        set.insert('\u{0013}'.to_string());
        set.insert('\u{0014}'.to_string());
        set.insert('\u{0015}'.to_string());
        set.insert('\u{0016}'.to_string());
        set.insert('\u{0017}'.to_string());
        set.insert('\u{0018}'.to_string());
        set.insert('\u{0019}'.to_string());
        set.insert('\u{001A}'.to_string());
        set.insert('\u{001B}'.to_string());
        set.insert('\u{001C}'.to_string());
        set.insert('\u{001D}'.to_string());
        set.insert('\u{001E}'.to_string());
        set.insert('\u{001F}'.to_string());
        // 削除文字
        set.insert('\u{007F}'.to_string());
        // C1制御コード
        set.insert('\u{0080}'.to_string());
        set.insert('\u{0081}'.to_string());
        set.insert('\u{0082}'.to_string());
        set.insert('\u{0083}'.to_string());
        set.insert('\u{0084}'.to_string());
        set.insert('\u{0085}'.to_string());
        set.insert('\u{0086}'.to_string());
        set.insert('\u{0087}'.to_string());
        set.insert('\u{0088}'.to_string());
        set.insert('\u{0089}'.to_string());
        set.insert('\u{008A}'.to_string());
        set.insert('\u{008B}'.to_string());
        set.insert('\u{008C}'.to_string());
        set.insert('\u{008D}'.to_string());
        set.insert('\u{008E}'.to_string());
        set.insert('\u{008F}'.to_string());
        set.insert('\u{0090}'.to_string());
        set.insert('\u{0091}'.to_string());
        set.insert('\u{0092}'.to_string());
        set.insert('\u{0093}'.to_string());
        set.insert('\u{0094}'.to_string());
        set.insert('\u{0095}'.to_string());
        set.insert('\u{0096}'.to_string());
        set.insert('\u{0097}'.to_string());
        set.insert('\u{0098}'.to_string());
        set.insert('\u{0099}'.to_string());
        set.insert('\u{009A}'.to_string());
        set.insert('\u{009B}'.to_string());
        set.insert('\u{009C}'.to_string());
        set.insert('\u{009D}'.to_string());
        set.insert('\u{009E}'.to_string());
        set.insert('\u{009F}'.to_string());
        set
    });

    #[test]
    fn has_control_char_returns_true_if_control_char_is_passed() {
        let mut result_set = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            result_set.push(has_control_char(s));
        }
        for result in result_set {
            assert!(result);
        }
    }

    #[test]
    fn has_control_char_returns_false_if_no_control_char_is_passed() {
        let s = "The quick brown fox jumps over the lazy dog. いろはにほへと　ちりぬるを　わかよたれそ　つねならむ　うゐのおくやま　けふこえて　あさきゆめみし　ゑひもせす。";
        let result = has_control_char(s);
        assert!(!result);
    }
}
