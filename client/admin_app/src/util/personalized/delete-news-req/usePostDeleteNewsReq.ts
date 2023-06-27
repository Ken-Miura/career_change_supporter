import { ref } from 'vue'
import { postDeleteNewsReq } from './PostDeleteNewsReq'

export function usePostDeleteNewsReq () {
  const postDeleteNewsReqDone = ref(true)
  const postDeleteNewsReqFunc = async (newsId: number) => {
    try {
      postDeleteNewsReqDone.value = false
      const response = await postDeleteNewsReq(newsId)
      return response
    } finally {
      postDeleteNewsReqDone.value = true
    }
  }
  return {
    postDeleteNewsReqDone,
    postDeleteNewsReqFunc
  }
}
