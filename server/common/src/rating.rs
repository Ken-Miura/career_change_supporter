// Copyright 2023 Ken Miura

pub fn calculate_average_rating(ratings: Vec<i16>) -> f64 {
    let size = ratings.len();
    let mut sum = 0;
    for rating in ratings {
        sum += rating as usize
    }
    (sum / size) as f64
    // sum as f64 / size as f64
}

/// 小数点以下2桁目を四捨五入し、小数点以下1桁目までを示す少数を文字列表現として返す。
pub fn round_rating_to_one_decimal_places(rating: f64) -> String {
    let result = (rating * 10.0).round() / 10.0;
    // format!("{:.1}", rating) のみで少数点以下2桁目を四捨五入し、小数点以下1桁まで求める動作となる。
    // しかし、下記のドキュメントに、その動作（四捨五入）に関して正式な仕様として記載がないため、四捨五入の箇所は自身で実装する。
    // https://doc.rust-lang.org/std/fmt/
    format!("{:.1}", result)
}

#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(Debug)]
    struct RoundToOneDecimalPlacesTestCase {
        name: String,
        input: f64,
        expected: String,
    }

    static ROUNT_TO_ONE_DECIMAL_PLACES_TEST_CASE_SET: Lazy<Vec<RoundToOneDecimalPlacesTestCase>> =
        Lazy::new(|| {
            vec![
                RoundToOneDecimalPlacesTestCase {
                    name: "x.x4 -> round down".to_string(),
                    input: 3.64,
                    expected: "3.6".to_string(),
                },
                RoundToOneDecimalPlacesTestCase {
                    name: "x.x5 -> round up".to_string(),
                    input: 3.65,
                    expected: "3.7".to_string(),
                },
                RoundToOneDecimalPlacesTestCase {
                    name: "x.95 -> round up".to_string(),
                    input: 3.95,
                    expected: "4.0".to_string(),
                },
                RoundToOneDecimalPlacesTestCase {
                    name: "x.x0 -> round down".to_string(),
                    input: 4.10,
                    expected: "4.1".to_string(),
                },
                RoundToOneDecimalPlacesTestCase {
                    name: "x.x9 -> round up".to_string(),
                    input: 2.19,
                    expected: "2.2".to_string(),
                },
            ]
        });

    #[test]
    fn test_round_rating_to_one_decimal_places() {
        for test_case in ROUNT_TO_ONE_DECIMAL_PLACES_TEST_CASE_SET.iter() {
            let result = round_rating_to_one_decimal_places(test_case.input);
            let message = format!("test case \"{}\" failed", test_case.name.clone());
            assert_eq!(test_case.expected, result, "{}", message);
        }
    }
}
