// Copyright 2022 Ken Miura
//! 口座情報のバリデーションを行う。バリデーションはPAY.JPを実装した際に理解した仕様を参考としている。

use std::{collections::HashSet, error::Error, fmt::Display};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::handlers::session::authentication::authenticated_handlers::personal_info::{
    FIRST_NAME_MAX_LENGTH, FIRST_NAME_MIN_LENGTH, LAST_NAME_MAX_LENGTH, LAST_NAME_MIN_LENGTH,
};

use super::BankAccount;

const BANK_CODE_REGEXP: &str = r"^[0-9]{4}$";
/// 数字4桁
static BANK_CODE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(BANK_CODE_REGEXP).expect("failed to compile num char regexp"));

const BRANCH_CODE_REGEXP: &str = r"^[0-9]{3}$";
/// 数字3桁
static BRANCH_CODE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(BRANCH_CODE_REGEXP).expect("failed to compile num char regexp"));

/// サポートする預金種別。"普通"のみのサポートとする。
static ACCOUNT_TYPE_SET: Lazy<HashSet<String>> = Lazy::new(|| {
    let mut set: HashSet<String> = HashSet::with_capacity(1);
    set.insert("普通".to_string());
    set
});

const ACCOUNT_NUMBER_REGEXP: &str = r"^[0-9]{7,8}$";
/// 数字7桁、または8桁
///
/// 一般的にゆうちょ銀行が8桁、それ以外の金融機関は7桁と成るため、それらに沿うか確認する。
static ACCOUNT_NUMBER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(ACCOUNT_NUMBER_REGEXP).expect("failed to compile num char regexp"));

/// 口座名義の長さの最小値
static ACCOUNT_HOLDER_NAME_MIN_LENGTH: Lazy<usize> = Lazy::new(|| {
    // "セイ＋空白＋メイ"分の長さ
    LAST_NAME_MIN_LENGTH + 1 + FIRST_NAME_MIN_LENGTH
});

/// 口座名義の長さの最大値
///
/// pay.jpとしては255文字まで受け付ける。しかし、このシステムの入力制限に合わせて数字を調整する。
static ACCOUNT_HOLDER_NAME_MAX_LENGTH: Lazy<usize> = Lazy::new(|| {
    // "セイ＋空白＋メイ"分の長さ
    let length = LAST_NAME_MAX_LENGTH + 1 + FIRST_NAME_MAX_LENGTH;
    if length > 255 {
        panic!("exceed pay.jp limit")
    }
    length
});

const ZENKAKU_KATAKANA_ZENKAKU_SPACE_REGEXP: &str = r"^[ァ-ヴー　]+$";
/// 全角カタカナと全角スペースのみのケース<br>
/// 参考: https://qiita.com/nasuB7373/items/17adc4b808a8bd39624d<br>
/// \p{katakana}は、半角カタカナも含むので使わない<br>
///  pay.jpの問い合わせ回答によると全角が推奨されるため、全角のみの利用を想定する。
static ZENKAKU_KATAKANA_ZENKAKU_SPACE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(ZENKAKU_KATAKANA_ZENKAKU_SPACE_REGEXP)
        .expect("failed to compile zenkaku katakana regexp")
});

pub(super) fn validate_bank_account(
    bank_account: &BankAccount,
) -> Result<(), BankAccountValidationError> {
    validate_bank_code(bank_account.bank_code.as_str())?;
    validate_branch_code(bank_account.branch_code.as_str())?;
    validate_account_type(bank_account.account_type.as_str())?;
    validate_account_number(bank_account.account_number.as_str())?;
    validate_account_holder_name(bank_account.account_holder_name.as_str())?;
    Ok(())
}

fn validate_bank_code(bank_code: &str) -> Result<(), BankAccountValidationError> {
    if !BANK_CODE_RE.is_match(bank_code) {
        return Err(BankAccountValidationError::InvalidBankCodeFormat(
            bank_code.to_string(),
        ));
    }
    Ok(())
}

fn validate_branch_code(branch_code: &str) -> Result<(), BankAccountValidationError> {
    if !BRANCH_CODE_RE.is_match(branch_code) {
        return Err(BankAccountValidationError::InvalidBranchCodeFormat(
            branch_code.to_string(),
        ));
    }
    Ok(())
}

fn validate_account_type(account_type: &str) -> Result<(), BankAccountValidationError> {
    if !ACCOUNT_TYPE_SET.contains(account_type) {
        return Err(BankAccountValidationError::InvalidAccountType(
            account_type.to_string(),
        ));
    }
    Ok(())
}

fn validate_account_number(account_number: &str) -> Result<(), BankAccountValidationError> {
    if !ACCOUNT_NUMBER_RE.is_match(account_number) {
        return Err(BankAccountValidationError::InvalidAccountNumberFormat(
            account_number.to_string(),
        ));
    }
    Ok(())
}

fn validate_account_holder_name(
    account_holder_name: &str,
) -> Result<(), BankAccountValidationError> {
    let account_holder_name_length = account_holder_name.chars().count();
    let min_length = *ACCOUNT_HOLDER_NAME_MIN_LENGTH;
    let max_length = *ACCOUNT_HOLDER_NAME_MAX_LENGTH;
    if !(min_length..=max_length).contains(&account_holder_name_length) {
        return Err(BankAccountValidationError::InvalidAccountHolderNameLength {
            length: account_holder_name_length,
            min_length,
            max_length,
        });
    }
    if !ZENKAKU_KATAKANA_ZENKAKU_SPACE_RE.is_match(account_holder_name) {
        return Err(BankAccountValidationError::IllegalCharInAccountHolderName(
            account_holder_name.to_string(),
        ));
    }
    Ok(())
}

/// Error related to [validate_bank_account()]
#[derive(Debug, PartialEq)]
pub(super) enum BankAccountValidationError {
    InvalidBankCodeFormat(String),
    InvalidBranchCodeFormat(String),
    InvalidAccountType(String),
    InvalidAccountNumberFormat(String),
    InvalidAccountHolderNameLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInAccountHolderName(String),
}

impl Display for BankAccountValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BankAccountValidationError::InvalidBankCodeFormat(bank_code) => write!(
                f,
                "bank_code: invalid format: {} (binary: {:X?})",
                bank_code,
                bank_code.as_bytes().to_vec()
            ),
            BankAccountValidationError::InvalidBranchCodeFormat(branch_code) => write!(
                f,
                "branch_code: invalid format: {} (binary: {:X?})",
                branch_code,
                branch_code.as_bytes().to_vec()
            ),
            BankAccountValidationError::InvalidAccountType(account_type) => write!(
                f,
                "account_type: invalid format: {} (binary: {:X?})",
                account_type,
                account_type.as_bytes().to_vec()
            ),
            BankAccountValidationError::InvalidAccountNumberFormat(account_number) => write!(
                f,
                "account_number: invalid format: {} (binary: {:X?})",
                account_number,
                account_number.as_bytes().to_vec()
            ),
            BankAccountValidationError::InvalidAccountHolderNameLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid account_holder_name length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            BankAccountValidationError::IllegalCharInAccountHolderName(account_holder_name) => {
                write!(
                    f,
                    "account_holder_name: invalid charcter included: {} (binary: {:X?})",
                    account_holder_name,
                    account_holder_name.as_bytes().to_vec()
                )
            }
        }
    }
}

impl Error for BankAccountValidationError {}

#[cfg(test)]
mod tests {

    use crate::handlers::session::authentication::authenticated_handlers::tests::{
        CONTROL_CHAR_SET, NUMBER_SET, SPACE_SET, SYMBOL_SET,
    };

    use super::*;

    #[test]
    fn validate_bank_account_success1() {
        let bank_account = BankAccount {
            bank_code: "0123".to_string(),
            branch_code: "456".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "タナカ　タロウ".to_string(),
        };
        validate_bank_account(&bank_account).expect("failed to get Ok");
    }

    #[test]
    fn validate_bank_account_success2() {
        let bank_account = BankAccount {
            bank_code: "0123".to_string(),
            branch_code: "456".to_string(),
            account_type: "普通".to_string(),
            account_number: "12345678".to_string(),
            account_holder_name: "タナカ　タロウ".to_string(),
        };
        validate_bank_account(&bank_account).expect("failed to get Ok");
    }

    #[test]
    fn validate_bank_account_fail_3_digit_bank_code() {
        let bank_account = BankAccount {
            bank_code: "012".to_string(),
            branch_code: "456".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "タナカ　タロウ".to_string(),
        };

        let err = validate_bank_account(&bank_account).expect_err("failed to get Err");

        assert_eq!(
            BankAccountValidationError::InvalidBankCodeFormat(bank_account.bank_code),
            err
        )
    }

    #[test]
    fn validate_bank_account_fail_5_digit_bank_code() {
        let bank_account = BankAccount {
            bank_code: "01234".to_string(),
            branch_code: "456".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "タナカ　タロウ".to_string(),
        };

        let err = validate_bank_account(&bank_account).expect_err("failed to get Err");

        assert_eq!(
            BankAccountValidationError::InvalidBankCodeFormat(bank_account.bank_code),
            err
        )
    }

    #[test]
    fn validate_bank_account_returns_err_if_bank_code_is_control_char() {
        let mut bank_account_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let bank_account = BankAccount {
                bank_code: s.to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBankCodeFormat(
                    bank_account.bank_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_bank_code_includes_control_char() {
        let mut bank_account_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "01".to_string() + s + "3",
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBankCodeFormat(
                    bank_account.bank_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_bank_code_starts_with_control_char() {
        let mut bank_account_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let bank_account = BankAccount {
                bank_code: s.to_string() + "123",
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBankCodeFormat(
                    bank_account.bank_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_bank_code_ends_with_control_char() {
        let mut bank_account_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "012".to_string() + s,
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBankCodeFormat(
                    bank_account.bank_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_bank_code_is_symbol() {
        let mut bank_account_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let bank_account = BankAccount {
                bank_code: s.to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBankCodeFormat(
                    bank_account.bank_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_bank_code_includes_symbol() {
        let mut bank_account_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "01".to_string() + s + "3",
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBankCodeFormat(
                    bank_account.bank_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_bank_code_starts_with_symbol() {
        let mut bank_account_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let bank_account = BankAccount {
                bank_code: s.to_string() + "123",
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBankCodeFormat(
                    bank_account.bank_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_bank_code_ends_with_symbol() {
        let mut bank_account_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "012".to_string() + s,
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBankCodeFormat(
                    bank_account.bank_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_bank_code_is_space() {
        let mut bank_account_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let bank_account = BankAccount {
                bank_code: s.to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBankCodeFormat(
                    bank_account.bank_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_bank_code_includes_space() {
        let mut bank_account_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "01".to_string() + s + "3",
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBankCodeFormat(
                    bank_account.bank_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_bank_code_starts_with_space() {
        let mut bank_account_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let bank_account = BankAccount {
                bank_code: s.to_string() + "123",
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBankCodeFormat(
                    bank_account.bank_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_bank_code_ends_with_space() {
        let mut bank_account_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "012".to_string() + s,
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBankCodeFormat(
                    bank_account.bank_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_fail_2_digit_branch_code() {
        let bank_account = BankAccount {
            bank_code: "0123".to_string(),
            branch_code: "45".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "タナカ　タロウ".to_string(),
        };

        let err = validate_bank_account(&bank_account).expect_err("failed to get Err");

        assert_eq!(
            BankAccountValidationError::InvalidBranchCodeFormat(bank_account.branch_code),
            err
        )
    }

    #[test]
    fn validate_bank_account_fail_4_digit_branch_code() {
        let bank_account = BankAccount {
            bank_code: "0123".to_string(),
            branch_code: "4567".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "タナカ　タロウ".to_string(),
        };

        let err = validate_bank_account(&bank_account).expect_err("failed to get Err");

        assert_eq!(
            BankAccountValidationError::InvalidBranchCodeFormat(bank_account.branch_code),
            err
        )
    }

    #[test]
    fn validate_bank_account_returns_err_if_branch_code_is_control_char() {
        let mut bank_account_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: s.to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBranchCodeFormat(
                    bank_account.branch_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_branch_code_includes_control_char() {
        let mut bank_account_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "4".to_string() + s + "6",
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBranchCodeFormat(
                    bank_account.branch_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_branch_code_starts_with_control_char() {
        let mut bank_account_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: s.to_string() + "45",
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBranchCodeFormat(
                    bank_account.branch_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_branch_code_ends_with_control_char() {
        let mut bank_account_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "45".to_string() + s,
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBranchCodeFormat(
                    bank_account.branch_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_branch_code_is_symbol() {
        let mut bank_account_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: s.to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBranchCodeFormat(
                    bank_account.branch_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_branch_code_includes_symbol() {
        let mut bank_account_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "4".to_string() + s + "6",
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBranchCodeFormat(
                    bank_account.branch_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_branch_code_starts_with_symbol() {
        let mut bank_account_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: s.to_string() + "56",
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBranchCodeFormat(
                    bank_account.branch_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_branch_code_ends_with_symbol() {
        let mut bank_account_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "45".to_string() + s,
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBranchCodeFormat(
                    bank_account.branch_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_branch_code_is_space() {
        let mut bank_account_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: s.to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBranchCodeFormat(
                    bank_account.branch_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_branch_code_includes_space() {
        let mut bank_account_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "4".to_string() + s + "6",
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBranchCodeFormat(
                    bank_account.branch_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_branch_code_starts_with_space() {
        let mut bank_account_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: s.to_string() + "56",
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBranchCodeFormat(
                    bank_account.branch_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_branch_code_ends_with_space() {
        let mut bank_account_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "45".to_string() + s,
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidBranchCodeFormat(
                    bank_account.branch_code.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_fail_unsupported_account_type() {
        let bank_account = BankAccount {
            bank_code: "0123".to_string(),
            branch_code: "456".to_string(),
            account_type: "当座".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "タナカ　タロウ".to_string(),
        };

        let err = validate_bank_account(&bank_account).expect_err("failed to get Err");

        assert_eq!(
            BankAccountValidationError::InvalidAccountType(bank_account.account_type),
            err
        )
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_type_is_number() {
        let mut bank_account_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: s.to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountType(
                    bank_account.account_type.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_type_includes_number() {
        let mut bank_account_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普".to_string() + s + "通",
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountType(
                    bank_account.account_type.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_type_starts_with_number() {
        let mut bank_account_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: s.to_string() + "普通",
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountType(
                    bank_account.account_type.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_type_ends_with_number() {
        let mut bank_account_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string() + s,
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountType(
                    bank_account.account_type.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_type_is_control_char() {
        let mut bank_account_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: s.to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountType(
                    bank_account.account_type.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_type_includes_control_char() {
        let mut bank_account_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普".to_string() + s + "通",
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountType(
                    bank_account.account_type.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_type_starts_with_control_char() {
        let mut bank_account_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: s.to_string() + "普通",
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountType(
                    bank_account.account_type.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_type_ends_with_control_char() {
        let mut bank_account_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string() + s,
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountType(
                    bank_account.account_type.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_type_is_symbol() {
        let mut bank_account_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "4567".to_string(),
                branch_code: "789".to_string(),
                account_type: s.to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountType(
                    bank_account.account_type.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_type_includes_symbol() {
        let mut bank_account_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "8901".to_string(),
                branch_code: "012".to_string(),
                account_type: "普".to_string() + s + "通",
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountType(
                    bank_account.account_type.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_type_starts_with_symbol() {
        let mut bank_account_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "345".to_string(),
                account_type: s.to_string() + "普通",
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountType(
                    bank_account.account_type.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_type_ends_with_symbol() {
        let mut bank_account_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "345".to_string(),
                account_type: "普通".to_string() + s,
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountType(
                    bank_account.account_type.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_type_is_space() {
        let mut bank_account_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "123".to_string(),
                account_type: s.to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountType(
                    bank_account.account_type.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_type_includes_space() {
        let mut bank_account_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普".to_string() + s + "通",
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountType(
                    bank_account.account_type.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_type_starts_with_space() {
        let mut bank_account_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "555".to_string(),
                account_type: s.to_string() + "普通",
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountType(
                    bank_account.account_type.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_type_ends_with_space() {
        let mut bank_account_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "455".to_string(),
                account_type: "普通".to_string() + s,
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountType(
                    bank_account.account_type.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_fail_6_digit_account_number() {
        let bank_account = BankAccount {
            bank_code: "0123".to_string(),
            branch_code: "456".to_string(),
            account_type: "普通".to_string(),
            account_number: "123456".to_string(),
            account_holder_name: "タナカ　タロウ".to_string(),
        };

        let err = validate_bank_account(&bank_account).expect_err("failed to get Err");

        assert_eq!(
            BankAccountValidationError::InvalidAccountNumberFormat(bank_account.account_number),
            err
        )
    }

    #[test]
    fn validate_bank_account_fail_9_digit_account_number() {
        let bank_account = BankAccount {
            bank_code: "0123".to_string(),
            branch_code: "456".to_string(),
            account_type: "普通".to_string(),
            account_number: "123456789".to_string(),
            account_holder_name: "タナカ　タロウ".to_string(),
        };

        let err = validate_bank_account(&bank_account).expect_err("failed to get Err");

        assert_eq!(
            BankAccountValidationError::InvalidAccountNumberFormat(bank_account.account_number),
            err
        )
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_number_is_control_char() {
        let mut bank_account_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: s.to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountNumberFormat(
                    bank_account.account_number.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_number_includes_control_char() {
        let mut bank_account_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234".to_string() + s + "567",
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountNumberFormat(
                    bank_account.account_number.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_number_starts_with_control_char() {
        let mut bank_account_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: s.to_string() + "234567",
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountNumberFormat(
                    bank_account.account_number.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_number_ends_with_control_char() {
        let mut bank_account_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "123456".to_string() + s,
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountNumberFormat(
                    bank_account.account_number.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_number_is_symbol() {
        let mut bank_account_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: s.to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountNumberFormat(
                    bank_account.account_number.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_number_includes_symbol() {
        let mut bank_account_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "123".to_string() + s + "567",
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountNumberFormat(
                    bank_account.account_number.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_number_starts_with_symbol() {
        let mut bank_account_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: s.to_string() + "234567",
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountNumberFormat(
                    bank_account.account_number.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_number_ends_with_symbol() {
        let mut bank_account_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "123456".to_string() + s,
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountNumberFormat(
                    bank_account.account_number.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_number_is_space() {
        let mut bank_account_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: s.to_string(),
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountNumberFormat(
                    bank_account.account_number.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_number_includes_space() {
        let mut bank_account_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234".to_string() + s + "67",
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountNumberFormat(
                    bank_account.account_number.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_number_starts_with_space() {
        let mut bank_account_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: s.to_string() + "2345678",
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountNumberFormat(
                    bank_account.account_number.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_number_ends_with_space() {
        let mut bank_account_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "123456".to_string() + s,
                account_holder_name: "タナカ　タロウ".to_string(),
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::InvalidAccountNumberFormat(
                    bank_account.account_number.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_fail_account_holder_name_invalid_account_holder_name_length1() {
        let bank_account = BankAccount {
            bank_code: "0123".to_string(),
            branch_code: "456".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "アア".to_string(),
        };

        let err = validate_bank_account(&bank_account).expect_err("failed to get Err");

        assert_eq!(
            BankAccountValidationError::InvalidAccountHolderNameLength {
                length: bank_account.account_holder_name.chars().count(),
                min_length: *ACCOUNT_HOLDER_NAME_MIN_LENGTH,
                max_length: *ACCOUNT_HOLDER_NAME_MAX_LENGTH
            },
            err
        )
    }

    #[test]
    fn validate_bank_account_fail_account_holder_name_invalid_account_holder_name_length2() {
        let bank_account = BankAccount {
            bank_code: "0123".to_string(),
            branch_code: "456".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "アアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアア".to_string(),
        };

        let err = validate_bank_account(&bank_account).expect_err("failed to get Err");

        assert_eq!(
            BankAccountValidationError::InvalidAccountHolderNameLength {
                length: bank_account.account_holder_name.chars().count(),
                min_length: *ACCOUNT_HOLDER_NAME_MIN_LENGTH,
                max_length: *ACCOUNT_HOLDER_NAME_MAX_LENGTH
            },
            err
        )
    }

    #[test]
    fn validate_bank_account_fail_account_holder_name_valid_account_holder_name_length1() {
        let bank_account = BankAccount {
            bank_code: "0123".to_string(),
            branch_code: "456".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "アアア".to_string(),
        };

        validate_bank_account(&bank_account).expect("failed to get Ok");
    }

    #[test]
    fn validate_bank_account_fail_account_holder_name_valid_account_holder_name_length2() {
        let bank_account = BankAccount {
            bank_code: "0123".to_string(),
            branch_code: "456".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "アアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアア".to_string(),
        };

        validate_bank_account(&bank_account).expect("failed to get Ok");
    }

    #[test]
    fn validate_bank_account_fail_account_holder_name_includes_hankaku_space() {
        let bank_account = BankAccount {
            bank_code: "0123".to_string(),
            branch_code: "456".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "タナカ タロウ".to_string(),
        };

        let err = validate_bank_account(&bank_account).expect_err("failed to get Err");

        assert_eq!(
            BankAccountValidationError::IllegalCharInAccountHolderName(
                bank_account.account_holder_name
            ),
            err
        )
    }

    #[test]
    fn validate_bank_account_fail_account_holder_name_includes_hankaku_hiragana() {
        let bank_account = BankAccount {
            bank_code: "0123".to_string(),
            branch_code: "456".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "ﾀﾅｶ　ﾀﾛｳ".to_string(),
        };

        let err = validate_bank_account(&bank_account).expect_err("failed to get Err");

        assert_eq!(
            BankAccountValidationError::IllegalCharInAccountHolderName(
                bank_account.account_holder_name
            ),
            err
        )
    }

    #[test]
    fn validate_bank_account_fail_account_holder_name_includes_hiragana() {
        let bank_account = BankAccount {
            bank_code: "0123".to_string(),
            branch_code: "456".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "たなか　たろう".to_string(),
        };

        let err = validate_bank_account(&bank_account).expect_err("failed to get Err");

        assert_eq!(
            BankAccountValidationError::IllegalCharInAccountHolderName(
                bank_account.account_holder_name
            ),
            err
        )
    }

    #[test]
    fn validate_bank_account_fail_account_holder_name_includes_kanji() {
        let bank_account = BankAccount {
            bank_code: "0123".to_string(),
            branch_code: "456".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "田中　太郎".to_string(),
        };

        let err = validate_bank_account(&bank_account).expect_err("failed to get Err");

        assert_eq!(
            BankAccountValidationError::IllegalCharInAccountHolderName(
                bank_account.account_holder_name
            ),
            err
        )
    }

    #[test]
    fn validate_bank_account_fail_account_holder_name_includes_hiragana_partly() {
        let bank_account = BankAccount {
            bank_code: "0123".to_string(),
            branch_code: "456".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "タナか　たロウ".to_string(),
        };

        let err = validate_bank_account(&bank_account).expect_err("failed to get Err");

        assert_eq!(
            BankAccountValidationError::IllegalCharInAccountHolderName(
                bank_account.account_holder_name
            ),
            err
        )
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_holder_name_is_control_char() {
        let mut bank_account_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: s.to_string() + s + s,
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::IllegalCharInAccountHolderName(
                    bank_account.account_holder_name.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_holder_name_includes_control_char() {
        let mut bank_account_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ".to_string() + s + "タロウ",
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::IllegalCharInAccountHolderName(
                    bank_account.account_holder_name.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_holder_name_starts_with_control_char() {
        let mut bank_account_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: s.to_string() + "タロウ",
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::IllegalCharInAccountHolderName(
                    bank_account.account_holder_name.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_holder_name_ends_with_control_char() {
        let mut bank_account_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　".to_string() + s,
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::IllegalCharInAccountHolderName(
                    bank_account.account_holder_name.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_holder_name_is_symbol() {
        let mut bank_account_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: s.to_string() + s + s,
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::IllegalCharInAccountHolderName(
                    bank_account.account_holder_name.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_holder_name_includes_symbol() {
        let mut bank_account_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ".to_string() + s + "　タロウ",
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::IllegalCharInAccountHolderName(
                    bank_account.account_holder_name.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_holder_name_starts_with_symbol() {
        let mut bank_account_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: s.to_string() + "　タロウ",
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::IllegalCharInAccountHolderName(
                    bank_account.account_holder_name.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_holder_name_ends_with_symbol() {
        let mut bank_account_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string() + s,
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::IllegalCharInAccountHolderName(
                    bank_account.account_holder_name.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_holder_name_is_number() {
        let mut bank_account_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: s.to_string() + s + s,
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::IllegalCharInAccountHolderName(
                    bank_account.account_holder_name.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_holder_name_includes_number() {
        let mut bank_account_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ".to_string() + s + "　タロウ",
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::IllegalCharInAccountHolderName(
                    bank_account.account_holder_name.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_holder_name_starts_with_number() {
        let mut bank_account_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: s.to_string() + "　タロウ",
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::IllegalCharInAccountHolderName(
                    bank_account.account_holder_name.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_bank_account_returns_err_if_account_holder_name_ends_with_number() {
        let mut bank_account_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
            let bank_account = BankAccount {
                bank_code: "0123".to_string(),
                branch_code: "456".to_string(),
                account_type: "普通".to_string(),
                account_number: "1234567".to_string(),
                account_holder_name: "タナカ　タロウ".to_string() + s,
            };
            bank_account_list.push(bank_account);
        }
        for bank_account in bank_account_list {
            let err = validate_bank_account(&bank_account).expect_err("failed to get Err");
            assert_eq!(
                BankAccountValidationError::IllegalCharInAccountHolderName(
                    bank_account.account_holder_name.to_string()
                ),
                err
            );
        }
    }
}
