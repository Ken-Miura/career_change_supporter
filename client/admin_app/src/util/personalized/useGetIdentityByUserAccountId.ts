import { ref } from 'vue'
import { getIdentityByUserAccountId } from './GetIdentityByUserAccountId'

export function useGetIdentityByUserAccountId () {
  const waitingGetIdentityByUserAccountIdDone = ref(false)
  const getIdentityByUserAccountIdFunc = async (userAccountId: string) => {
    try {
      waitingGetIdentityByUserAccountIdDone.value = true
      const response = await getIdentityByUserAccountId(userAccountId)
      return response
    } finally {
      waitingGetIdentityByUserAccountIdDone.value = false
    }
  }
  return {
    waitingGetIdentityByUserAccountIdDone,
    getIdentityByUserAccountIdFunc
  }
}
