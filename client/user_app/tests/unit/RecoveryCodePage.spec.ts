import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import RecoveryCodePage from '@/views/RecoveryCodePage.vue'
import { PostRecoveryCodeResp } from '@/util/mfa/PostRecoveryCodeResp'
import { Code } from '@/util/Error'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Message } from '@/util/Message'

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

  it('displays recovery code input', async () => {
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

    const loginLabel = wrapper.find('[data-test="login-label"]')
    expect(loginLabel.text()).toContain('ログイン')

    const loginDescription = wrapper.find('[data-test="login-description"]')
    expect(loginDescription.text()).toContain('二段階認証設定時に保存したリカバリーコードを入力して下さい。リカバリーコードによるログイン後、二段階認証の設定は無効化されますので、適宜再設定を行うようお願いします。')

    const passCodeLinkArea = wrapper.find('[data-test="pass-code-link-area"]')
    const passCodeRouterLink = passCodeLinkArea.findComponent(RouterLinkStub)
    expect(passCodeRouterLink.text()).toContain('認証アプリ（パスコード）を用いたログイン')
    expect(passCodeRouterLink.props().to).toBe('/mfa')

    const loginButton = wrapper.find('[data-test="login-button"]')
    expect(loginButton.text()).toContain('ログイン')
  })

  it('moves profile if recovery code check is successfull', async () => {
    const resp = PostRecoveryCodeResp.create()
    postRecoveryCodeFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RecoveryCodePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const recoveryCodeInput = wrapper.find('[data-test="recovery-code-input"]')
    await recoveryCodeInput.setValue('8fa6557546aa49eabe5e18b5214b9369')

    const loginButton = wrapper.find('[data-test="login-button"]')
    await loginButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/profile')
  })

  it(`moves login if ${Code.UNAUTHORIZED} is returned`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    postRecoveryCodeFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(RecoveryCodePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const recoveryCodeInput = wrapper.find('[data-test="recovery-code-input"]')
    await recoveryCodeInput.setValue('8fa6557546aa49eabe5e18b5214b9369')

    const loginButton = wrapper.find('[data-test="login-button"]')
    await loginButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/login')
  })

  it(`displays alert message ${Message.LOGIN_FAILED} when connection error happened`, async () => {
    const errDetail = 'connection error'
    postRecoveryCodeFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(RecoveryCodePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const recoveryCodeInput = wrapper.find('[data-test="recovery-code-input"]')
    await recoveryCodeInput.setValue('8fa6557546aa49eabe5e18b5214b9369')

    const loginButton = wrapper.find('[data-test="login-button"]')
    await loginButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)

    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.LOGIN_FAILED)
    expect(resultMessage).toContain(errDetail)
  })

  it(`displays alert message ${Message.INVALID_RECOVERY_CODE_MESSAGE} if ${Code.INVALID_RECOVERY_CODE} is returned`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_RECOVERY_CODE))
    postRecoveryCodeFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(RecoveryCodePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const recoveryCodeInput = wrapper.find('[data-test="recovery-code-input"]')
    await recoveryCodeInput.setValue('8fa6557546aa49eabe5e18b5214b9369')

    const loginButton = wrapper.find('[data-test="login-button"]')
    await loginButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)

    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_RECOVERY_CODE_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_RECOVERY_CODE.toString())
  })
})
