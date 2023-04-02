import { ref } from 'vue'
import { postRecoveryCode } from './PostRecoveryCode'

// eslint-disable-next-line
export function usePostRecoveryCode () {
  const postRecoveryCodeDone = ref(true)
  const postRecoveryCodeFunc = async (recoveryCode: string) => {
    try {
      postRecoveryCodeDone.value = false
      const response = await postRecoveryCode(recoveryCode)
      return response
    } finally {
      postRecoveryCodeDone.value = true
    }
  }
  return {
    postRecoveryCodeDone,
    postRecoveryCodeFunc
  }
}
