import { ref } from 'vue'
import { getAgreementsByUserAccountId } from './GetAgreementsByUserAccountId'

export function useGetAgreementsByUserAccountId () {
  const getAgreementsByUserAccountIdDone = ref(true)
  const getAgreementsByUserAccountIdFunc = async (userAccountId: string) => {
    try {
      getAgreementsByUserAccountIdDone.value = false
      const response = await getAgreementsByUserAccountId(userAccountId)
      return response
    } finally {
      getAgreementsByUserAccountIdDone.value = true
    }
  }
  return {
    getAgreementsByUserAccountIdDone,
    getAgreementsByUserAccountIdFunc
  }
}
