import { login } from '@/util/login/Login'
import { ref } from 'vue'

// eslint-disable-next-line
export function useLogin () {
  const loginDone = ref(true)
  const loginFunc = async (emailAddress: string, password: string) => {
    try {
      loginDone.value = false
      const response = await login(emailAddress, password)
      return response
    } finally {
      loginDone.value = true
    }
  }
  return {
    loginDone,
    loginFunc
  }
}
