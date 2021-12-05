// Copyright 2021 Ken Miura

use serde::{Deserialize, Serialize};

/// PAY.JP APIにおけるlistオブジェクトを示す <https://pay.jp/docs/api/#list%E3%82%AA%E3%83%96%E3%82%B8%E3%82%A7%E3%82%AF%E3%83%88>
#[derive(Serialize, Deserialize, Debug)]
// NOTE: trait boundをstructに設定すべきかどうかは、次のURLで議論されている <https://github.com/rust-lang/rust-clippy/issues/1689>
// そのため、Serialize, Deserialize, DebugをstructのTの境界に導入すべきかよく確認、検討する必要あり
pub struct List<T> {
    pub object: String,
    pub has_more: bool,
    pub url: String,
    pub data: Vec<T>,
    pub count: u32,
}
