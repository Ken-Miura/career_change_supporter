// Copyright 2021 Ken Miura

//! API呼び出し時の処理の内、user_service crateのコード発生したエラーに対して付与するエラーコードを列挙する。
//! user_service crateでのエラーコードには、20000-29999までの値を利用する。

pub(crate) const UNEXPECTED_ERR: u32 = 20000;
