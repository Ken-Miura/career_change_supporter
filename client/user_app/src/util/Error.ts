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
  export const ACCOUNT_DISABLED = 20007
  export const NOT_TERMS_OF_USE_AGREED_YET = 20008
  export const ALREADY_AGREED_TERMS_OF_USE = 20009
  export const REACH_PASSWORD_CHANGE_REQ_LIMIT = 20010
  export const NO_ACCOUNT_FOUND = 20011
  export const NO_PWD_CHANGE_REQ_FOUND = 20012
  export const PWD_CHANGE_REQ_EXPIRED = 20013
  export const REACH_PAYMENT_PLATFORM_RATE_LIMIT = 20014
  export const INVALID_LAST_NAME_LENGTH = 20015
  export const ILLEGAL_CHAR_IN_LAST_NAME = 20016
  export const INVALID_FIRST_NAME_LENGTH = 20017
  export const ILLEGAL_CHAR_IN_FIRST_NAME = 20018
  export const INVALID_LAST_NAME_FURIGANA_LENGTH = 20019
  export const ILLEGAL_CHAR_IN_LAST_NAME_FURIGANA = 20020
  export const INVALID_FIRST_NAME_FURIGANA_LENGTH = 20021
  export const ILLEGAL_CHAR_IN_FIRST_NAME_FURIGANA = 20022
  export const ILLEGAL_DATE = 20023
  export const ILLEGAL_AGE = 20024
  export const INVALID_PREFECTURE = 20025
  export const INVALID_CITY_LENGTH = 20026
  export const ILLEGAL_CHAR_IN_CITY = 20027
  export const INVALID_ADDRESS_LINE1_LENGTH = 20028
  export const ILLEGAL_CHAR_IN_ADDRESS_LINE1 = 20029
  export const INVALID_ADDRESS_LINE2_LENGTH = 20030
  export const ILLEGAL_CHAR_IN_ADDRESS_LINE2 = 20031
  export const INVALID_TEL_NUM_FORMAT = 20032
  export const NO_NAME_FOUND = 20033
  export const NO_FILE_NAME_FOUND = 20034
  export const DATA_PARSE_FAILURE = 20035
  export const INVALID_NAME_IN_FIELD = 20036
  export const INVALID_UTF8_SEQUENCE = 20037
  export const INVALID_IDENTITY_JSON = 20038
  export const NO_JPEG_EXTENSION = 20039
  export const EXCEED_MAX_IDENTITY_IMAGE_SIZE_LIMIT = 20040
  export const INVALID_JPEG_IMAGE = 20041
  export const NO_IDENTITY_FOUND = 20042
  export const NO_IDENTITY_IMAGE1_FOUND = 20043
  export const IDENTITY_INFO_REQ_ALREADY_EXISTS = 20044
  export const DATE_OF_BIRTH_IS_NOT_MATCH = 20045
  export const NO_IDENTITY_UPDATED = 20046
  export const FIRST_NAME_IS_NOT_MATCH = 20047
  export const INVALID_MULTIPART_FORM_DATA = 20048
  export const INVALID_CAREER_JSON = 20049
  export const INVALID_COMPANY_NAME_LENGTH = 20050
  export const ILLEGAL_CHAR_IN_COMPANY_NAME = 20051
  export const INVALID_DEPARTMENT_NAME_LENGTH = 20052
  export const ILLEGAL_CHAR_IN_DEPARTMENT_NAME = 20053
  export const INVALID_OFFICE_LENGTH = 20054
  export const ILLEGAL_CHAR_IN_OFFICE = 20055
  export const ILLEGAL_CAREER_START_DATE = 20056
  export const ILLEGAL_CAREER_END_DATE = 20057
  export const CAREER_START_DATE_EXCEEDS_CAREER_END_DATE = 20058
  export const ILLEGAL_CONTRACT_TYPE = 20059
  export const INVALID_PROFESSION_LENGTH = 20060
  export const ILLEGAL_CHAR_IN_PROFESSION = 20061
  export const ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN = 20062
  export const INVALID_POSITION_NAME_LENGTH = 20063
  export const ILLEGAL_CHAR_IN_POSITION_NAME = 20064
  export const INVALID_NOTE_LENGTH = 20065
  export const ILLEGAL_CHAR_IN_NOTE = 20066
  export const NO_CAREER_FOUND = 20067
  export const NO_CAREER_IMAGE1_FOUND = 20068
  export const EXCEED_MAX_CAREER_IMAGE_SIZE_LIMIT = 20069
  export const REACH_CAREER_NUM_LIMIT = 20070
  export const NO_IDENTITY_REGISTERED = 20071
  export const REACH_CREATE_CAREER_REQ_NUM_LIMIT = 20072
}

export function createErrorMessage (code: number): string {
  if (code === Code.UNEXPECTED_ERR_COMMON || code === Code.UNEXPECTED_ERR_USER) {
    return `${Message.UNEXPECTED_ERR} (${code})`
  } else if (code === Code.INVALID_EMAIL_ADDRESS_FORMAT) {
    return `${Message.INVALID_EMAIL_ADDRESS_FORMAT_MESSAGE} (${code})`
  } else if (code === Code.INVALID_PASSWORD_FORMAT) {
    return `${Message.INVALID_PASSWORD_FORMAT_MESSAGE} (${code})`
  } else if (code === Code.INVALID_UUID_FORMAT) {
    return `${Message.INVALID_UUID_FORMAT_MESSAGE} (${code})`
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
  } else if (code === Code.ACCOUNT_DISABLED) {
    return `${Message.ACCOUNT_DISABLED_MESSAGE} (${code})`
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
  } else if (code === Code.INVALID_LAST_NAME_LENGTH) {
    return `${Message.INVALID_LAST_NAME_LENGTH_MESSAGE} (${code})`
  } else if (code === Code.ILLEGAL_CHAR_IN_LAST_NAME) {
    return `${Message.ILLEGAL_CHAR_IN_LAST_NAME_MESSAGE} (${code})`
  } else if (code === Code.INVALID_FIRST_NAME_LENGTH) {
    return `${Message.INVALID_FIRST_NAME_LENGTH_MESSAGE} (${code})`
  } else if (code === Code.ILLEGAL_CHAR_IN_FIRST_NAME) {
    return `${Message.ILLEGAL_CHAR_IN_FIRST_NAME_MESSAGE} (${code})`
  } else if (code === Code.INVALID_LAST_NAME_FURIGANA_LENGTH) {
    return `${Message.INVALID_LAST_NAME_FURIGANA_LENGTH_MESSAGE} (${code})`
  } else if (code === Code.ILLEGAL_CHAR_IN_LAST_NAME_FURIGANA) {
    return `${Message.ILLEGAL_CHAR_IN_LAST_NAME_FURIGANA_MESSAGE} (${code})`
  } else if (code === Code.INVALID_FIRST_NAME_FURIGANA_LENGTH) {
    return `${Message.INVALID_FIRST_NAME_FURIGANA_LENGTH_MESSAGE} (${code})`
  } else if (code === Code.ILLEGAL_CHAR_IN_FIRST_NAME_FURIGANA) {
    return `${Message.ILLEGAL_CHAR_IN_FIRST_NAME_FURIGANA_MESSAGE} (${code})`
  } else if (code === Code.ILLEGAL_DATE) {
    return `${Message.ILLEGAL_DATE_MESSAGE} (${code})`
  } else if (code === Code.ILLEGAL_AGE) {
    return `${Message.ILLEGAL_AGE_MESSAGE} (${code})`
  } else if (code === Code.INVALID_PREFECTURE) {
    return `${Message.INVALID_PREFECTURE_MESSAGE} (${code})`
  } else if (code === Code.INVALID_CITY_LENGTH) {
    return `${Message.INVALID_CITY_LENGTH_MESSAGE} (${code})`
  } else if (code === Code.ILLEGAL_CHAR_IN_CITY) {
    return `${Message.ILLEGAL_CHAR_IN_CITY_MESSAGE} (${code})`
  } else if (code === Code.INVALID_ADDRESS_LINE1_LENGTH) {
    return `${Message.INVALID_ADDRESS_LINE1_LENGTH_MESSAGE} (${code})`
  } else if (code === Code.ILLEGAL_CHAR_IN_ADDRESS_LINE1) {
    return `${Message.ILLEGAL_CHAR_IN_ADDRESS_LINE1_MESSAGE} (${code})`
  } else if (code === Code.INVALID_ADDRESS_LINE2_LENGTH) {
    return `${Message.INVALID_ADDRESS_LINE2_LENGTH_MESSAGE} (${code})`
  } else if (code === Code.ILLEGAL_CHAR_IN_ADDRESS_LINE2) {
    return `${Message.ILLEGAL_CHAR_IN_ADDRESS_LINE2_MESSAGE} (${code})`
  } else if (code === Code.INVALID_TEL_NUM_FORMAT) {
    return `${Message.INVALID_TEL_NUM_FORMAT_MESSAGE} (${code})`
  } else if (code === Code.NO_NAME_FOUND) {
    return `${Message.NO_NAME_FOUND_MESSAGE} (${code})`
  } else if (code === Code.NO_FILE_NAME_FOUND) {
    return `${Message.NO_FILE_NAME_FOUND_MESSAGE} (${code})`
  } else if (code === Code.DATA_PARSE_FAILURE) {
    return `${Message.DATA_PARSE_FAILURE_MESSAGE} (${code})`
  } else if (code === Code.INVALID_NAME_IN_FIELD) {
    return `${Message.INVALID_NAME_IN_FIELD_MESSAGE} (${code})`
  } else if (code === Code.INVALID_UTF8_SEQUENCE) {
    return `${Message.INVALID_UTF8_SEQUENCE_MESSAGE} (${code})`
  } else if (code === Code.INVALID_IDENTITY_JSON) {
    return `${Message.INVALID_IDENTITY_JSON_MESSAGE} (${code})`
  } else if (code === Code.NO_JPEG_EXTENSION) {
    return `${Message.NO_JPEG_EXTENSION_MESSAGE} (${code})`
  } else if (code === Code.EXCEED_MAX_IDENTITY_IMAGE_SIZE_LIMIT) {
    return `${Message.EXCEED_MAX_IDENTITY_IMAGE_SIZE_LIMIT_MESSAGE} (${code})`
  } else if (code === Code.INVALID_JPEG_IMAGE) {
    return `${Message.INVALID_JPEG_IMAGE_MESSAGE} (${code})`
  } else if (code === Code.NO_IDENTITY_FOUND) {
    return `${Message.NO_IDENTITY_FOUND_MESSAGE} (${code})`
  } else if (code === Code.NO_IDENTITY_IMAGE1_FOUND) {
    return `${Message.NO_IDENTITY_IMAGE1_FOUND_MESSAGE} (${code})`
  } else if (code === Code.IDENTITY_INFO_REQ_ALREADY_EXISTS) {
    return `${Message.IDENTITY_INFO_REQ_ALREADY_EXISTS_MESSAGE} (${code})`
  } else if (code === Code.DATE_OF_BIRTH_IS_NOT_MATCH) {
    return `${Message.DATE_OF_BIRTH_IS_NOT_MATCH_MESSAGE} (${code})`
  } else if (code === Code.NO_IDENTITY_UPDATED) {
    return `${Message.NO_IDENTITY_UPDATED_MESSAGE} (${code})`
  } else if (code === Code.FIRST_NAME_IS_NOT_MATCH) {
    return `${Message.FIRST_NAME_IS_NOT_MATCH_MESSAGE} (${code})`
  } else if (code === Code.INVALID_MULTIPART_FORM_DATA) {
    return `${Message.INVALID_MULTIPART_FORM_DATA_MESSAGE} (${code})`
  } else {
    throw new Error(`unexpected code: ${code}`)
  }
}
