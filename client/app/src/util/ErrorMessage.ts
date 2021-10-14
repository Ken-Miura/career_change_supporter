import { Message } from '@/util/Message'

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
    return `${Message.UNEXPECTED_ERR} (${code})`
  } else if (code === INVALID_EMAIL_ADDRESS_FORMAT) {
    return `${Message.INVALID_EMAIL_ADDRESS_FORMAT_MESSAGE} (${code})`
  } else if (code === INVALID_PASSWORD_FORMAT) {
    return `${Message.INVALID_PASSWORD_FORMAT_MESSAGE} (${code})`
  } else if (code === ACCOUNT_ALREADY_EXISTS) {
    return `${Message.ACCOUNT_ALREADY_EXISTS_MESSAGE} (${code})`
  } else if (code === REACH_TEMP_ACCOUNTS_LIMIT) {
    return `${Message.REACH_TEMP_ACCOUNTS_LIMIT_MESSAGE} (${code})`
  } else if (code === INVALID_UUID) {
    return `${Message.INVALID_UUID_MESSAGE} (${code})`
  } else if (code === TEMP_ACCOUNT_EXPIRED) {
    return `${Message.TEMP_ACCOUNT_EXPIRED_MESSAGE} (${code})`
  } else if (code === NO_TEMP_ACCOUNT_FOUND) {
    return `${Message.NO_TEMP_ACCOUNT_FOUND_MESSAGE} (${code})`
  } else if (code === EMAIL_OR_PWD_INCORRECT) {
    return `${Message.EMAIL_OR_PWD_INCORRECT_MESSAGE} (${code})`
  } else if (code === UNAUTHORIZED) {
    return `${Message.UNAUTHORIZED_MESSAGE} (${code})`
  } else {
    throw new Error(`unexpected code: ${code}`)
  }
}
