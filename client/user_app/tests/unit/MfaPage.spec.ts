import { mount, flushPromises, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import MfaPage from '@/views/MfaPage.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import PassCodeInput from '@/components/PassCodeInput.vue'
import { PostPassCodeResp } from '@/util/mfa/PostPassCodeResp'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { Message } from '@/util/Message'

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

  it('moves profile if pass code check is successfull', async () => {
    const resp = PostPassCodeResp.create()
    postPassCodeFuncMock.mockResolvedValue(resp)
    const wrapper = mount(MfaPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const passCodeInputComponent = wrapper.findComponent(PassCodeInput)
    const passCodeInput = passCodeInputComponent.find('input')
    await passCodeInput.setValue('123456')

    const loginButton = wrapper.find('[data-test="login-button"]')
    await loginButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/profile')
  })

  it(`moves login if ${Code.UNAUTHORIZED} is returned`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    postPassCodeFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(MfaPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const passCodeInputComponent = wrapper.findComponent(PassCodeInput)
    const passCodeInput = passCodeInputComponent.find('input')
    await passCodeInput.setValue('123456')

    const loginButton = wrapper.find('[data-test="login-button"]')
    await loginButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/login')
  })

  it(`displays alert message ${Message.LOGIN_FAILED} when connection error happened`, async () => {
    const errDetail = 'connection error'
    postPassCodeFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(MfaPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const passCodeInputComponent = wrapper.findComponent(PassCodeInput)
    const passCodeInput = passCodeInputComponent.find('input')
    await passCodeInput.setValue('123456')

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

  it(`displays alert message ${Message.INVALID_PASS_CODE_MESSAGE} if ${Code.INVALID_PASS_CODE} is returned`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_PASS_CODE))
    postPassCodeFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(MfaPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const passCodeInputComponent = wrapper.findComponent(PassCodeInput)
    const passCodeInput = passCodeInputComponent.find('input')
    await passCodeInput.setValue('123456')

    const loginButton = wrapper.find('[data-test="login-button"]')
    await loginButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)

    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_PASS_CODE_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_PASS_CODE.toString())
  })

  it(`displays alert message ${Message.MFA_IS_NOT_ENABLED_MESSAGE} if ${Code.MFA_IS_NOT_ENABLED} is returned`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.MFA_IS_NOT_ENABLED))
    postPassCodeFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(MfaPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const passCodeInputComponent = wrapper.findComponent(PassCodeInput)
    const passCodeInput = passCodeInputComponent.find('input')
    await passCodeInput.setValue('123456')

    const loginButton = wrapper.find('[data-test="login-button"]')
    await loginButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)

    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.MFA_IS_NOT_ENABLED_MESSAGE)
    expect(resultMessage).toContain(Code.MFA_IS_NOT_ENABLED.toString())
  })

  it(`displays alert message ${Message.PASS_CODE_DOES_NOT_MATCH_MESSAGE} if ${Code.PASS_CODE_DOES_NOT_MATCH} is returned`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.PASS_CODE_DOES_NOT_MATCH))
    postPassCodeFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(MfaPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const passCodeInputComponent = wrapper.findComponent(PassCodeInput)
    const passCodeInput = passCodeInputComponent.find('input')
    await passCodeInput.setValue('123456')

    const loginButton = wrapper.find('[data-test="login-button"]')
    await loginButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)

    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.PASS_CODE_DOES_NOT_MATCH_MESSAGE)
    expect(resultMessage).toContain(Code.PASS_CODE_DOES_NOT_MATCH.toString())
  })

  it(`displays alert message ${Message.NO_ACCOUNT_FOUND_MESSAGE} if ${Code.NO_ACCOUNT_FOUND} is returned`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_ACCOUNT_FOUND))
    postPassCodeFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(MfaPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const passCodeInputComponent = wrapper.findComponent(PassCodeInput)
    const passCodeInput = passCodeInputComponent.find('input')
    await passCodeInput.setValue('123456')

    const loginButton = wrapper.find('[data-test="login-button"]')
    await loginButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)

    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.NO_ACCOUNT_FOUND_MESSAGE)
    expect(resultMessage).toContain(Code.NO_ACCOUNT_FOUND.toString())
  })
})
