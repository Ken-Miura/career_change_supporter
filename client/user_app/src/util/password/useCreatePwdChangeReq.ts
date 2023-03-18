import { createPwdChangeReq } from '@/util/password/CreatePwdChangeReq'
import { ref } from 'vue'

// eslint-disable-next-line
export function useCreatePwdChangeReq () {
  const createPwdChangeReqDone = ref(true)
  const createPwdChangeReqFunc = async (emailAddress: string) => {
    try {
      createPwdChangeReqDone.value = false
      const response = await createPwdChangeReq(emailAddress)
      return response
    } finally {
      createPwdChangeReqDone.value = true
    }
  }
  return {
    createPwdChangeReqDone,
    createPwdChangeReqFunc
  }
}
