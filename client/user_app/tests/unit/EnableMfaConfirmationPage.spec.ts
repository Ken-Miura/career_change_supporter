import { ref } from 'vue'

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

describe('EnableMfaConfirmationPage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    getTempMfaSecretDoneMock.value = true
    getTempMfaSecretFuncMock.mockReset()
    postEnableMfaReqDoneMock.value = true
    postEnableMfaReqFuncMock.mockReset()
  })

  it('', async () => {
    console.log('test')
  })
})
