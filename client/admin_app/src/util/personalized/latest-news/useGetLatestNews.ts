import { ref } from 'vue'
import { getLatestNews } from './GetLatestNews'

export function useGetLatestNews () {
  const getLatestNewsDone = ref(true)
  const getLatestNewsFunc = async () => {
    try {
      getLatestNewsDone.value = false
      const response = await getLatestNews()
      return response
    } finally {
      getLatestNewsDone.value = true
    }
  }
  return {
    getLatestNewsDone,
    getLatestNewsFunc
  }
}
