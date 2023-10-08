import { ref } from 'vue'
import { postAwaitingWithdrawal } from './PostAwaitingWithdrawal'

export function usePostAwaitingWithdrawal () {
  const postAwaitingWithdrawalDone = ref(true)
  const postAwaitingWithdrawalFunc = async (consultationId: number) => {
    try {
      postAwaitingWithdrawalDone.value = false
      const response = await postAwaitingWithdrawal(consultationId)
      return response
    } finally {
      postAwaitingWithdrawalDone.value = true
    }
  }
  return {
    postAwaitingWithdrawalDone,
    postAwaitingWithdrawalFunc
  }
}
