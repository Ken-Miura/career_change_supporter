import { mount, flushPromises, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import MfaPage from '@/views/MfaPage.vue'

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

  it('has WaitingCircle while calling postPassCode', async () => {
    postPassCodeDoneMock.value = false
    const wrapper = mount(MfaPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const waitingCircles = wrapper.findAllComponents(WaitingCircle)
    expect(waitingCircles.length).toBe(1)
  })
})
