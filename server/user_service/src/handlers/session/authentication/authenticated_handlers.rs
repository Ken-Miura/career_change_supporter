// Copyright 2023 Ken Miura

pub(crate) mod agreement;
pub(crate) mod authenticated_users;
pub(crate) mod consultation;
pub(crate) mod delete_accounts;
pub(crate) mod mfs_setting;
pub(crate) mod personal_info;
mod platform_fee_rate;
pub(crate) mod refresh;
mod rewards_info;

const MIN_FEE_PER_HOUR_IN_YEN: i32 = 3000;
const MAX_FEE_PER_HOUR_IN_YEN: i32 = 10000;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use once_cell::sync::Lazy;

    pub(super) static SYMBOL_SET: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut set: HashSet<String> = HashSet::with_capacity(32);
        set.insert("!".to_string());
        set.insert("\"".to_string());
        set.insert("#".to_string());
        set.insert("$".to_string());
        set.insert("%".to_string());
        set.insert("&".to_string());
        set.insert("'".to_string());
        set.insert("(".to_string());
        set.insert(")".to_string());
        set.insert("*".to_string());
        set.insert("+".to_string());
        set.insert(",".to_string());
        set.insert("-".to_string());
        set.insert(".".to_string());
        set.insert("/".to_string());
        set.insert(":".to_string());
        set.insert(";".to_string());
        set.insert("<".to_string());
        set.insert("=".to_string());
        set.insert(">".to_string());
        set.insert("?".to_string());
        set.insert("@".to_string());
        set.insert("[".to_string());
        set.insert("\\".to_string());
        set.insert("]".to_string());
        set.insert("^".to_string());
        set.insert("_".to_string());
        set.insert("`".to_string());
        set.insert("{".to_string());
        set.insert("|".to_string());
        set.insert("}".to_string());
        set.insert("~".to_string());
        set
    });

    pub(super) static CONTROL_CHAR_SET: Lazy<HashSet<String>> = Lazy::new(|| {
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

    pub(super) static NON_NEW_LINE_CONTROL_CHAR_SET: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut set: HashSet<String> = HashSet::with_capacity(32 - 2 + 1 + 32);
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
        // set.insert('\u{000A}'.to_string());
        set.insert('\u{000B}'.to_string());
        set.insert('\u{000C}'.to_string());
        // set.insert('\u{000D}'.to_string());
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

    pub(super) static NEW_LINE_CONTROL_CHAR_SET: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut set: HashSet<String> = HashSet::with_capacity(2);
        set.insert('\u{000A}'.to_string());
        set.insert('\u{000D}'.to_string());
        set
    });

    pub(super) static SPACE_SET: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut set: HashSet<String> = HashSet::with_capacity(2);
        // 半角スペース
        set.insert(" ".to_string());
        // 全角スペース
        set.insert("　".to_string());
        set
    });

    pub(super) static NUMBER_SET: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut set: HashSet<String> = HashSet::with_capacity(10);
        set.insert("0".to_string());
        set.insert("1".to_string());
        set.insert("2".to_string());
        set.insert("3".to_string());
        set.insert("4".to_string());
        set.insert("5".to_string());
        set.insert("6".to_string());
        set.insert("7".to_string());
        set.insert("8".to_string());
        set.insert("9".to_string());
        set
    });
}
