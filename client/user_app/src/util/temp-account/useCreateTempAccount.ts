import { createTempAccount } from '@/util/temp-account/CreateTempAccount'
import { ref } from 'vue'

// eslint-disable-next-line
export function useCreateTempAccount () {
  const createTempAccountDone = ref(true)
  const createTempAccountFunc = async (emailAddress: string, password: string) => {
    try {
      createTempAccountDone.value = false
      const response = await createTempAccount(emailAddress, password)
      return response
    } finally {
      createTempAccountDone.value = true
    }
  }
  return {
    createTempAccountDone,
    createTempAccountFunc
  }
}
