import { mount, RouterLinkStub } from '@vue/test-utils'
import NewAccount from '@/views/NewAccount.vue'
import EmailAddress from '@/components/EmailAddress.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import Password from '@/components/Password.vue'
import { createTempAccount } from '@/util/new-account/CreateTempAccount'
import { CreateTempAccountResp } from '@/util/new-account/CreateTempAccountResp'
import { Message } from '@/util/Message'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { nextTick } from 'vue'
import { Code } from '@/util/Error'

jest.mock('@/util/new-account/CreateTempAccount')
const createTempAccountMock = createTempAccount as jest.MockedFunction<typeof createTempAccount>

// 参考: https://stackoverflow.com/questions/68763693/vue-routers-injection-fails-during-a-jest-unit-test
const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

const emailAddress = 'test@example.com'
const pwd = 'abcdABCD1234'

describe('NewAccount.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    createTempAccountMock.mockReset()
  })

  it('has one EmailAddress, two Passwords and one AlertMessage', () => {
    const wrapper = mount(NewAccount, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const emailAddresses = wrapper.findAllComponents(EmailAddress)
    expect(emailAddresses.length).toBe(1)
    const passwords = wrapper.findAllComponents(Password)
    expect(passwords.length).toBe(2)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
  })

  it('has AlertMessage with a hidden attribute when created', () => {
    const wrapper = mount(NewAccount, {
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

  it('moves to TempAccountCreated when email address and password are passed', async () => {
    createTempAccountMock.mockResolvedValue(CreateTempAccountResp.create(emailAddress))

    const wrapper = mount(NewAccount, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const emailAddr = wrapper.findComponent(EmailAddress)
    const emailAddrInput = emailAddr.find('input')
    emailAddrInput.setValue(emailAddress)

    const pwds = wrapper.findAllComponents(Password)
    const pwdInput = pwds[0].find('input')
    pwdInput.setValue(pwd)
    const pwdConfirmationInput = pwds[1].find('input')
    pwdConfirmationInput.setValue(pwd)

    const button = wrapper.find('button')
    await button.trigger('submit')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    const data = JSON.parse(`{ "name": "TempAccountCreated", "params": {"emailAddress": "${emailAddress}"} }`)
    expect(routerPushMock).toHaveBeenCalledWith(data)
  })

  it(`displays alert message ${Message.ACCOUNT_ALREADY_EXISTS_MESSAGE} when account already exists`, async () => {
    const apiErr = ApiError.create(Code.ACCOUNT_ALREADY_EXISTS)
    const apiErrorResp = ApiErrorResp.create(400, apiErr)
    createTempAccountMock.mockResolvedValue(apiErrorResp)

    const wrapper = mount(NewAccount, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const emailAddr = wrapper.findComponent(EmailAddress)
    const emailAddrInput = emailAddr.find('input')
    emailAddrInput.setValue(emailAddress)

    const pwds = wrapper.findAllComponents(Password)
    const pwdInput = pwds[0].find('input')
    pwdInput.setValue(pwd)
    const pwdConfirmationInput = pwds[1].find('input')
    pwdConfirmationInput.setValue(pwd)

    const button = wrapper.find('button')
    await button.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.ACCOUNT_ALREADY_EXISTS_MESSAGE)
    expect(resultMessage).toContain(Code.ACCOUNT_ALREADY_EXISTS)
  })

  it(`displays alert message ${Message.REACH_TEMP_ACCOUNTS_LIMIT_MESSAGE} when reach new account limit`, async () => {
    const apiErr = ApiError.create(Code.REACH_TEMP_ACCOUNTS_LIMIT)
    const apiErrorResp = ApiErrorResp.create(400, apiErr)
    createTempAccountMock.mockResolvedValue(apiErrorResp)

    const wrapper = mount(NewAccount, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const emailAddr = wrapper.findComponent(EmailAddress)
    const emailAddrInput = emailAddr.find('input')
    emailAddrInput.setValue(emailAddress)

    const pwds = wrapper.findAllComponents(Password)
    const pwdInput = pwds[0].find('input')
    pwdInput.setValue(pwd)
    const pwdConfirmationInput = pwds[1].find('input')
    pwdConfirmationInput.setValue(pwd)

    const button = wrapper.find('button')
    await button.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.REACH_TEMP_ACCOUNTS_LIMIT_MESSAGE)
    expect(resultMessage).toContain(Code.REACH_TEMP_ACCOUNTS_LIMIT)
  })
})
