import { getNews } from '@/util/news/GetNews'
import { ref } from 'vue'

// eslint-disable-next-line
export function useGetNews () {
  const getNewsDone = ref(true)
  const getNewsFunc = async () => {
    try {
      getNewsDone.value = false
      const response = await getNews()
      return response
    } finally {
      getNewsDone.value = true
    }
  }
  return {
    getNewsDone,
    getNewsFunc
  }
}
