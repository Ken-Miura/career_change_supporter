import { ref } from 'vue'

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

const postPassCodeDoneMock = ref(true)
const postPassCodeFuncMock = jest.fn()
jest.mock('@/util/mfa/usePostPassCode', () => ({
  usePostPassCode: () => ({
    postPassCodeDone: postPassCodeDoneMock,
    postPassCodeFunc: postPassCodeFuncMock
  })
}))

describe('MfaPage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    postPassCodeDoneMock.value = true
    postPassCodeFuncMock.mockReset()
  })

  it('test', async () => {
    console.log('test')
  })
})
