import { reactive } from 'vue'

// eslint-disable-next-line
export function useIdentity () {
  const form = reactive({
    lastName: '',
    firstName: '',
    lastNameFurigana: '',
    firstNameFurigana: '',
    sex: '' as 'male' | 'female' | string,
    dayOfBirth: '',
    monthOfBirth: '',
    yearOfBirth: '',
    prefecture: '',
    city: '',
    addressLine1: '',
    address_line2: '',
    telephone_number: ''
  })
  const setLastName = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      // HTMLInputElement以外が来るときはinputタグ以外に関数が指定されている。
      // inputタグ以外にしていすることは想定していないため、Errorとする。
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.lastName = target.value
  }
  const setFirstName = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.firstName = target.value
  }
  const setLastNameFurigana = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      // HTMLInputElement以外が来るときはinputタグ以外に関数が指定されている。
      // inputタグ以外にしていすることは想定していないため、Errorとする。
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.lastNameFurigana = target.value
  }
  const setFirstNameFurigana = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.firstNameFurigana = target.value
  }
  return {
    form,
    setLastName,
    setFirstName,
    setLastNameFurigana,
    setFirstNameFurigana
  }
}
