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
  // mapのキーとしてcodeを利用してる。そのキー同士の比較に === が使われるため、浮動少数（number）を使わないようにstringに変換する。
  // code同士の加減乗除は行っておらず、codeに利用する数字は整数のみのため、問題は発生しないと思われる。
  // しかし、一般論として浮動少数の等価比較（===）は避けるべきと考えられるため、上記の対応を行う。
  const codeStr = code.toString()
  const result = createErrorMessageInternal(codeStr)
  return result
}

export function createErrorMessageInternal (code: string): string {
  const message = codeToMessage.get(code)
  if (!message) {
    throw new Error(`unexpected code: ${code}`)
  }
  return message
}

const codeToMessage = new Map<string, string>()
codeToMessage.set(Code.UNEXPECTED_ERR_COMMON.toString(), `${Message.UNEXPECTED_ERR} (${Code.UNEXPECTED_ERR_COMMON})`)
codeToMessage.set(Code.UNEXPECTED_ERR_ADMIN.toString(), `${Message.UNEXPECTED_ERR} (${Code.UNEXPECTED_ERR_ADMIN})`)
codeToMessage.set(Code.INVALID_EMAIL_ADDRESS_FORMAT.toString(), `${Message.INVALID_EMAIL_ADDRESS_FORMAT_MESSAGE} (${Code.INVALID_EMAIL_ADDRESS_FORMAT})`)
codeToMessage.set(Code.INVALID_PASSWORD_FORMAT.toString(), `${Message.INVALID_PASSWORD_FORMAT_MESSAGE} (${Code.INVALID_PASSWORD_FORMAT})`)
codeToMessage.set(Code.INVALID_UUID_FORMAT.toString(), `${Message.INVALID_UUID_FORMAT_MESSAGE} (${Code.INVALID_UUID_FORMAT})`)
codeToMessage.set(Code.EMAIL_OR_PWD_INCORRECT.toString(), `${Message.EMAIL_OR_PWD_INCORRECT_MESSAGE} (${Code.EMAIL_OR_PWD_INCORRECT})`)
codeToMessage.set(Code.UNAUTHORIZED.toString(), `${Message.UNAUTHORIZED_MESSAGE} (${Code.UNAUTHORIZED})`)
codeToMessage.set(Code.ILLEGAL_PAGE_SIZE.toString(), `${Message.ILLEGAL_PAGE_SIZE_MESSAGE} (${Code.ILLEGAL_PAGE_SIZE})`)
codeToMessage.set(Code.NO_CREATE_IDENTITY_REQ_DETAIL_FOUND.toString(), `${Message.NO_CREATE_IDENTITY_REQ_DETAIL_FOUND_MESSAGE} (${Code.NO_CREATE_IDENTITY_REQ_DETAIL_FOUND})`)
codeToMessage.set(Code.ILLEGAL_DATE.toString(), `${Message.ILLEGAL_DATE_MESSAGE} (${Code.ILLEGAL_DATE})`)
codeToMessage.set(Code.INVALID_FORMAT_REASON.toString(), `${Message.INVALID_FORMAT_REASON_MESSAGE} (${Code.INVALID_FORMAT_REASON})`)
codeToMessage.set(Code.NO_UPDATE_IDENTITY_REQ_DETAIL_FOUND.toString(), `${Message.NO_UPDATE_IDENTITY_REQ_DETAIL_FOUND_MESSAGE} (${Code.NO_UPDATE_IDENTITY_REQ_DETAIL_FOUND})`)
codeToMessage.set(Code.NO_USER_ACCOUNT_FOUND.toString(), `${Message.NO_USER_ACCOUNT_FOUND_MESSAGE} (${Code.NO_USER_ACCOUNT_FOUND})`)
codeToMessage.set(Code.NO_IDENTITY_FOUND.toString(), `${Message.NO_IDENTITY_FOUND_MESSAGE} (${Code.NO_IDENTITY_FOUND})`)
codeToMessage.set(Code.NO_CREATE_CAREER_REQ_DETAIL_FOUND.toString(), `${Message.NO_CREATE_CAREER_REQ_DETAIL_FOUND_MESSAGE} (${Code.NO_CREATE_CAREER_REQ_DETAIL_FOUND})`)
