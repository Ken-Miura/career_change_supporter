const RETRY_REQUEST = '通信環境を確認し、一定時間後に再度お試し下さい'
const ASK_ADMIN = '予期せぬエラーが発生しました。表示されている数字を管理者にお伝え下さい'

// classを利用を検討したが、constにできないためnamespaceを選択
// namespaceは、非推奨ではないため、代替可能な手段ができるまで利用
// eslint-disable-next-line
export namespace Message {
    export const UNEXPECTED_ERR = `予期せぬエラーが発生しました。${RETRY_REQUEST}`
    export const INVALID_EMAIL_ADDRESS_FORMAT_MESSAGE = 'メールアドレスの形式が不正です'
    export const INVALID_PASSWORD_FORMAT_MESSAGE = 'パスワードの形式が不正です。英大文字、英小文字、数字、記号の内、2種類以上を組み合わせた10文字以上32文字以下の文字列を指定して下さい'
    export const INVALID_UUID_FORMAT_MESSAGE = 'UUIDの形式が不正です。URLが提供されたものと一致するかご確認下さい'
    export const EMAIL_OR_PWD_INCORRECT_MESSAGE = 'メールアドレス、もしくはパスワードが間違っています'
    export const INVALID_PASS_CODE_FORMAT_MESSAGE = '不正な形式のパスコードです'
    export const MFA_IS_NOT_ENABLED_MESSAGE = '二段階認証が有効になっていません'
    export const PASS_CODE_DOES_NOT_MATCH_MESSAGE = 'パスコードが間違っています'
    export const LOGIN_FAILED = `ログインに失敗しました。${RETRY_REQUEST}`
    export const UNAUTHORIZED_MESSAGE = '認証が必要です'
    export const NO_ACCOUNT_FOUND_MESSAGE = 'アカウントが存在しません'
    export const ILLEGAL_PAGE_SIZE_MESSAGE = `${ASK_ADMIN}`
    export const NO_CREATE_IDENTITY_REQ_DETAIL_FOUND_MESSAGE = `${ASK_ADMIN}`
    export const ILLEGAL_DATE_MESSAGE = `${ASK_ADMIN}`
    export const INVALID_FORMAT_REASON_MESSAGE = `${ASK_ADMIN}`
    export const NO_UPDATE_IDENTITY_REQ_DETAIL_FOUND_MESSAGE = `${ASK_ADMIN}`
    export const NO_IDENTITY_FOUND_MESSAGE = `${ASK_ADMIN}`
    export const NO_CREATE_CAREER_REQ_DETAIL_FOUND_MESSAGE = `${ASK_ADMIN}`
    export const BOTH_ACCOUNT_ID_AND_EMAIL_ADDRESS_ARE_EMPTY_MESSAGE = 'アカウントIDとメールアドレスの両方が空です'
    export const BOTH_ACCOUNT_ID_AND_EMAIL_ADDRESS_ARE_FILLED_MESSAGE = 'アカウントIDとメールアドレスの両方が入力されています'
    export const USER_ACCOUNT_SEARCH_PARAM_IS_NULL = '検索に必要なパラメータが設定されていません'
    export const ACCOUNT_ID_IS_NOT_POSITIVE_MESSAGE = 'アカウントIDが正の数ではありません'
    export const CONSULTATION_ID_IS_NOT_POSITIVE_MESSAGE = '相談IDが正の数ではありません'
    export const SETTLEMENT_ID_IS_NOT_POSITIVE_MESSAGE = '決済IDが正の数ではありません'
    export const STOPPED_SETTLEMENT_ID_IS_NOT_POSITIVE_MESSAGE = '停止中決済IDが正の数ではありません'
    export const CREDIT_FACILITIES_ALREADY_EXPIRED_MESSAGE = '確保した与信枠の有効期限が過ぎています'
    export const PAYMENT_RELATED_ERR_MESSAGE = 'PAY.JP関連のエラーです'
    export const RECEIPT_ID_IS_NOT_POSITIVE_MESSAGE = '領収書IDが正の数ではありません'
    export const EXCEEDS_REFUND_TIME_LIMIT_MESSAGE = '返金可能な期限を過ぎています'
    export const ILLEGAL_DATE_TIME_MESSAGE = '不正な日時です'
    export const ILLEGAL_MAINTENANCE_DATE_TIME_MESSAGE = '不正なメンテナンス日時です'
    export const MAINTENANCE_ALREADY_HAS_BEEN_SET_MESSAGE = '既にメンテナンスが設定されています'
    export const EXCEEDS_MAX_MAINTENANCE_DURATION_LIMIT_MESSAGE = 'メンテナンスに設定可能な最大期間（72時間）を超えています'
    export const INVALID_TITLE_LENGTH_MESSAGE = 'タイトルの長さが不正です'
    export const ILLEGAL_TITLE_MESSAGE = 'タイトルに不正な文字があります（制御文字や半角文字を含むことは出来ません）'
    export const INVALID_BODY_LENGTH_MESSAGE = '本文の長さが不正です'
    export const ILLEGAL_BODY_MESSAGE = '本文に不正な文字があります（制御文字や半角文字を含むことは出来ません）'
    export const INVALID_NEWS_ID_MESSAGE = '不正なお知らせIDです'
    export const NO_AWAITING_PAYMENT_FOUND_MESSAGE = '対象が存在しません。既に処理されている可能性があります'
    export const NO_AWAITING_WITHDRAWAL_FOUND_MESSAGE = `${NO_AWAITING_PAYMENT_FOUND_MESSAGE}`
}
