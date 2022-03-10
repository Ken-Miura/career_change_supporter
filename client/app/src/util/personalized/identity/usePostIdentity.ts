import { postIdentity } from '@/util/personalized/identity/PostIdentity'
import { ref } from 'vue'
import { Identity } from '../profile/Identity'

// eslint-disable-next-line
export function usePostIdentity () {
  const postIdentityDone = ref(false)
  const postIdentityFunc = async (identity: Identity, image1: File, image2: File | null) => {
    try {
      const response = await postIdentity(identity, image1, image2)
      return response
    } finally {
      postIdentityDone.value = true
    }
  }
  return {
    postIdentityDone,
    postIdentityFunc
  }
}
