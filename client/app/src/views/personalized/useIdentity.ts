import { createPrefectureList } from '@/util/personalized/profile/createPrefectureList'
import { reactive } from 'vue'

// eslint-disable-next-line
export function useIdentity () {
  const initialValue = createPrefectureList()[0]
  const form = reactive({
    lastName: '',
    firstName: '',
    lastNameFurigana: '',
    firstNameFurigana: '',
    sex: 'male' as 'male' | 'female',
    dayOfBirth: '',
    monthOfBirth: '',
    yearOfBirth: '',
    prefecture: initialValue,
    city: '',
    addressLine1: '',
    addressLine2: '',
    telephoneNumber: ''
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
  const setCity = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.city = target.value
  }
  const setAddressLine1 = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.addressLine1 = target.value
  }
  const setAddressLine2 = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.addressLine2 = target.value
  }
  const setTelephoneNumber = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    form.telephoneNumber = target.value
  }
  return {
    form,
    setLastName,
    setFirstName,
    setLastNameFurigana,
    setFirstNameFurigana,
    setCity,
    setAddressLine1,
    setAddressLine2,
    setTelephoneNumber
  }
}
