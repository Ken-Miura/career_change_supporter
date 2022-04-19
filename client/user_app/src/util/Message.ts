const RETRY_REQUEST = '通信環境を確認し、一定時間後に再度お試し下さい'

// classを利用を検討したが、constにできないためnamespaceを選択
// namespaceは、非推奨ではないため、代替可能な手段ができるまで利用
// eslint-disable-next-line
export namespace Message {
    export const PASSWORD_CONFIRMATION_FAILED = 'パスワードと確認用パスワードが一致していません'
    export const TEMP_ACCOUNT_CREATION_FAILED = `新規登録に失敗しました。${RETRY_REQUEST}`
    export const UNEXPECTED_ERR = `予期せぬエラーが発生しました。${RETRY_REQUEST}`
    export const INVALID_EMAIL_ADDRESS_FORMAT_MESSAGE = 'メールアドレスの形式が不正です'
    export const INVALID_PASSWORD_FORMAT_MESSAGE = 'パスワードの形式が不正です。英大文字、英小文字、数字、記号の内、2種類以上を組み合わせた10文字以上32文字以下の文字列を指定して下さい'
    export const ACCOUNT_ALREADY_EXISTS_MESSAGE = 'アカウントが既に存在しています'
    export const REACH_TEMP_ACCOUNTS_LIMIT_MESSAGE = '新規登録試行回数の上限に達しました。数日後、再度新規登録を行って下さい'
    export const INVALID_UUID_FORMAT_MESSAGE = 'UUIDの形式が不正です。URLが提供されたものと一致するかご確認下さい'
    export const TEMP_ACCOUNT_EXPIRED_MESSAGE = '有効期限が過ぎています。もう一度初めから手続きをお願い致します'
    export const NO_TEMP_ACCOUNT_FOUND_MESSAGE = '指定されたURLが見つかりませんでした。URLが提供されたものと一致するかご確認下さい'
    export const INVALID_QUERY_PARAM = '指定されたURLの形式が正しくありません。URLが提供されたものと一致するかご確認下さい'
    export const EMAIL_OR_PWD_INCORRECT_MESSAGE = 'メールアドレス、もしくはパスワードが間違っています'
    export const UNAUTHORIZED_MESSAGE = '認証が必要です'
    export const ACCOUNT_DISABLED_MESSAGE = 'アカウントが無効化されています'
    export const ACCOUNT_CREATED = '新規登録が完了しました。'
    export const ACCOUNT_CREATION_FAILED = `新規登録に失敗しました。${RETRY_REQUEST}`
    export const LOGIN_FAILED = `ログインに失敗しました。${RETRY_REQUEST}`
    export const PASSWORD_CHANGE_REQUEST_FAILED = `パスワード変更に失敗しました。${RETRY_REQUEST}`
    export const REACH_PASSWORD_CHANGE_REQ_LIMIT_MESSAGE = 'パスワード変更試行回数の上限に達しました。数日後、再度パスワード変更を行って下さい'
    export const PWD_CHANGE_REQ_EXPIRED_MESSAGE = TEMP_ACCOUNT_EXPIRED_MESSAGE
    export const PASSWORD_CHANGED_MESSAGE = 'パスワードを変更しました'
    export const NO_ACCOUNT_FOUND_MESSAGE = 'アカウントが存在しません'
    export const NO_PWD_CHANGE_REQ_FOUND_MESSAGE = NO_TEMP_ACCOUNT_FOUND_MESSAGE
    export const REACH_PAYMENT_PLATFORM_RATE_LIMIT_MESSAGE = '多量のリクエストを処理しているため、サーバーに負荷がかかっています。一定時間後に再度お試し下さい。'
    export const NO_IDENTITY_FOUND = 'ユーザー情報が設定されていません。先にユーザー情報を設定して下さい。'
    export const NO_IDENTITY_IMAGE1_SELECTED = '表面の画像が選択されていません。'
    export const INVALID_LAST_NAME_LENGTH_MESSAGE = '氏名の姓の文字数が規定範囲外です。'
    export const ILLEGAL_CHAR_IN_LAST_NAME_MESSAGE = '氏名の姓に不正な文字が含まれています。'
    export const INVALID_FIRST_NAME_LENGTH_MESSAGE = '氏名の名の文字数が規定範囲外です。'
    export const ILLEGAL_CHAR_IN_FIRST_NAME_MESSAGE = '氏名の名に不正な文字が含まれています。'
    export const INVALID_LAST_NAME_FURIGANA_LENGTH_MESSAGE = 'フリガナのセイの文字数が規定範囲外です。'
    export const ILLEGAL_CHAR_IN_LAST_NAME_FURIGANA_MESSAGE = 'フリガナには全角カナのみをご利用下さい。'
    export const INVALID_FIRST_NAME_FURIGANA_LENGTH_MESSAGE = 'フリガナのメイの文字数が規定範囲外です。'
    export const ILLEGAL_CHAR_IN_FIRST_NAME_FURIGANA_MESSAGE = ILLEGAL_CHAR_IN_LAST_NAME_FURIGANA_MESSAGE
    export const ILLEGAL_DATE_MESSAGE = '不正な生年月日です。'
    export const ILLEGAL_AGE_MESSAGE = 'サービスをご利用可能な年齢に達していません。'
    export const INVALID_PREFECTURE_MESSAGE = '不正な都道府県です。'
    export const INVALID_CITY_LENGTH_MESSAGE = '住所の市区町村の文字数が規定範囲外です。'
    export const ILLEGAL_CHAR_IN_CITY_MESSAGE = '住所の市区町村に不正な文字が含まれています。'
    export const INVALID_ADDRESS_LINE1_LENGTH_MESSAGE = '住所の番地の文字数が規定範囲外です。'
    export const ILLEGAL_CHAR_IN_ADDRESS_LINE1_MESSAGE = '住所の番地に不正な文字が含まれています。'
    export const INVALID_ADDRESS_LINE2_LENGTH_MESSAGE = '住所の建物名・部屋番号の文字数が規定範囲外です。'
    export const ILLEGAL_CHAR_IN_ADDRESS_LINE2_MESSAGE = '住所の建物名・部屋番号に不正な文字が含まれています。'
    export const INVALID_TEL_NUM_FORMAT_MESSAGE = '不正な形式の電話番号です。'
    export const NO_NAME_FOUND_MESSAGE = 'HTTPボディのmultipart/form-data内にnameが存在しないデータがあります。'
    export const NO_FILE_NAME_FOUND_MESSAGE = 'HTTPボディのmultipart/form-data内で画像データに対してfilenameが存在していません。'
    export const DATA_PARSE_FAILURE_MESSAGE = 'HTTPボディのmultipart/form-dataに不正なデータが含まれています。'
    export const INVALID_NAME_IN_FIELD_MESSAGE = 'HTTPボディのmultipart/form-data内にnameに不正な文字列が含まれています。'
    export const INVALID_UTF8_SEQUENCE_MESSAGE = 'UTF-8として不正な文字列です。エンコーディング方式にUTF-8を用いてください。'
    export const INVALID_IDENTITY_JSON_MESSAGE = '不正な形式のユーザー情報です。'
    export const NO_JPEG_EXTENSION_MESSAGE = '画像のファイル名がJPEGを示す拡張子ではありません。ファイル名は".jpg"、".jpeg"、".JPG"、".JPEG"または".jpe"で終わる必要があります。'
    export const EXCEED_MAX_IDENTITY_IMAGE_SIZE_LIMIT_MESSAGE = '画像のファイルサイズが4MBを超えています。'
    export const INVALID_JPEG_IMAGE_MESSAGE = '身分証明書の画像がJPEG以外の形式です。身分証明書の画像にはJPEG形式をご利用下さい。'
    export const NO_IDENTITY_FOUND_MESSAGE = 'ユーザー情報が含まれていない不正なリクエストです。'
    export const NO_IDENTITY_IMAGE1_FOUND_MESSAGE = '身分証明書の表面の画像が含まれていない不正なリクエストです。'
    export const IDENTITY_INFO_REQ_ALREADY_EXISTS_MESSAGE = '既に本人確認を依頼済みです。確認作業が終わるまでお待ち下さい。'
    export const POST_IDENTITY_RESULT_MESSAGE = '本人確認を受け付けました。本人確認作業の完了後、入力した値がユーザー情報に反映されます。'
    export const DATE_OF_BIRTH_IS_NOT_MATCH_MESSAGE = '生年月日を変更することはできません'
    export const NO_IDENTITY_UPDATED_MESSAGE = 'ユーザー情報に変更がありません'
    export const FIRST_NAME_IS_NOT_MATCH_MESSAGE = '氏名の内、名の変更はできません'
}
