import { mount, flushPromises, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import EnableMfaConfirmationPage from '@/views/personalized/EnableMfaConfirmationPage.vue'
import { GetTempMfaSecretResp } from '@/util/personalized/enable-mfa-confirmation/GetTempMfaSecretResp'
import { TempMfaSecret } from '@/util/personalized/enable-mfa-confirmation/TempMfaSecret'

const getTempMfaSecretDoneMock = ref(true)
const getTempMfaSecretFuncMock = jest.fn()
jest.mock('@/util/personalized/enable-mfa-confirmation/useGetTempMfaSecret', () => ({
  useGetTempMfaSecret: () => ({
    getTempMfaSecretDone: getTempMfaSecretDoneMock,
    getTempMfaSecretFunc: getTempMfaSecretFuncMock
  })
}))

const postEnableMfaReqDoneMock = ref(true)
const postEnableMfaReqFuncMock = jest.fn()
jest.mock('@/util/personalized/enable-mfa-confirmation/usePostEnableMfaReq', () => ({
  usePostEnableMfaReq: () => ({
    postEnableMfaReqDone: postEnableMfaReqDoneMock,
    postEnableMfaReqFunc: postEnableMfaReqFuncMock
  })
}))

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

const recoveryCodeMock = null as string | null
const storeCommitMock = jest.fn()
jest.mock('vuex', () => ({
  useStore: () => ({
    commit: storeCommitMock,
    state: {
      recoveryCode: recoveryCodeMock
    }
  })
}))

const tempMfaSecret = {
  base64_encoded_image: '',
  base32_encoded_secret: ''
} as TempMfaSecret

describe('EnableMfaConfirmationPage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    getTempMfaSecretDoneMock.value = true
    getTempMfaSecretFuncMock.mockReset()
    postEnableMfaReqDoneMock.value = true
    postEnableMfaReqFuncMock.mockReset()
  })

  it('test', async () => {
    const resp = GetTempMfaSecretResp.create(tempMfaSecret)
    getTempMfaSecretFuncMock.mockResolvedValue(resp)
    const wrapper = mount(EnableMfaConfirmationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
  })
})
