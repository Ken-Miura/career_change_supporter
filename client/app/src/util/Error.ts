import { Message } from '@/util/Message'

// classを利用を検討したが、constにできないためnamespaceを選択
// namespaceは、非推奨ではないため、代替可能な手段ができるまで利用
// eslint-disable-next-line
export namespace Code {
  export const UNEXPECTED_ERR_COMMON = 10000
  export const INVALID_EMAIL_ADDRESS_FORMAT = 10001
  export const INVALID_PASSWORD_FORMAT = 10002
  export const INVALID_UUID_FORMAT = 10003

  export const UNEXPECTED_ERR_USER = 20000
  export const ACCOUNT_ALREADY_EXISTS = 20001
  export const REACH_TEMP_ACCOUNTS_LIMIT = 20002
  export const TEMP_ACCOUNT_EXPIRED = 20003
  export const NO_TEMP_ACCOUNT_FOUND = 20004
  export const EMAIL_OR_PWD_INCORRECT = 20005
  export const UNAUTHORIZED = 20006
  export const NOT_TERMS_OF_USE_AGREED_YET = 20007
  export const ALREADY_AGREED_TERMS_OF_USE = 20008
  export const REACH_PASSWORD_CHANGE_REQ_LIMIT = 20009
  export const NO_ACCOUNT_FOUND = 20010
  export const NO_PWD_CHANGE_REQ_FOUND = 20011
  export const PWD_CHANGE_REQ_EXPIRED = 20012
  export const REACH_PAYMENT_PLATFORM_RATE_LIMIT = 20013
}

export function createErrorMessage (code: number): string {
  if (code === Code.UNEXPECTED_ERR_COMMON || code === Code.UNEXPECTED_ERR_USER) {
    return `${Message.UNEXPECTED_ERR} (${code})`
  } else if (code === Code.INVALID_EMAIL_ADDRESS_FORMAT) {
    return `${Message.INVALID_EMAIL_ADDRESS_FORMAT_MESSAGE} (${code})`
  } else if (code === Code.INVALID_PASSWORD_FORMAT) {
    return `${Message.INVALID_PASSWORD_FORMAT_MESSAGE} (${code})`
  } else if (code === Code.INVALID_UUID_FORMAT) {
    return `${Message.INVALID_UUID_MESSAGE} (${code})`
  } else if (code === Code.ACCOUNT_ALREADY_EXISTS) {
    return `${Message.ACCOUNT_ALREADY_EXISTS_MESSAGE} (${code})`
  } else if (code === Code.REACH_TEMP_ACCOUNTS_LIMIT) {
    return `${Message.REACH_TEMP_ACCOUNTS_LIMIT_MESSAGE} (${code})`
  } else if (code === Code.TEMP_ACCOUNT_EXPIRED) {
    return `${Message.TEMP_ACCOUNT_EXPIRED_MESSAGE} (${code})`
  } else if (code === Code.NO_TEMP_ACCOUNT_FOUND) {
    return `${Message.NO_TEMP_ACCOUNT_FOUND_MESSAGE} (${code})`
  } else if (code === Code.EMAIL_OR_PWD_INCORRECT) {
    return `${Message.EMAIL_OR_PWD_INCORRECT_MESSAGE} (${code})`
  } else if (code === Code.UNAUTHORIZED) {
    return `${Message.UNAUTHORIZED_MESSAGE} (${code})`
  } else if (code === Code.REACH_PASSWORD_CHANGE_REQ_LIMIT) {
    return `${Message.REACH_PASSWORD_CHANGE_REQ_LIMIT_MESSAGE} (${code})`
  } else if (code === Code.PWD_CHANGE_REQ_EXPIRED) {
    return `${Message.PWD_CHANGE_REQ_EXPIRED_MESSAGE} (${code})`
  } else if (code === Code.NO_ACCOUNT_FOUND) {
    return `${Message.NO_ACCOUNT_FOUND_MESSAGE} (${code})`
  } else if (code === Code.NO_PWD_CHANGE_REQ_FOUND) {
    return `${Message.NO_PWD_CHANGE_REQ_FOUND_MESSAGE} (${code})`
  } else if (code === Code.REACH_PAYMENT_PLATFORM_RATE_LIMIT) {
    return `${Message.REACH_PAYMENT_PLATFORM_RATE_LIMIT_MESSAGE} (${code})`
  } else {
    throw new Error(`unexpected code: ${code}`)
  }
}
