import { ref } from 'vue'
import { postTempMfaSecret } from './PostTempMfaSecret'

export function usePostTempMfaSecret () {
  const postTempMfaSecretDone = ref(true)
  const postTempMfaSecretFunc = async () => {
    try {
      postTempMfaSecretDone.value = false
      const response = await postTempMfaSecret()
      return response
    } finally {
      postTempMfaSecretDone.value = true
    }
  }
  return {
    postTempMfaSecretDone,
    postTempMfaSecretFunc
  }
}
