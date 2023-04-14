import { mount, flushPromises, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import MfaPage from '@/views/MfaPage.vue'
import AlertMessage from '@/components/AlertMessage.vue'

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

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(0)

    const header = wrapper.find('[data-test="header"]')
    expect(header.text()).toContain('就職先・転職先を見極めるためのサイト')
    const waitingCircles = wrapper.findAllComponents(WaitingCircle)
    expect(waitingCircles.length).toBe(1)
  })

  it('displays pass code input', async () => {
    const wrapper = mount(MfaPage, {
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

    const loginLabel = wrapper.find('[data-test="login-label"]')
    expect(loginLabel.text()).toContain('ログイン')

    const loginDescription = wrapper.find('[data-test="login-description"]')
    expect(loginDescription.text()).toContain('認証アプリに表示されているパスコード（6桁の数字）を入力して下さい。')

    const recoveryCodeLinkArea = wrapper.find('[data-test="recovery-code-link-area"]')
    const recoveryCodeRouterLink = recoveryCodeLinkArea.findComponent(RouterLinkStub)
    expect(recoveryCodeRouterLink.text()).toContain('リカバリーコードを用いたログイン')
    expect(recoveryCodeRouterLink.props().to).toBe('/recovery-code')

    const loginButton = wrapper.find('[data-test="login-button"]')
    expect(loginButton.text()).toContain('ログイン')
  })
})
