const UNEXPECTED_ERR_COMMON = 10000
const INVALID_EMAIL_ADDRESS_FORMAT = 10001
const INVALID_PASSWORD_FORMAT = 10002

const UNEXPECTED_ERR_USER = 20000
const ACCOUNT_ALREADY_EXISTS = 20001
const REACH_TEMP_ACCOUNTS_LIMIT = 20002
const INVALID_UUID = 20003
const TEMP_ACCOUNT_EXPIRED = 20004
const NO_TEMP_ACCOUNT_FOUND = 20005
const EMAIL_OR_PWD_INCORRECT = 20006
const UNAUTHORIZED = 20007

export function createErrorMessage (code: number): string {
  if (code === UNEXPECTED_ERR_COMMON || code === UNEXPECTED_ERR_USER) {
    return `予期せぬエラーが発生しました。一定時間後に再度お試し下さい (${code})`
  } else if (code === INVALID_EMAIL_ADDRESS_FORMAT) {
    return `メールアドレスの形式が不正です (${code})`
  } else if (code === INVALID_PASSWORD_FORMAT) {
    return `パスワードの形式が不正です。英大文字、英小文字、数字、記号の内、2種類以上を組み合わせた10文字以上32文字以下の文字列を指定して下さい (${code})`
  } else if (code === ACCOUNT_ALREADY_EXISTS) {
    return `アカウントが既に存在しています (${code})`
  } else if (code === REACH_TEMP_ACCOUNTS_LIMIT) {
    return `新規作成依頼回数の上限に達しました。数日後、再度新規作成をお試し下さい (${code})`
  } else if (code === INVALID_UUID) {
    return `UUIDの形式が不正です。URLが提供されたものと一致するかご確認下さい (${code})`
  } else if (code === TEMP_ACCOUNT_EXPIRED) {
    return `有効期限が過ぎています。もう一度初めから手続きをお願い致します (${code})`
  } else if (code === NO_TEMP_ACCOUNT_FOUND) {
    return `指定されたURLが見つかりませんでした。URLが提供されたものと一致するかご確認下さい (${code})`
  } else if (code === EMAIL_OR_PWD_INCORRECT) {
    return `メールアドレス、もしくはパスワードが間違っています (${code})`
  } else if (code === UNAUTHORIZED) {
    return `認証が必要です (${code})`
  } else {
    throw new Error(`unexpected code: ${code}`)
  }
}
