import { ref } from 'vue'
import { deleteAccount } from './DeleteAccount'

export function useDeleteAccount () {
  const deleteAccountDone = ref(true)
  const deleteAccountFunc = async (accountDeleteConfirmed: boolean) => {
    try {
      deleteAccountDone.value = false
      const response = await deleteAccount(accountDeleteConfirmed)
      return response
    } finally {
      deleteAccountDone.value = true
    }
  }
  return {
    deleteAccountDone,
    deleteAccountFunc
  }
}
