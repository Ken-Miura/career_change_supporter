import { reactive } from 'vue'

// eslint-disable-next-line
export function useConsultantSearchParam () {
  const form = reactive({
    companyName: '',
    departmentName: '',
    office: '',
    equalOrMoreYearsOfService: '',
    lessThanYearsOfService: '',
    employed: '',
    contractType: '',
    profession: '',
    equalOrMoreAnnualIncomeInManYen: '',
    equalOrLessAnnualIncomeInManYen: '',
    isManager: '',
    positionName: '',
    isNewGraduate: '',
    note: '',
    equalOrMoreFeePerHourInYen: '',
    equalOrLessFeePerHourInYen: ''
  })
  const setCompanyName = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      // HTMLInputElement以外が来るときはinputタグ以外に関数が指定されている。
      // inputタグ以外にしていすることは想定していないため、Errorとする。
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.companyName = target.value
  }
  const setDepartmentName = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.departmentName = target.value
  }
  const setOffice = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      // HTMLInputElement以外が来るときはinputタグ以外に関数が指定されている。
      // inputタグ以外にしていすることは想定していないため、Errorとする。
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.office = target.value
  }
  const setProfession = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.profession = target.value
  }
  const setEqualOrMoreAnnualIncomeInManYen = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.equalOrMoreAnnualIncomeInManYen = target.value
  }
  const setEqualOrLessAnnualIncomeInManYen = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.equalOrLessAnnualIncomeInManYen = target.value
  }
  const setPositionName = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.positionName = target.value
  }
  const setNote = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLTextAreaElement)) {
      throw new Error(`!(target instanceof HTMLTextAreaElement): target is ${target}`)
    }
    form.note = target.value
  }
  const setEqualOrMoreFeePerHourInYen = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.equalOrMoreFeePerHourInYen = target.value
  }
  const setEqualOrLessFeePerHourInYen = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.equalOrLessFeePerHourInYen = target.value
  }
  return {
    form,
    setCompanyName,
    setDepartmentName,
    setOffice,
    setProfession,
    setEqualOrMoreAnnualIncomeInManYen,
    setEqualOrLessAnnualIncomeInManYen,
    setPositionName,
    setNote,
    setEqualOrMoreFeePerHourInYen,
    setEqualOrLessFeePerHourInYen,
    equalOrMoreFeePerHourInYen: '',
    equalOrLessFeePerHourInYen: ''
  }
}
