import { ref } from 'vue'
import { getTempMfaSecret } from './GetTempMfaSecret'

// eslint-disable-next-line
export function useGetTempMfaSecret () {
  const getTempMfaSecretDone = ref(true)
  const getTempMfaSecretFunc = async () => {
    try {
      getTempMfaSecretDone.value = false
      const response = await getTempMfaSecret()
      return response
    } finally {
      getTempMfaSecretDone.value = true
    }
  }
  return {
    getTempMfaSecretDone,
    getTempMfaSecretFunc
  }
}
