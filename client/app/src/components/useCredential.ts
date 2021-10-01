import { computed, reactive } from 'vue'

// eslint-disable-next-line
export function useCredentil () {
  const form = reactive({
    emailAddress: '',
    password: '',
    passwordConfirmation: ''
  })
  const setEmailAddress = (emailAddress: string) => {
    form.emailAddress = emailAddress
  }
  const setPassword = (password: string) => {
    form.password = password
  }
  const setPasswordConfirmation = (passwordConfirmation: string) => {
    form.passwordConfirmation = passwordConfirmation
  }
  const passwordsAreSame = computed(() => form.password === form.passwordConfirmation)
  return {
    form,
    setEmailAddress,
    setPassword,
    setPasswordConfirmation,
    passwordsAreSame
  }
}
