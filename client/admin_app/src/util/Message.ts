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
    export const LOGIN_FAILED = `ログインに失敗しました。${RETRY_REQUEST}`
    export const UNAUTHORIZED_MESSAGE = '認証が必要です'
    export const ILLEGAL_PAGE_SIZE_MESSAGE = `${ASK_ADMIN}`
    export const NO_CREATE_IDENTITY_REQ_DETAIL_FOUND_MESSAGE = `${ASK_ADMIN}`
    export const ILLEGAL_DATE_MESSAGE = `${ASK_ADMIN}`
    export const INVALID_FORMAT_REASON_MESSAGE = `${ASK_ADMIN}`
    export const NO_UPDATE_IDENTITY_REQ_DETAIL_FOUND_MESSAGE = `${ASK_ADMIN}`
    export const NO_USER_ACCOUNT_FOUND_MESSAGE = 'ユーザーアカウントが存在しません。既にアカウントが削除されているものと思われます。次の確認依頼に進んで下さい'
    export const NO_IDENTITY_FOUND_MESSAGE = `${ASK_ADMIN}`
    export const NO_CREATE_CAREER_REQ_DETAIL_FOUND_MESSAGE = `${ASK_ADMIN}`
}
