import { Message } from '@/util/Message'

// classを利用を検討したが、constにできないためnamespaceを選択
// namespaceは、非推奨ではないため、代替可能な手段ができるまで利用
// eslint-disable-next-line
export namespace Code {
  export const UNEXPECTED_ERR_COMMON = 10000
  export const INVALID_EMAIL_ADDRESS_FORMAT = 10001
  export const INVALID_PASSWORD_FORMAT = 10002
  export const INVALID_UUID_FORMAT = 10003

  export const UNEXPECTED_ERR_ADMIN = 30000
  export const EMAIL_OR_PWD_INCORRECT = 30001
  export const UNAUTHORIZED = 30002
  export const ILLEGAL_PAGE_SIZE = 30003
  export const NO_CREATE_IDENTITY_REQ_DETAIL_FOUND = 30004
  export const ILLEGAL_DATE = 30005
  export const INVALID_FORMAT_REASON = 30006
  export const NO_UPDATE_IDENTITY_REQ_DETAIL_FOUND = 30007
  export const NO_USER_ACCOUNT_FOUND = 30008
  export const NO_IDENTITY_FOUND = 30009
  export const NO_CREATE_CAREER_REQ_DETAIL_FOUND = 30010
}

export function createErrorMessage (code: number): string {
  if (code === Code.UNEXPECTED_ERR_COMMON || code === Code.UNEXPECTED_ERR_ADMIN) {
    return `${Message.UNEXPECTED_ERR} (${code})`
  } else if (code === Code.INVALID_EMAIL_ADDRESS_FORMAT) {
    return `${Message.INVALID_EMAIL_ADDRESS_FORMAT_MESSAGE} (${code})`
  } else if (code === Code.INVALID_PASSWORD_FORMAT) {
    return `${Message.INVALID_PASSWORD_FORMAT_MESSAGE} (${code})`
  } else if (code === Code.INVALID_UUID_FORMAT) {
    return `${Message.INVALID_UUID_FORMAT_MESSAGE} (${code})`
  } else if (code === Code.EMAIL_OR_PWD_INCORRECT) {
    return `${Message.EMAIL_OR_PWD_INCORRECT_MESSAGE} (${code})`
  } else if (code === Code.UNAUTHORIZED) {
    return `${Message.UNAUTHORIZED_MESSAGE} (${code})`
  } else if (code === Code.ILLEGAL_PAGE_SIZE) {
    return `${Message.ILLEGAL_PAGE_SIZE_MESSAGE} (${code})`
  } else if (code === Code.NO_CREATE_IDENTITY_REQ_DETAIL_FOUND) {
    return `${Message.NO_CREATE_IDENTITY_REQ_DETAIL_FOUND_MESSAGE} (${code})`
  } else if (code === Code.ILLEGAL_DATE) {
    return `${Message.ILLEGAL_DATE_MESSAGE} (${code})`
  } else if (code === Code.INVALID_FORMAT_REASON) {
    return `${Message.INVALID_FORMAT_REASON_MESSAGE} (${code})`
  } else if (code === Code.NO_UPDATE_IDENTITY_REQ_DETAIL_FOUND) {
    return `${Message.NO_UPDATE_IDENTITY_REQ_DETAIL_FOUND_MESSAGE} (${code})`
  } else if (code === Code.NO_USER_ACCOUNT_FOUND) {
    return `${Message.NO_USER_ACCOUNT_FOUND_MESSAGE} (${code})`
  } else if (code === Code.NO_IDENTITY_FOUND) {
    return `${Message.NO_IDENTITY_FOUND_MESSAGE} (${code})`
  } else if (code === Code.NO_CREATE_CAREER_REQ_DETAIL_FOUND) {
    return `${Message.NO_CREATE_CAREER_REQ_DETAIL_FOUND_MESSAGE} (${code})`
  } else {
    throw new Error(`unexpected code: ${code}`)
  }
}
