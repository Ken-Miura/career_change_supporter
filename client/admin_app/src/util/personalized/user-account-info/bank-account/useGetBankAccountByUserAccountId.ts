import { ref } from 'vue'
import { getBankAccountByUserAccountId } from './GetBankAccountByUserAccountId'

export function useGetBankAccountByUserAccountId () {
  const getBankAccountByUserAccountIdDone = ref(true)
  const getBankAccountByUserAccountIdFunc = async (userAccountId: string) => {
    try {
      getBankAccountByUserAccountIdDone.value = false
      const response = await getBankAccountByUserAccountId(userAccountId)
      return response
    } finally {
      getBankAccountByUserAccountIdDone.value = true
    }
  }
  return {
    getBankAccountByUserAccountIdDone,
    getBankAccountByUserAccountIdFunc
  }
}
