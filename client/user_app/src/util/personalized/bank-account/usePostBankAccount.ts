import { ref } from 'vue'
import { BankAccount } from '../BankAccount'
import { postBankAccount } from './PostBankAccount'

export function usePostBankAccount () {
  const postBankAccountDone = ref(true)
  const postBankAccountFunc = async (bankAccount: BankAccount) => {
    try {
      postBankAccountDone.value = false
      const response = await postBankAccount(bankAccount)
      return response
    } finally {
      postBankAccountDone.value = true
    }
  }
  return {
    postBankAccountDone,
    postBankAccountFunc
  }
}
