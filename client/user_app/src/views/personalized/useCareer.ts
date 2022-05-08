import { reactive } from 'vue'

// eslint-disable-next-line
export function useCareer () {
  const form = reactive({
    companyName: '',
    departmentName: '',
    office: '',
    careerStartYear: '',
    careerStartMonth: '',
    careerStartDay: '',
    careerEndYear: '',
    careerEndMonth: '',
    careerEndDay: '',
    contractType: 'regular',
    profession: '',
    annualIncomeInManYen: '',
    isManager: '',
    positionName: '',
    isNewGraduate: '',
    note: ''
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
  const setAnnualIncomeInManYen = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.annualIncomeInManYen = target.value
  }
  const setIsManager = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.isManager = target.value
  }
  const setPositionName = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.positionName = target.value
  }
  const setIsNewGraduate = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.isNewGraduate = target.value
  }
  const setNote = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.note = target.value
  }
  return {
    form,
    setCompanyName,
    setDepartmentName,
    setOffice,
    setProfession,
    setAnnualIncomeInManYen,
    setIsManager,
    setPositionName,
    setIsNewGraduate,
    setNote
  }
}
