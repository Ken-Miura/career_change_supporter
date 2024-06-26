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
  export const INVALID_PASS_CODE_FORMAT = 30002
  export const MFA_IS_NOT_ENABLED = 30003
  export const PASS_CODE_DOES_NOT_MATCH = 30004
  export const UNAUTHORIZED = 30005
  export const NO_ACCOUNT_FOUND = 30006
  export const ILLEGAL_PAGE_SIZE = 30007
  export const NO_CREATE_IDENTITY_REQ_DETAIL_FOUND = 30008
  export const ILLEGAL_DATE = 30009
  export const INVALID_FORMAT_REASON = 30010
  export const NO_UPDATE_IDENTITY_REQ_DETAIL_FOUND = 30011
  export const NO_IDENTITY_FOUND = 30012
  export const NO_CREATE_CAREER_REQ_DETAIL_FOUND = 30013
  export const ACCOUNT_ID_IS_NOT_POSITIVE = 30014
  export const CONSULTATION_ID_IS_NOT_POSITIVE = 30015
  export const ILLEGAL_DATE_TIME = 30022
  export const ILLEGAL_MAINTENANCE_DATE_TIME = 30023
  export const MAINTENANCE_ALREADY_HAS_BEEN_SET = 30024
  export const EXCEEDS_MAX_MAINTENANCE_DURATION_LIMIT = 30025
  export const INVALID_TITLE_LENGTH = 30026
  export const ILLEGAL_TITLE = 30027
  export const INVALID_BODY_LENGTH = 30028
  export const ILLEGAL_BODY = 30029
  export const INVALID_NEWS_ID = 30030
  export const NO_AWAITING_PAYMENT_FOUND = 30031
  export const NO_AWAITING_WITHDRAWAL_FOUND = 30032
}

export function createErrorMessage (code: number): string {
  // mapのキーとしてcodeを利用してる。そのキー同士の比較に === が使われるため、浮動少数（number）を使わないようにstringに変換する。
  // code同士の加減乗除は行っておらず、codeに利用する数字は整数のみのため、問題は発生しないと思われる。
  // しかし、一般論として浮動少数の等価比較（===）は避けるべきと考えられるため、上記の対応を行う。
  const codeStr = code.toString()
  const result = createErrorMessageInternal(codeStr)
  return result
}

function createErrorMessageInternal (code: string): string {
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
codeToMessage.set(Code.INVALID_PASS_CODE_FORMAT.toString(), `${Message.INVALID_PASS_CODE_FORMAT_MESSAGE} (${Code.INVALID_PASS_CODE_FORMAT})`)
codeToMessage.set(Code.MFA_IS_NOT_ENABLED.toString(), `${Message.MFA_IS_NOT_ENABLED_MESSAGE} (${Code.MFA_IS_NOT_ENABLED})`)
codeToMessage.set(Code.PASS_CODE_DOES_NOT_MATCH.toString(), `${Message.PASS_CODE_DOES_NOT_MATCH_MESSAGE} (${Code.PASS_CODE_DOES_NOT_MATCH})`)
codeToMessage.set(Code.UNAUTHORIZED.toString(), `${Message.UNAUTHORIZED_MESSAGE} (${Code.UNAUTHORIZED})`)
codeToMessage.set(Code.NO_ACCOUNT_FOUND.toString(), `${Message.NO_ACCOUNT_FOUND_MESSAGE} (${Code.NO_ACCOUNT_FOUND})`)
codeToMessage.set(Code.ILLEGAL_PAGE_SIZE.toString(), `${Message.ILLEGAL_PAGE_SIZE_MESSAGE} (${Code.ILLEGAL_PAGE_SIZE})`)
codeToMessage.set(Code.NO_CREATE_IDENTITY_REQ_DETAIL_FOUND.toString(), `${Message.NO_CREATE_IDENTITY_REQ_DETAIL_FOUND_MESSAGE} (${Code.NO_CREATE_IDENTITY_REQ_DETAIL_FOUND})`)
codeToMessage.set(Code.ILLEGAL_DATE.toString(), `${Message.ILLEGAL_DATE_MESSAGE} (${Code.ILLEGAL_DATE})`)
codeToMessage.set(Code.INVALID_FORMAT_REASON.toString(), `${Message.INVALID_FORMAT_REASON_MESSAGE} (${Code.INVALID_FORMAT_REASON})`)
codeToMessage.set(Code.NO_UPDATE_IDENTITY_REQ_DETAIL_FOUND.toString(), `${Message.NO_UPDATE_IDENTITY_REQ_DETAIL_FOUND_MESSAGE} (${Code.NO_UPDATE_IDENTITY_REQ_DETAIL_FOUND})`)
codeToMessage.set(Code.NO_IDENTITY_FOUND.toString(), `${Message.NO_IDENTITY_FOUND_MESSAGE} (${Code.NO_IDENTITY_FOUND})`)
codeToMessage.set(Code.NO_CREATE_CAREER_REQ_DETAIL_FOUND.toString(), `${Message.NO_CREATE_CAREER_REQ_DETAIL_FOUND_MESSAGE} (${Code.NO_CREATE_CAREER_REQ_DETAIL_FOUND})`)
codeToMessage.set(Code.ACCOUNT_ID_IS_NOT_POSITIVE.toString(), `${Message.ACCOUNT_ID_IS_NOT_POSITIVE_MESSAGE} (${Code.ACCOUNT_ID_IS_NOT_POSITIVE})`)
codeToMessage.set(Code.CONSULTATION_ID_IS_NOT_POSITIVE.toString(), `${Message.CONSULTATION_ID_IS_NOT_POSITIVE_MESSAGE} (${Code.CONSULTATION_ID_IS_NOT_POSITIVE})`)
codeToMessage.set(Code.ILLEGAL_DATE_TIME.toString(), `${Message.ILLEGAL_DATE_TIME_MESSAGE} (${Code.ILLEGAL_DATE_TIME})`)
codeToMessage.set(Code.ILLEGAL_MAINTENANCE_DATE_TIME.toString(), `${Message.ILLEGAL_MAINTENANCE_DATE_TIME_MESSAGE} (${Code.ILLEGAL_MAINTENANCE_DATE_TIME})`)
codeToMessage.set(Code.MAINTENANCE_ALREADY_HAS_BEEN_SET.toString(), `${Message.MAINTENANCE_ALREADY_HAS_BEEN_SET_MESSAGE} (${Code.MAINTENANCE_ALREADY_HAS_BEEN_SET})`)
codeToMessage.set(Code.EXCEEDS_MAX_MAINTENANCE_DURATION_LIMIT.toString(), `${Message.EXCEEDS_MAX_MAINTENANCE_DURATION_LIMIT_MESSAGE} (${Code.EXCEEDS_MAX_MAINTENANCE_DURATION_LIMIT})`)
codeToMessage.set(Code.INVALID_TITLE_LENGTH.toString(), `${Message.INVALID_TITLE_LENGTH_MESSAGE} (${Code.INVALID_TITLE_LENGTH})`)
codeToMessage.set(Code.ILLEGAL_TITLE.toString(), `${Message.ILLEGAL_TITLE_MESSAGE} (${Code.ILLEGAL_TITLE})`)
codeToMessage.set(Code.INVALID_BODY_LENGTH.toString(), `${Message.INVALID_BODY_LENGTH_MESSAGE} (${Code.INVALID_BODY_LENGTH})`)
codeToMessage.set(Code.ILLEGAL_BODY.toString(), `${Message.ILLEGAL_BODY_MESSAGE} (${Code.ILLEGAL_BODY})`)
codeToMessage.set(Code.INVALID_NEWS_ID.toString(), `${Message.INVALID_NEWS_ID_MESSAGE} (${Code.INVALID_NEWS_ID})`)
codeToMessage.set(Code.NO_AWAITING_PAYMENT_FOUND.toString(), `${Message.NO_AWAITING_PAYMENT_FOUND_MESSAGE} (${Code.NO_AWAITING_PAYMENT_FOUND})`)
codeToMessage.set(Code.NO_AWAITING_WITHDRAWAL_FOUND.toString(), `${Message.NO_AWAITING_WITHDRAWAL_FOUND_MESSAGE} (${Code.NO_AWAITING_WITHDRAWAL_FOUND})`)
