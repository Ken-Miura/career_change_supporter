import { updatePassword } from '@/util/password/UpdatePassword'
import { ref } from 'vue'

// eslint-disable-next-line
export function useUpdatePassword () {
  const updatePasswordDone = ref(true)
  const updatePasswordFunc = async (pwdChangeReqId: string, password: string) => {
    try {
      updatePasswordDone.value = false
      const response = await updatePassword(pwdChangeReqId, password)
      return response
    } finally {
      updatePasswordDone.value = true
    }
  }
  return {
    updatePasswordDone,
    updatePasswordFunc
  }
}
