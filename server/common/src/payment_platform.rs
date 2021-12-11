// Copyright 2021 Ken Miura

//! PAY.JPのAPI (<https://pay.jp/docs/api/>) を利用するためのモジュール群。
//! 型やnullableかどうかは下記のSDKを参照し実装<br>
//! <https://pay.jp/docs/library>

pub mod access_info;
pub mod charge;
pub mod err;
pub mod list;
pub mod tenant;
pub mod tenant_transfer;
