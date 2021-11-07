import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import Login from '@/views/Login.vue'
import { refresh } from '@/util/refresh/Refresh'
import EmailAddress from '@/components/EmailAddress.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import Password from '@/components/Password.vue'
import { Message } from '@/util/Message'
import { login } from '@/util/login/Login'
import { LoginResp } from '@/util/login/LoginResp'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { nextTick } from 'vue'
import { checkAgreementStatus } from '@/util/agreement-status/CheckAgreementStatus'
import { CheckAgreementStatusResp } from '@/util/agreement-status/CheckAgreementStatusResp'

jest.mock('@/util/refresh/Refresh')
const refreshMock = refresh as jest.MockedFunction<typeof refresh>

jest.mock('@/util/login/Login')
const loginMock = login as jest.MockedFunction<typeof login>

jest.mock('@/util/agreement-status/CheckAgreementStatus')
const checkAgreementStatusMock = checkAgreementStatus as jest.MockedFunction<typeof checkAgreementStatus>

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

const EMAIL_ADDRESS = 'test@example.com'
const PWD = 'abcdABCD1234'

describe('Login.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    refreshMock.mockReset()
    loginMock.mockReset()
    checkAgreementStatusMock.mockReset()
  })

  it('has one EmailAddress, one Password and one AlertMessage', () => {
    const wrapper = mount(Login, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const emailAddresses = wrapper.findAllComponents(EmailAddress)
    expect(emailAddresses.length).toBe(1)
    const passwords = wrapper.findAllComponents(Password)
    expect(passwords.length).toBe(1)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
  })

  it('has AlertMessage with a hidden attribute when created', () => {
    const wrapper = mount(Login, {
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
    refreshMock.mockResolvedValue(true)
    checkAgreementStatusMock.mockResolvedValue(CheckAgreementStatusResp.create())

    mount(Login, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('profile')
  })

  it('moves to terms-of-use when session has already existed and user has not agreed terms of use yet', async () => {
    refreshMock.mockResolvedValue(true)
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    checkAgreementStatusMock.mockResolvedValue(apiErrResp)

    mount(Login, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('terms-of-use')
  })

  // かなりのレアケースなのでテストは必要ないと思われる。このテストが開発の進行を阻害する場合、削除を検討する。
  it('moves to login when sessoin has expired on checking agreement status', async () => {
    refreshMock.mockResolvedValue(true)
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    checkAgreementStatusMock.mockResolvedValue(apiErrResp)

    mount(Login, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('login')
  })

  it('does not move when session has not existed yet', async () => {
    refreshMock.mockResolvedValue(false)

    mount(Login, {
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

    const wrapper = mount(Login, {
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

  it('moves to profile when login is successful', async () => {
    refreshMock.mockResolvedValue(false)
    loginMock.mockResolvedValue(LoginResp.create())

    const wrapper = mount(Login, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const emailAddr = wrapper.findComponent(EmailAddress)
    const emailAddrInput = emailAddr.find('input')
    emailAddrInput.setValue(EMAIL_ADDRESS)

    const pwd = wrapper.findComponent(Password)
    const pwdInput = pwd.find('input')
    pwdInput.setValue(PWD)

    const button = wrapper.find('button')
    await button.trigger('submit')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('profile')
  })

  it(`displays alert message ${Message.EMAIL_OR_PWD_INCORRECT_MESSAGE} when login fails`, async () => {
    refreshMock.mockResolvedValue(false)
    loginMock.mockResolvedValue(ApiErrorResp.create(401, ApiError.create(Code.EMAIL_OR_PWD_INCORRECT)))

    const wrapper = mount(Login, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const emailAddr = wrapper.findComponent(EmailAddress)
    const emailAddrInput = emailAddr.find('input')
    emailAddrInput.setValue(EMAIL_ADDRESS)

    const pwd = wrapper.findComponent(Password)
    const pwdInput = pwd.find('input')
    pwdInput.setValue(PWD)

    const button = wrapper.find('button')
    await button.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.EMAIL_OR_PWD_INCORRECT_MESSAGE)
    expect(resultMessage).toContain(Code.EMAIL_OR_PWD_INCORRECT)
  })
})
