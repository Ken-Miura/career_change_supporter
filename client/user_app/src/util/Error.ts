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
  export const NO_CAREER_TO_HANDLE_FOUND = 20073
  export const ILLEGAL_FEE_PER_HOUR_IN_YEN = 20074
  export const INVALID_BANK_CODE_FORMAT = 20075
  export const INVALID_BRANCH_CODE_FORMAT = 20076
  export const INVALID_ACCOUNT_TYPE = 20077
  export const INVALID_ACCOUNT_NUMBER_FORMAT = 20078
  export const INVALID_ACCOUNT_HOLDER_NAME_LENGTH = 20079
  export const ILLEGAL_CHAR_IN_ACCOUNT_HOLDER_NAME = 20080
  export const ACCOUNT_HOLDER_NAME_DOES_NOT_MATCH_FULL_NAME = 20081
  export const INVALID_BANK = 20082
  export const INVALID_BANK_BRANCH = 20083
  export const INVALID_BANK_ACCOUNT_NUMBER = 20084
  export const ILLEGAL_YEARS_OF_SERVICE = 20085
  export const EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_ANNUAL_INCOME_IN_MAN_YEN = 20086
  export const EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_FEE_PER_HOUR_IN_YEN = 20087
  export const INVALID_SORT_KEY = 20088
  export const INVALID_SORT_ORDER = 20089
  export const INVALID_CONSULTANT_SEARCH_PARAM_FROM = 20090
  export const INVALID_CONSULTANT_SEARCH_PARAM_SIZE = 20091
  export const NON_POSITIVE_CONSULTANT_ID = 20092
  export const CONSULTANT_DOES_NOT_EXIST = 20093
  export const EQUAL_OR_MORE_IS_LESS_THAN_OR_MORE_YEARS_OF_SERVICE = 20094
  export const NO_CAREERS_FOUND = 20095
  export const NO_FEE_PER_HOUR_IN_YEN_FOUND = 20096
  export const FEE_PER_HOUR_IN_YEN_WAS_UPDATED = 20097
  export const CONSULTANT_IS_NOT_AVAILABLE = 20098
  export const PROFIT_OBJECTIVE_USE_IS_NOT_ALLOWED = 20099
  export const ILLEGAL_CONSULTATION_DATE_TIME = 20100
  export const ILLEGAL_CONSULTATION_HOUR = 20101
  export const INVALID_CONSULTATION_DATE_TIME = 20102
  export const DUPLICATE_DATE_TIME_CANDIDATES = 20103
  export const THREE_D_SECURE_ERROR = 20104
  export const EXCEED_MAX_ANNUAL_REWARDS = 20105
  export const CARD_AUTH_PAYMENT_ERROR = 20106
  export const PAY_JP_CODE_INCORRECT_CARD_DATA = 20107
  export const PAY_JP_CODE_CARD_DECLINED = 20108
  export const PAY_JP_CODE_CARD_FLAGGED = 20109
  export const PAY_JP_CODE_UNACCEPTABLE_BRAND = 20110
  export const PAY_JP_CODE_THREE_D_SECURE_INCOMPLETED = 20111
  export const PAY_JP_CODE_THREE_D_SECURE_FAILED = 20112
  export const PAY_JP_CODE_NOT_IN_THREE_D_SECURE_FLOW = 20113
  export const NON_POSITIVE_CONSULTATION_REQ_ID = 20114
  export const NO_CONSULTATION_REQ_FOUND = 20115
  export const INVALID_CANDIDATE = 20116
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
codeToMessage.set(Code.UNEXPECTED_ERR_USER.toString(), `${Message.UNEXPECTED_ERR} (${Code.UNEXPECTED_ERR_USER})`)
codeToMessage.set(Code.INVALID_EMAIL_ADDRESS_FORMAT.toString(), `${Message.INVALID_EMAIL_ADDRESS_FORMAT_MESSAGE} (${Code.INVALID_EMAIL_ADDRESS_FORMAT})`)
codeToMessage.set(Code.INVALID_PASSWORD_FORMAT.toString(), `${Message.INVALID_PASSWORD_FORMAT_MESSAGE} (${Code.INVALID_PASSWORD_FORMAT})`)
codeToMessage.set(Code.INVALID_UUID_FORMAT.toString(), `${Message.INVALID_UUID_FORMAT_MESSAGE} (${Code.INVALID_UUID_FORMAT})`)
codeToMessage.set(Code.ACCOUNT_ALREADY_EXISTS.toString(), `${Message.ACCOUNT_ALREADY_EXISTS_MESSAGE} (${Code.ACCOUNT_ALREADY_EXISTS})`)
codeToMessage.set(Code.REACH_TEMP_ACCOUNTS_LIMIT.toString(), `${Message.REACH_TEMP_ACCOUNTS_LIMIT_MESSAGE} (${Code.REACH_TEMP_ACCOUNTS_LIMIT})`)
codeToMessage.set(Code.TEMP_ACCOUNT_EXPIRED.toString(), `${Message.TEMP_ACCOUNT_EXPIRED_MESSAGE} (${Code.TEMP_ACCOUNT_EXPIRED})`)
codeToMessage.set(Code.NO_TEMP_ACCOUNT_FOUND.toString(), `${Message.NO_TEMP_ACCOUNT_FOUND_MESSAGE} (${Code.NO_TEMP_ACCOUNT_FOUND})`)
codeToMessage.set(Code.EMAIL_OR_PWD_INCORRECT.toString(), `${Message.EMAIL_OR_PWD_INCORRECT_MESSAGE} (${Code.EMAIL_OR_PWD_INCORRECT})`)
codeToMessage.set(Code.UNAUTHORIZED.toString(), `${Message.UNAUTHORIZED_MESSAGE} (${Code.UNAUTHORIZED})`)
codeToMessage.set(Code.ACCOUNT_DISABLED.toString(), `${Message.ACCOUNT_DISABLED_MESSAGE} (${Code.ACCOUNT_DISABLED})`)
codeToMessage.set(Code.REACH_PASSWORD_CHANGE_REQ_LIMIT.toString(), `${Message.REACH_PASSWORD_CHANGE_REQ_LIMIT_MESSAGE} (${Code.REACH_PASSWORD_CHANGE_REQ_LIMIT})`)
codeToMessage.set(Code.PWD_CHANGE_REQ_EXPIRED.toString(), `${Message.PWD_CHANGE_REQ_EXPIRED_MESSAGE} (${Code.PWD_CHANGE_REQ_EXPIRED})`)
codeToMessage.set(Code.NO_ACCOUNT_FOUND.toString(), `${Message.NO_ACCOUNT_FOUND_MESSAGE} (${Code.NO_ACCOUNT_FOUND})`)
codeToMessage.set(Code.NO_PWD_CHANGE_REQ_FOUND.toString(), `${Message.NO_PWD_CHANGE_REQ_FOUND_MESSAGE} (${Code.NO_PWD_CHANGE_REQ_FOUND})`)
codeToMessage.set(Code.REACH_PAYMENT_PLATFORM_RATE_LIMIT.toString(), `${Message.REACH_PAYMENT_PLATFORM_RATE_LIMIT_MESSAGE} (${Code.REACH_PAYMENT_PLATFORM_RATE_LIMIT})`)
codeToMessage.set(Code.INVALID_LAST_NAME_LENGTH.toString(), `${Message.INVALID_LAST_NAME_LENGTH_MESSAGE} (${Code.INVALID_LAST_NAME_LENGTH})`)
codeToMessage.set(Code.ILLEGAL_CHAR_IN_LAST_NAME.toString(), `${Message.ILLEGAL_CHAR_IN_LAST_NAME_MESSAGE} (${Code.ILLEGAL_CHAR_IN_LAST_NAME})`)
codeToMessage.set(Code.INVALID_FIRST_NAME_LENGTH.toString(), `${Message.INVALID_FIRST_NAME_LENGTH_MESSAGE} (${Code.INVALID_FIRST_NAME_LENGTH})`)
codeToMessage.set(Code.ILLEGAL_CHAR_IN_FIRST_NAME.toString(), `${Message.ILLEGAL_CHAR_IN_FIRST_NAME_MESSAGE} (${Code.ILLEGAL_CHAR_IN_FIRST_NAME})`)
codeToMessage.set(Code.INVALID_LAST_NAME_FURIGANA_LENGTH.toString(), `${Message.INVALID_LAST_NAME_FURIGANA_LENGTH_MESSAGE} (${Code.INVALID_LAST_NAME_FURIGANA_LENGTH})`)
codeToMessage.set(Code.ILLEGAL_CHAR_IN_LAST_NAME_FURIGANA.toString(), `${Message.ILLEGAL_CHAR_IN_LAST_NAME_FURIGANA_MESSAGE} (${Code.ILLEGAL_CHAR_IN_LAST_NAME_FURIGANA})`)
codeToMessage.set(Code.INVALID_FIRST_NAME_FURIGANA_LENGTH.toString(), `${Message.INVALID_FIRST_NAME_FURIGANA_LENGTH_MESSAGE} (${Code.INVALID_FIRST_NAME_FURIGANA_LENGTH})`)
codeToMessage.set(Code.ILLEGAL_CHAR_IN_FIRST_NAME_FURIGANA.toString(), `${Message.ILLEGAL_CHAR_IN_FIRST_NAME_FURIGANA_MESSAGE} (${Code.ILLEGAL_CHAR_IN_FIRST_NAME_FURIGANA})`)
codeToMessage.set(Code.ILLEGAL_DATE.toString(), `${Message.ILLEGAL_DATE_MESSAGE} (${Code.ILLEGAL_DATE})`)
codeToMessage.set(Code.ILLEGAL_AGE.toString(), `${Message.ILLEGAL_AGE_MESSAGE} (${Code.ILLEGAL_AGE})`)
codeToMessage.set(Code.INVALID_PREFECTURE.toString(), `${Message.INVALID_PREFECTURE_MESSAGE} (${Code.INVALID_PREFECTURE})`)
codeToMessage.set(Code.INVALID_CITY_LENGTH.toString(), `${Message.INVALID_CITY_LENGTH_MESSAGE} (${Code.INVALID_CITY_LENGTH})`)
codeToMessage.set(Code.ILLEGAL_CHAR_IN_CITY.toString(), `${Message.ILLEGAL_CHAR_IN_CITY_MESSAGE} (${Code.ILLEGAL_CHAR_IN_CITY})`)
codeToMessage.set(Code.INVALID_ADDRESS_LINE1_LENGTH.toString(), `${Message.INVALID_ADDRESS_LINE1_LENGTH_MESSAGE} (${Code.INVALID_ADDRESS_LINE1_LENGTH})`)
codeToMessage.set(Code.ILLEGAL_CHAR_IN_ADDRESS_LINE1.toString(), `${Message.ILLEGAL_CHAR_IN_ADDRESS_LINE1_MESSAGE} (${Code.ILLEGAL_CHAR_IN_ADDRESS_LINE1})`)
codeToMessage.set(Code.INVALID_ADDRESS_LINE2_LENGTH.toString(), `${Message.INVALID_ADDRESS_LINE2_LENGTH_MESSAGE} (${Code.INVALID_ADDRESS_LINE2_LENGTH})`)
codeToMessage.set(Code.ILLEGAL_CHAR_IN_ADDRESS_LINE2.toString(), `${Message.ILLEGAL_CHAR_IN_ADDRESS_LINE2_MESSAGE} (${Code.ILLEGAL_CHAR_IN_ADDRESS_LINE2})`)
codeToMessage.set(Code.INVALID_TEL_NUM_FORMAT.toString(), `${Message.INVALID_TEL_NUM_FORMAT_MESSAGE} (${Code.INVALID_TEL_NUM_FORMAT})`)
codeToMessage.set(Code.NO_NAME_FOUND.toString(), `${Message.NO_NAME_FOUND_MESSAGE} (${Code.NO_NAME_FOUND})`)
codeToMessage.set(Code.NO_FILE_NAME_FOUND.toString(), `${Message.NO_FILE_NAME_FOUND_MESSAGE} (${Code.NO_FILE_NAME_FOUND})`)
codeToMessage.set(Code.DATA_PARSE_FAILURE.toString(), `${Message.DATA_PARSE_FAILURE_MESSAGE} (${Code.DATA_PARSE_FAILURE})`)
codeToMessage.set(Code.INVALID_NAME_IN_FIELD.toString(), `${Message.INVALID_NAME_IN_FIELD_MESSAGE} (${Code.INVALID_NAME_IN_FIELD})`)
codeToMessage.set(Code.INVALID_UTF8_SEQUENCE.toString(), `${Message.INVALID_UTF8_SEQUENCE_MESSAGE} (${Code.INVALID_UTF8_SEQUENCE})`)
codeToMessage.set(Code.INVALID_IDENTITY_JSON.toString(), `${Message.INVALID_IDENTITY_JSON_MESSAGE} (${Code.INVALID_IDENTITY_JSON})`)
codeToMessage.set(Code.NO_JPEG_EXTENSION.toString(), `${Message.NO_JPEG_EXTENSION_MESSAGE} (${Code.NO_JPEG_EXTENSION})`)
codeToMessage.set(Code.EXCEED_MAX_IDENTITY_IMAGE_SIZE_LIMIT.toString(), `${Message.EXCEED_MAX_IDENTITY_IMAGE_SIZE_LIMIT_MESSAGE} (${Code.EXCEED_MAX_IDENTITY_IMAGE_SIZE_LIMIT})`)
codeToMessage.set(Code.INVALID_JPEG_IMAGE.toString(), `${Message.INVALID_JPEG_IMAGE_MESSAGE} (${Code.INVALID_JPEG_IMAGE})`)
codeToMessage.set(Code.NO_IDENTITY_FOUND.toString(), `${Message.NO_IDENTITY_FOUND_MESSAGE} (${Code.NO_IDENTITY_FOUND})`)
codeToMessage.set(Code.NO_IDENTITY_IMAGE1_FOUND.toString(), `${Message.NO_IDENTITY_IMAGE1_FOUND_MESSAGE} (${Code.NO_IDENTITY_IMAGE1_FOUND})`)
codeToMessage.set(Code.IDENTITY_INFO_REQ_ALREADY_EXISTS.toString(), `${Message.IDENTITY_INFO_REQ_ALREADY_EXISTS_MESSAGE} (${Code.IDENTITY_INFO_REQ_ALREADY_EXISTS})`)
codeToMessage.set(Code.DATE_OF_BIRTH_IS_NOT_MATCH.toString(), `${Message.DATE_OF_BIRTH_IS_NOT_MATCH_MESSAGE} (${Code.DATE_OF_BIRTH_IS_NOT_MATCH})`)
codeToMessage.set(Code.NO_IDENTITY_UPDATED.toString(), `${Message.NO_IDENTITY_UPDATED_MESSAGE} (${Code.NO_IDENTITY_UPDATED})`)
codeToMessage.set(Code.FIRST_NAME_IS_NOT_MATCH.toString(), `${Message.FIRST_NAME_IS_NOT_MATCH_MESSAGE} (${Code.FIRST_NAME_IS_NOT_MATCH})`)
codeToMessage.set(Code.INVALID_MULTIPART_FORM_DATA.toString(), `${Message.INVALID_MULTIPART_FORM_DATA_MESSAGE} (${Code.INVALID_MULTIPART_FORM_DATA})`)
codeToMessage.set(Code.INVALID_CAREER_JSON.toString(), `${Message.INVALID_CAREER_JSON_MESSAGE} (${Code.INVALID_CAREER_JSON})`)
codeToMessage.set(Code.INVALID_COMPANY_NAME_LENGTH.toString(), `${Message.INVALID_COMPANY_NAME_LENGTH_MESSAGE} (${Code.INVALID_COMPANY_NAME_LENGTH})`)
codeToMessage.set(Code.ILLEGAL_CHAR_IN_COMPANY_NAME.toString(), `${Message.ILLEGAL_CHAR_IN_COMPANY_NAME_MESSAGE} (${Code.ILLEGAL_CHAR_IN_COMPANY_NAME})`)
codeToMessage.set(Code.INVALID_DEPARTMENT_NAME_LENGTH.toString(), `${Message.INVALID_DEPARTMENT_NAME_LENGTH_MESSAGE} (${Code.INVALID_DEPARTMENT_NAME_LENGTH})`)
codeToMessage.set(Code.ILLEGAL_CHAR_IN_DEPARTMENT_NAME.toString(), `${Message.ILLEGAL_CHAR_IN_DEPARTMENT_NAME_MESSAGE} (${Code.ILLEGAL_CHAR_IN_DEPARTMENT_NAME})`)
codeToMessage.set(Code.INVALID_OFFICE_LENGTH.toString(), `${Message.INVALID_OFFICE_LENGTH_MESSAGE} (${Code.INVALID_OFFICE_LENGTH})`)
codeToMessage.set(Code.ILLEGAL_CHAR_IN_OFFICE.toString(), `${Message.ILLEGAL_CHAR_IN_OFFICE_MESSAGE} (${Code.ILLEGAL_CHAR_IN_OFFICE})`)
codeToMessage.set(Code.ILLEGAL_CAREER_START_DATE.toString(), `${Message.ILLEGAL_CAREER_START_DATE_MESSAGE} (${Code.ILLEGAL_CAREER_START_DATE})`)
codeToMessage.set(Code.ILLEGAL_CAREER_END_DATE.toString(), `${Message.ILLEGAL_CAREER_END_DATE_MESSAGE} (${Code.ILLEGAL_CAREER_END_DATE})`)
codeToMessage.set(Code.CAREER_START_DATE_EXCEEDS_CAREER_END_DATE.toString(), `${Message.CAREER_START_DATE_EXCEEDS_CAREER_END_DATE_MESSAGE} (${Code.CAREER_START_DATE_EXCEEDS_CAREER_END_DATE})`)
codeToMessage.set(Code.ILLEGAL_CONTRACT_TYPE.toString(), `${Message.ILLEGAL_CONTRACT_TYPE_MESSAGE} (${Code.ILLEGAL_CONTRACT_TYPE})`)
codeToMessage.set(Code.INVALID_PROFESSION_LENGTH.toString(), `${Message.INVALID_PROFESSION_LENGTH_MESSAGE} (${Code.INVALID_PROFESSION_LENGTH})`)
codeToMessage.set(Code.ILLEGAL_CHAR_IN_PROFESSION.toString(), `${Message.ILLEGAL_CHAR_IN_PROFESSION_MESSAGE} (${Code.ILLEGAL_CHAR_IN_PROFESSION})`)
codeToMessage.set(Code.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN.toString(), `${Message.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN_MESSAGE} (${Code.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN})`)
codeToMessage.set(Code.INVALID_POSITION_NAME_LENGTH.toString(), `${Message.INVALID_POSITION_NAME_LENGTH_MESSAGE} (${Code.INVALID_POSITION_NAME_LENGTH})`)
codeToMessage.set(Code.ILLEGAL_CHAR_IN_POSITION_NAME.toString(), `${Message.ILLEGAL_CHAR_IN_POSITION_NAME_MESSAGE} (${Code.ILLEGAL_CHAR_IN_POSITION_NAME})`)
codeToMessage.set(Code.INVALID_NOTE_LENGTH.toString(), `${Message.INVALID_NOTE_LENGTH_MESSAGE} (${Code.INVALID_NOTE_LENGTH})`)
codeToMessage.set(Code.ILLEGAL_CHAR_IN_NOTE.toString(), `${Message.ILLEGAL_CHAR_IN_NOTE_MESSAGE} (${Code.ILLEGAL_CHAR_IN_NOTE})`)
codeToMessage.set(Code.NO_CAREER_FOUND.toString(), `${Message.NO_CAREER_FOUND_MESSAGE} (${Code.NO_CAREER_FOUND})`)
codeToMessage.set(Code.NO_CAREER_IMAGE1_FOUND.toString(), `${Message.NO_CAREER_IMAGE1_FOUND_MESSAGE} (${Code.NO_CAREER_IMAGE1_FOUND})`)
codeToMessage.set(Code.EXCEED_MAX_CAREER_IMAGE_SIZE_LIMIT.toString(), `${Message.EXCEED_MAX_CAREER_IMAGE_SIZE_LIMIT_MESSAGE} (${Code.EXCEED_MAX_CAREER_IMAGE_SIZE_LIMIT})`)
codeToMessage.set(Code.REACH_CAREER_NUM_LIMIT.toString(), `${Message.REACH_CAREER_NUM_LIMIT_MESSAGE} (${Code.REACH_CAREER_NUM_LIMIT})`)
codeToMessage.set(Code.NO_IDENTITY_REGISTERED.toString(), `${Message.NO_IDENTITY_REGISTERED_MESSAGE} (${Code.NO_IDENTITY_REGISTERED})`)
codeToMessage.set(Code.REACH_CREATE_CAREER_REQ_NUM_LIMIT.toString(), `${Message.REACH_CREATE_CAREER_REQ_NUM_LIMIT_MESSAGE} (${Code.REACH_CREATE_CAREER_REQ_NUM_LIMIT})`)
codeToMessage.set(Code.NO_CAREER_TO_HANDLE_FOUND.toString(), `${Message.NO_CAREER_TO_HANDLE_FOUND_MESSAGE} (${Code.NO_CAREER_TO_HANDLE_FOUND})`)
codeToMessage.set(Code.ILLEGAL_FEE_PER_HOUR_IN_YEN.toString(), `${Message.ILLEGAL_FEE_PER_HOUR_IN_YEN_MESSAGE} (${Code.ILLEGAL_FEE_PER_HOUR_IN_YEN})`)
codeToMessage.set(Code.INVALID_BANK_CODE_FORMAT.toString(), `${Message.INVALID_BANK_CODE_FORMAT_MESSAGE} (${Code.INVALID_BANK_CODE_FORMAT})`)
codeToMessage.set(Code.INVALID_BRANCH_CODE_FORMAT.toString(), `${Message.INVALID_BRANCH_CODE_FORMAT_MESSAGE} (${Code.INVALID_BRANCH_CODE_FORMAT})`)
codeToMessage.set(Code.INVALID_ACCOUNT_TYPE.toString(), `${Message.INVALID_ACCOUNT_TYPE_MESSAGE} (${Code.INVALID_ACCOUNT_TYPE})`)
codeToMessage.set(Code.INVALID_ACCOUNT_NUMBER_FORMAT.toString(), `${Message.INVALID_ACCOUNT_NUMBER_FORMAT_MESSAGE} (${Code.INVALID_ACCOUNT_NUMBER_FORMAT})`)
codeToMessage.set(Code.INVALID_ACCOUNT_HOLDER_NAME_LENGTH.toString(), `${Message.INVALID_ACCOUNT_HOLDER_NAME_LENGTH_MESSAGE} (${Code.INVALID_ACCOUNT_HOLDER_NAME_LENGTH})`)
codeToMessage.set(Code.ILLEGAL_CHAR_IN_ACCOUNT_HOLDER_NAME.toString(), `${Message.ILLEGAL_CHAR_IN_ACCOUNT_HOLDER_NAME_MESSAGE} (${Code.ILLEGAL_CHAR_IN_ACCOUNT_HOLDER_NAME})`)
codeToMessage.set(Code.ACCOUNT_HOLDER_NAME_DOES_NOT_MATCH_FULL_NAME.toString(), `${Message.ACCOUNT_HOLDER_NAME_DOES_NOT_MATCH_FULL_NAME_MESSAGE} (${Code.ACCOUNT_HOLDER_NAME_DOES_NOT_MATCH_FULL_NAME})`)
codeToMessage.set(Code.INVALID_BANK.toString(), `${Message.INVALID_BANK_MESSAGE} (${Code.INVALID_BANK})`)
codeToMessage.set(Code.INVALID_BANK_BRANCH.toString(), `${Message.INVALID_BANK_BRANCH_MESSAGE} (${Code.INVALID_BANK_BRANCH})`)
codeToMessage.set(Code.INVALID_BANK_ACCOUNT_NUMBER.toString(), `${Message.INVALID_BANK_ACCOUNT_NUMBER_MESSAGE} (${Code.INVALID_BANK_ACCOUNT_NUMBER})`)
codeToMessage.set(Code.ILLEGAL_YEARS_OF_SERVICE.toString(), `${Message.ILLEGAL_YEARS_OF_SERVICE_MESSAGE} (${Code.ILLEGAL_YEARS_OF_SERVICE})`)
codeToMessage.set(Code.EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_ANNUAL_INCOME_IN_MAN_YEN.toString(), `${Message.EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_ANNUAL_INCOME_IN_MAN_YEN_MESSAGE} (${Code.EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_ANNUAL_INCOME_IN_MAN_YEN})`)
codeToMessage.set(Code.EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_FEE_PER_HOUR_IN_YEN.toString(), `${Message.EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_FEE_PER_HOUR_IN_YEN_MESSAGE} (${Code.EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_FEE_PER_HOUR_IN_YEN})`)
codeToMessage.set(Code.INVALID_SORT_KEY.toString(), `${Message.INVALID_SORT_KEY_MESSAGE} (${Code.INVALID_SORT_KEY})`)
codeToMessage.set(Code.INVALID_SORT_ORDER.toString(), `${Message.INVALID_SORT_ORDER_MESSAGE} (${Code.INVALID_SORT_ORDER})`)
codeToMessage.set(Code.INVALID_CONSULTANT_SEARCH_PARAM_FROM.toString(), `${Message.INVALID_CONSULTANT_SEARCH_PARAM_FROM_MESSAGE} (${Code.INVALID_CONSULTANT_SEARCH_PARAM_FROM})`)
codeToMessage.set(Code.INVALID_CONSULTANT_SEARCH_PARAM_SIZE.toString(), `${Message.INVALID_CONSULTANT_SEARCH_PARAM_SIZE_MESSAGE} (${Code.INVALID_CONSULTANT_SEARCH_PARAM_SIZE})`)
codeToMessage.set(Code.NON_POSITIVE_CONSULTANT_ID.toString(), `${Message.NON_POSITIVE_CONSULTANT_ID_MESSAGE} (${Code.NON_POSITIVE_CONSULTANT_ID})`)
codeToMessage.set(Code.CONSULTANT_DOES_NOT_EXIST.toString(), `${Message.CONSULTANT_DOES_NOT_EXIST_MESSAGE} (${Code.CONSULTANT_DOES_NOT_EXIST})`)
codeToMessage.set(Code.EQUAL_OR_MORE_IS_LESS_THAN_OR_MORE_YEARS_OF_SERVICE.toString(), `${Message.EQUAL_OR_MORE_IS_LESS_THAN_OR_MORE_YEARS_OF_SERVICE_MESSAGE} (${Code.EQUAL_OR_MORE_IS_LESS_THAN_OR_MORE_YEARS_OF_SERVICE})`)
codeToMessage.set(Code.NO_CAREERS_FOUND.toString(), `${Message.NO_CAREERS_FOUND_MESSAGE} (${Code.NO_CAREERS_FOUND})`)
codeToMessage.set(Code.NO_FEE_PER_HOUR_IN_YEN_FOUND.toString(), `${Message.NO_FEE_PER_HOUR_IN_YEN_FOUND_MESSAGE} (${Code.NO_FEE_PER_HOUR_IN_YEN_FOUND})`)
codeToMessage.set(Code.FEE_PER_HOUR_IN_YEN_WAS_UPDATED.toString(), `${Message.FEE_PER_HOUR_IN_YEN_WAS_UPDATED_MESSAGE} (${Code.FEE_PER_HOUR_IN_YEN_WAS_UPDATED})`)
codeToMessage.set(Code.CONSULTANT_IS_NOT_AVAILABLE.toString(), `${Message.CONSULTANT_IS_NOT_AVAILABLE_MESSAGE} (${Code.CONSULTANT_IS_NOT_AVAILABLE})`)
codeToMessage.set(Code.PROFIT_OBJECTIVE_USE_IS_NOT_ALLOWED.toString(), `${Message.PROFIT_OBJECTIVE_USE_IS_NOT_ALLOWED_MESSAGE} (${Code.PROFIT_OBJECTIVE_USE_IS_NOT_ALLOWED})`)
codeToMessage.set(Code.ILLEGAL_CONSULTATION_DATE_TIME.toString(), `${Message.ILLEGAL_CONSULTATION_DATE_TIME_MESSAGE} (${Code.ILLEGAL_CONSULTATION_DATE_TIME})`)
codeToMessage.set(Code.ILLEGAL_CONSULTATION_HOUR.toString(), `${Message.ILLEGAL_CONSULTATION_HOUR_MESSAGE} (${Code.ILLEGAL_CONSULTATION_HOUR})`)
codeToMessage.set(Code.INVALID_CONSULTATION_DATE_TIME.toString(), `${Message.INVALID_CONSULTATION_DATE_TIME_MESSAGE} (${Code.INVALID_CONSULTATION_DATE_TIME})`)
codeToMessage.set(Code.DUPLICATE_DATE_TIME_CANDIDATES.toString(), `${Message.DUPLICATE_DATE_TIME_CANDIDATES_MESSAGE} (${Code.DUPLICATE_DATE_TIME_CANDIDATES})`)
codeToMessage.set(Code.THREE_D_SECURE_ERROR.toString(), `${Message.THREE_D_SECURE_ERROR_MESSAGE} (${Code.THREE_D_SECURE_ERROR})`)
codeToMessage.set(Code.EXCEED_MAX_ANNUAL_REWARDS.toString(), `${Message.EXCEED_MAX_ANNUAL_REWARDS_MESSAGE} (${Code.EXCEED_MAX_ANNUAL_REWARDS})`)
codeToMessage.set(Code.CARD_AUTH_PAYMENT_ERROR.toString(), `${Message.CARD_AUTH_PAYMENT_ERROR_MESSAGE} (${Code.CARD_AUTH_PAYMENT_ERROR})`)
codeToMessage.set(Code.PAY_JP_CODE_INCORRECT_CARD_DATA.toString(), `${Message.PAY_JP_CODE_INCORRECT_CARD_DATA_MESSAGE} (${Code.PAY_JP_CODE_INCORRECT_CARD_DATA})`)
codeToMessage.set(Code.PAY_JP_CODE_CARD_DECLINED.toString(), `${Message.PAY_JP_CODE_CARD_DECLINED_MESSAGE} (${Code.PAY_JP_CODE_CARD_DECLINED})`)
codeToMessage.set(Code.PAY_JP_CODE_CARD_FLAGGED.toString(), `${Message.PAY_JP_CODE_CARD_FLAGGED_MESSAGE} (${Code.PAY_JP_CODE_CARD_FLAGGED})`)
codeToMessage.set(Code.PAY_JP_CODE_UNACCEPTABLE_BRAND.toString(), `${Message.PAY_JP_CODE_UNACCEPTABLE_BRAND_MESSAGE} (${Code.PAY_JP_CODE_UNACCEPTABLE_BRAND})`)
codeToMessage.set(Code.PAY_JP_CODE_THREE_D_SECURE_INCOMPLETED.toString(), `${Message.PAY_JP_CODE_THREE_D_SECURE_INCOMPLETED_MESSAGE} (${Code.PAY_JP_CODE_THREE_D_SECURE_INCOMPLETED})`)
codeToMessage.set(Code.PAY_JP_CODE_THREE_D_SECURE_FAILED.toString(), `${Message.PAY_JP_CODE_THREE_D_SECURE_FAILED_MESSAGE} (${Code.PAY_JP_CODE_THREE_D_SECURE_FAILED})`)
codeToMessage.set(Code.PAY_JP_CODE_NOT_IN_THREE_D_SECURE_FLOW.toString(), `${Message.PAY_JP_CODE_NOT_IN_THREE_D_SECURE_FLOW_MESSAGE} (${Code.PAY_JP_CODE_NOT_IN_THREE_D_SECURE_FLOW})`)
codeToMessage.set(Code.NON_POSITIVE_CONSULTATION_REQ_ID.toString(), `${Message.NON_POSITIVE_CONSULTATION_REQ_ID_MESSAGE} (${Code.NON_POSITIVE_CONSULTATION_REQ_ID})`)
codeToMessage.set(Code.NO_CONSULTATION_REQ_FOUND.toString(), `${Message.NO_CONSULTATION_REQ_FOUND_MESSAGE} (${Code.NO_CONSULTATION_REQ_FOUND})`)
codeToMessage.set(Code.INVALID_CANDIDATE.toString(), `${Message.INVALID_CANDIDATE_MESSAGE} (${Code.INVALID_CANDIDATE})`)
