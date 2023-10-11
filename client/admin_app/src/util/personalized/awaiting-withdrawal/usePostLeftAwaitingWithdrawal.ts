import { ref } from 'vue'
import { postLeftAwaitingWithdrawal } from './PostLeftAwaitingWithdrawal'

export function usePostLeftAwaitingWithdrawal () {
  const postLeftAwaitingWithdrawalDone = ref(true)
  const postLeftAwaitingWithdrawalFunc = async (consultationId: number) => {
    try {
      postLeftAwaitingWithdrawalDone.value = false
      const response = await postLeftAwaitingWithdrawal(consultationId)
      return response
    } finally {
      postLeftAwaitingWithdrawalDone.value = true
    }
  }
  return {
    postLeftAwaitingWithdrawalDone,
    postLeftAwaitingWithdrawalFunc
  }
}
