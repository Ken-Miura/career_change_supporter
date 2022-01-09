import { reactive } from 'vue'

// eslint-disable-next-line
export function useIdentity () {
  const form = reactive({
    lastName: '',
    firstName: ''
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
  return {
    form,
    setLastName,
    setFirstName
  }
}
