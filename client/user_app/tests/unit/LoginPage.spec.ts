import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import LoginPage from '@/views/LoginPage.vue'
import EmailAddressInput from '@/components/EmailAddressInput.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import PasswordInput from '@/components/PasswordInput.vue'
import { Message } from '@/util/Message'
import { LoginResp } from '@/util/login/LoginResp'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { nextTick, ref } from 'vue'
import { refresh } from '@/util/personalized/refresh/Refresh'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { LoginResult } from '@/util/login/LoginResult'

const loginDoneMock = ref(true)
const loginFuncMock = jest.fn()
jest.mock('@/util/login/useLogin', () => ({
  useLogin: () => ({
    loginDone: loginDoneMock,
    loginFunc: loginFuncMock
  })
}))

jest.mock('@/util/personalized/refresh/Refresh')
const refreshMock = refresh as jest.MockedFunction<typeof refresh>

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

const EMAIL_ADDRESS = 'test@example.com'
const PWD = 'abcdABCD1234'

describe('LoginPage.vue', () => {
  beforeEach(() => {
    loginDoneMock.value = true
    loginFuncMock.mockReset()
    routerPushMock.mockClear()
    refreshMock.mockReset()
  })

  it('has one EmailAddressInput, one PasswordInput and one AlertMessage', () => {
    const wrapper = mount(LoginPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const emailAddresses = wrapper.findAllComponents(EmailAddressInput)
    expect(emailAddresses.length).toBe(1)
    const passwords = wrapper.findAllComponents(PasswordInput)
    expect(passwords.length).toBe(1)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
  })

  it('has AlertMessage with a hidden attribute when created', () => {
    const wrapper = mount(LoginPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).toContain('hidden')
  })

  it('moves to profile when session has already existed and user has already agreed terms of use', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())

    mount(LoginPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/profile')
  })

  it('moves to terms-of-use when session has already existed and user has not agreed terms of use yet', async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    refreshMock.mockResolvedValue(apiErrResp)

    mount(LoginPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/terms-of-use')
  })

  it('does not move when session has not existed yet', async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    refreshMock.mockResolvedValue(apiErrResp)

    mount(LoginPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays alert message ${Message.UNEXPECTED_ERR} when connection error happened on opening login page`, async () => {
    const errDetail = 'connection error'
    refreshMock.mockRejectedValue(new Error(errDetail))

    const wrapper = mount(LoginPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.UNEXPECTED_ERR)
    expect(resultMessage).toContain(errDetail)
  })

  it('displays header and WaitingCircle, no AlertMessage while requesting', async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    refreshMock.mockResolvedValue(apiErrResp)
    loginDoneMock.value = false
    const ls = { login_status: 'Finish' } as LoginResult
    loginFuncMock.mockResolvedValue(LoginResp.create(ls))

    const wrapper = mount(LoginPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const headers = wrapper.findAll('header')
    expect(headers.length).toBe(1)
    const waitingCircles = wrapper.findAllComponents(WaitingCircle)
    expect(waitingCircles.length).toBe(1)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(0)
  })

  it('moves to profile when login is successful', async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    refreshMock.mockResolvedValue(apiErrResp)
    const ls = { login_status: 'Finish' } as LoginResult
    loginFuncMock.mockResolvedValue(LoginResp.create(ls))

    const wrapper = mount(LoginPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const emailAddr = wrapper.findComponent(EmailAddressInput)
    const emailAddrInput = emailAddr.find('input')
    await emailAddrInput.setValue(EMAIL_ADDRESS)

    const pwd = wrapper.findComponent(PasswordInput)
    const pwdInput = pwd.find('input')
    await pwdInput.setValue(PWD)

    const button = wrapper.find('button')
    await button.trigger('submit')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/profile')
  })

  it('moves to mfa when login is successful and mfa is enabled', async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    refreshMock.mockResolvedValue(apiErrResp)
    const ls = { login_status: 'NeedMoreVerification' } as LoginResult
    loginFuncMock.mockResolvedValue(LoginResp.create(ls))

    const wrapper = mount(LoginPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const emailAddr = wrapper.findComponent(EmailAddressInput)
    const emailAddrInput = emailAddr.find('input')
    await emailAddrInput.setValue(EMAIL_ADDRESS)

    const pwd = wrapper.findComponent(PasswordInput)
    const pwdInput = pwd.find('input')
    await pwdInput.setValue(PWD)

    const button = wrapper.find('button')
    await button.trigger('submit')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/mfa')
  })

  it(`displays alert message ${Message.EMAIL_OR_PWD_INCORRECT_MESSAGE} when login fails`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    refreshMock.mockResolvedValue(apiErrResp)
    loginFuncMock.mockResolvedValue(ApiErrorResp.create(401, ApiError.create(Code.EMAIL_OR_PWD_INCORRECT)))

    const wrapper = mount(LoginPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const emailAddr = wrapper.findComponent(EmailAddressInput)
    const emailAddrInput = emailAddr.find('input')
    await emailAddrInput.setValue(EMAIL_ADDRESS)

    const pwd = wrapper.findComponent(PasswordInput)
    const pwdInput = pwd.find('input')
    await pwdInput.setValue(PWD)

    const button = wrapper.find('button')
    await button.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.EMAIL_OR_PWD_INCORRECT_MESSAGE)
    expect(resultMessage).toContain(Code.EMAIL_OR_PWD_INCORRECT.toString())
  })

  it(`displays alert message ${Message.ACCOUNT_DISABLED_MESSAGE} when account is disabled`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    refreshMock.mockResolvedValue(apiErrResp)
    loginFuncMock.mockResolvedValue(ApiErrorResp.create(400, ApiError.create(Code.ACCOUNT_DISABLED)))

    const wrapper = mount(LoginPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const emailAddr = wrapper.findComponent(EmailAddressInput)
    const emailAddrInput = emailAddr.find('input')
    await emailAddrInput.setValue(EMAIL_ADDRESS)

    const pwd = wrapper.findComponent(PasswordInput)
    const pwdInput = pwd.find('input')
    await pwdInput.setValue(PWD)

    const button = wrapper.find('button')
    await button.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.ACCOUNT_DISABLED_MESSAGE)
    expect(resultMessage).toContain(Code.ACCOUNT_DISABLED.toString())
  })
})
