import { ref } from 'vue'
import { getIdentityOptionByUserAccountId } from './GetIdentityOptionByUserAccountId'

export function useGetIdentityOptionByUserAccountId () {
  const getIdentityOptionByUserAccountIdDone = ref(true)
  const getIdentityOptionByUserAccountIdFunc = async (userAccountId: string) => {
    try {
      getIdentityOptionByUserAccountIdDone.value = false
      const response = await getIdentityOptionByUserAccountId(userAccountId)
      return response
    } finally {
      getIdentityOptionByUserAccountIdDone.value = true
    }
  }
  return {
    getIdentityOptionByUserAccountIdDone,
    getIdentityOptionByUserAccountIdFunc
  }
}
