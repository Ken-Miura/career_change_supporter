import { refresh } from '@/util/personalized/refresh/Refresh'
import { ref } from 'vue'

export function useRefresh () {
  const refreshDone = ref(true)
  const refreshFunc = async () => {
    try {
      refreshDone.value = false
      const response = await refresh()
      return response
    } finally {
      refreshDone.value = true
    }
  }
  return {
    refreshDone,
    refreshFunc
  }
}
