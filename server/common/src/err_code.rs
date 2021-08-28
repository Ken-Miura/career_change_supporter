// Copyright 2021 Ken Miura

//! API呼び出し時の処理の内、common crateのコード発生したエラーに対して付与するエラーコードを列挙する。
//! common crateでのエラーコードには、10000-19999までの値を利用する。

pub(crate) const UNEXPECTED_ERR: u32 = 10000;
pub(crate) const INVALID_EMAIL_ADDRESS_FORMAT: u32 = 10001;
pub(crate) const INVALID_PASSWORD_FORMAT: u32 = 10002;
