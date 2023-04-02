import { ref } from 'vue'
import { postPassCode } from './PostPassCode'

// eslint-disable-next-line
export function usePostPassCode () {
  const postPassCodeDone = ref(true)
  const postPassCodeFunc = async (passCode: string) => {
    try {
      postPassCodeDone.value = false
      const response = await postPassCode(passCode)
      return response
    } finally {
      postPassCodeDone.value = true
    }
  }
  return {
    postPassCodeDone,
    postPassCodeFunc
  }
}
