const RETRY_REQUEST = '通信環境を確認し、一定時間後に再度お試し下さい'

// classを利用を検討したが、constにできないためnamespaceを選択
// namespaceは、非推奨ではないため、代替可能な手段ができるまで利用
// eslint-disable-next-line
export namespace Message {
    export const PASSWORD_CONFIRMATION_FAILED = 'パスワードと確認用パスワードが一致していません'
    export const NEW_ACCOUNT_CREATION_FAILED = `新規登録に失敗しました。${RETRY_REQUEST}`
    export const UNEXPECTED_ERR = `予期せぬエラーが発生しました。${RETRY_REQUEST}`
    export const INVALID_EMAIL_ADDRESS_FORMAT_MESSAGE = 'メールアドレスの形式が不正です'
    export const INVALID_PASSWORD_FORMAT_MESSAGE = 'パスワードの形式が不正です。英大文字、英小文字、数字、記号の内、2種類以上を組み合わせた10文字以上32文字以下の文字列を指定して下さい'
    export const ACCOUNT_ALREADY_EXISTS_MESSAGE = 'アカウントが既に存在しています'
    export const REACH_TEMP_ACCOUNTS_LIMIT_MESSAGE = '新規作成試行回数の上限に達しました。数日後、再度新規作成を行って下さい'
    export const INVALID_UUID_MESSAGE = 'UUIDの形式が不正です。URLが提供されたものと一致するかご確認下さい'
    export const TEMP_ACCOUNT_EXPIRED_MESSAGE = '有効期限が過ぎています。もう一度初めから手続きをお願い致します'
    export const NO_TEMP_ACCOUNT_FOUND_MESSAGE = '指定されたURLが見つかりませんでした。URLが提供されたものと一致するかご確認下さい'
    export const EMAIL_OR_PWD_INCORRECT_MESSAGE = 'メールアドレス、もしくはパスワードが間違っています'
    export const UNAUTHORIZED_MESSAGE = '認証が必要です'
    export const ACCOUNT_CREATED = 'アカウントの作成が完了しました。'
    export const ACCOUNT_CREATION_FAILED = `アカウントの作成に失敗しました。${RETRY_REQUEST}`
}
