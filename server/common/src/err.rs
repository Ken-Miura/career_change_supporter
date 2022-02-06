// Copyright 2021 Ken Miura

//! エラーに関連する構造体、関数を集約するモジュール

/// API呼び出し時の処理の内、common crateのコード発生したエラーに対して付与するエラーコードの列挙<br>
/// common crateでのエラーコードには、10000-19999までの値を利用する。
pub(crate) enum Code {
    UnexpectedErr = 10000,
    InvalidEmailAddressFormat = 10001,
    InvalidPasswordFormat = 10002,
}
