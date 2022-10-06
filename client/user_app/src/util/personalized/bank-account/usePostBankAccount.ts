import { ref } from 'vue'
import { BankAccountRegisterReq } from './BankAccountRegisterReq'
import { postBankAccount } from './PostBankAccount'

export function usePostBankAccount () {
  const postBankAccountDone = ref(true)
  const postBankAccountFunc = async (bankAccountRegisterReq: BankAccountRegisterReq) => {
    try {
      postBankAccountDone.value = false
      const response = await postBankAccount(bankAccountRegisterReq)
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
