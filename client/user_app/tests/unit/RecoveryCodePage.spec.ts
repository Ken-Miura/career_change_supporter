import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import RecoveryCodePage from '@/views/RecoveryCodePage.vue'

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

const postRecoveryCodeDoneMock = ref(true)
const postRecoveryCodeFuncMock = jest.fn()
jest.mock('@/util/mfa/usePostRecoveryCode', () => ({
  usePostRecoveryCode: () => ({
    postRecoveryCodeDone: postRecoveryCodeDoneMock,
    postRecoveryCodeFunc: postRecoveryCodeFuncMock
  })
}))

describe('RecoveryCodePage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    postRecoveryCodeDoneMock.value = true
    postRecoveryCodeFuncMock.mockReset()
  })

  it('has WaitingCircle while calling postRecoveryCode', async () => {
    postRecoveryCodeDoneMock.value = false
    const wrapper = mount(RecoveryCodePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(0)

    const header = wrapper.find('[data-test="header"]')
    expect(header.text()).toContain('就職先・転職先を見極めるためのサイト')
    const waitingCircles = wrapper.findAllComponents(WaitingCircle)
    expect(waitingCircles.length).toBe(1)
  })
})
