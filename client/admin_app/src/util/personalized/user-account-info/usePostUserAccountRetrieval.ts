import { ref } from 'vue'
import { postUserAccountRetrievalByEmailAddress } from './PostUserAccountRetrievalByEmailAddress'
import { postUserAccountRetrievalByUserAccountId } from './PostUserAccountRetrievalByUserAccountId'

export function usePostUserAccountRetrieval () {
  const postUserAccountRetrievalDone = ref(true)

  const postUserAccountRetrievalByUserAccountIdFunc = async (userAccountId: number) => {
    try {
      postUserAccountRetrievalDone.value = false
      const response = await postUserAccountRetrievalByUserAccountId(userAccountId)
      return response
    } finally {
      postUserAccountRetrievalDone.value = true
    }
  }

  const postUserAccountRetrievalByEmailAddressFunc = async (emailAddress: string) => {
    try {
      postUserAccountRetrievalDone.value = false
      const response = await postUserAccountRetrievalByEmailAddress(emailAddress)
      return response
    } finally {
      postUserAccountRetrievalDone.value = true
    }
  }

  return {
    postUserAccountRetrievalDone,
    postUserAccountRetrievalByUserAccountIdFunc,
    postUserAccountRetrievalByEmailAddressFunc
  }
}
