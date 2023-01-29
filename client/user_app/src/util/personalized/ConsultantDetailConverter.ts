import { FIFTEEN_YEARS_OR_MORE_LESS_THAN_TWENTY_YEARS, FIVE_YEARS_OR_MORE_LESS_THAN_TEN_YEARS, LESS_THAN_THREE_YEARS, TEN_YEARS_OR_MORE_LESS_THAN_FIFTEEN_YEARS, THREE_YEARS_OR_MORE_LESS_THAN_FIVE_YEARS, TWENTY_YEARS_OR_MORE } from './consultant-detail/YearsOfService'

export const convertYearsOfServiceValue = (yearsOfService: string): string => {
  if (yearsOfService === LESS_THAN_THREE_YEARS) {
    return '3年未満'
  } else if (yearsOfService === THREE_YEARS_OR_MORE_LESS_THAN_FIVE_YEARS) {
    return '3年以上5年未満'
  } else if (yearsOfService === FIVE_YEARS_OR_MORE_LESS_THAN_TEN_YEARS) {
    return '5年以上10年未満'
  } else if (yearsOfService === TEN_YEARS_OR_MORE_LESS_THAN_FIFTEEN_YEARS) {
    return '10年以上15年未満'
  } else if (yearsOfService === FIFTEEN_YEARS_OR_MORE_LESS_THAN_TWENTY_YEARS) {
    return '15年以上20年未満'
  } else if (yearsOfService === TWENTY_YEARS_OR_MORE) {
    return '20年以上'
  } else {
    return '不明'
  }
}

export const convertEmployedValue = (employed: boolean): string => {
  if (employed) {
    return '在籍中'
  } else {
    return '退職済'
  }
}

export const convertContractTypeValue = (contractType: string): string => {
  if (contractType === 'regular') {
    return '正社員'
  } else if (contractType === 'contract') {
    return '契約社員'
  } else if (contractType === 'other') {
    return 'その他'
  } else {
    return '不明'
  }
}

export const convertIsManagerValue = (isManager: boolean): string => {
  if (isManager) {
    return '管理職'
  } else {
    return '非管理職'
  }
}

export const convertIsNewGraduateValue = (isNewGraduate: boolean): string => {
  if (isNewGraduate) {
    return '新卒入社'
  } else {
    return '中途入社'
  }
}
