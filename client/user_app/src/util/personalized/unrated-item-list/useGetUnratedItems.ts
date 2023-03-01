import { ref } from 'vue'
import { getUnratedItems } from './GetUnratedItems'

// eslint-disable-next-line
export function useGetUnratedItems () {
  const getUnratedItemsDone = ref(true)
  const getUnratedItemsFunc = async () => {
    try {
      getUnratedItemsDone.value = false
      const response = await getUnratedItems()
      return response
    } finally {
      getUnratedItemsDone.value = true
    }
  }
  return {
    getUnratedItemsDone,
    getUnratedItemsFunc
  }
}
