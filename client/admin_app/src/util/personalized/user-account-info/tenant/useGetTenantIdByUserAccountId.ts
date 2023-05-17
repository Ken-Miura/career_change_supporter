import { ref } from 'vue'
import { getTenantIdByUserAccountId } from '../tenant/GetTenantIdByUserAccountId'

export function useGetTenantIdByUserAccountId () {
  const getTenantIdByUserAccountIdDone = ref(true)
  const getTenantIdByUserAccountIdFunc = async (userAccountId: string) => {
    try {
      getTenantIdByUserAccountIdDone.value = false
      const response = await getTenantIdByUserAccountId(userAccountId)
      return response
    } finally {
      getTenantIdByUserAccountIdDone.value = true
    }
  }
  return {
    getTenantIdByUserAccountIdDone,
    getTenantIdByUserAccountIdFunc
  }
}
