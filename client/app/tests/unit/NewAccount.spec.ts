import { mount, RouterLinkStub } from '@vue/test-utils'
import NewAccount from '@/views/NewAccount.vue'
import EmailAddressInput from '@/components/EmailAddressInput.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import PasswordInput from '@/components/PasswordInput.vue'
import { createTempAccount } from '@/util/temp-account/CreateTempAccount'
import { CreateTempAccountResp } from '@/util/temp-account/CreateTempAccountResp'
import { Message } from '@/util/Message'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { nextTick } from 'vue'
import { Code } from '@/util/Error'

jest.mock('@/util/temp-account/CreateTempAccount')
const createTempAccountMock = createTempAccount as jest.MockedFunction<typeof createTempAccount>

// 参考: https://stackoverflow.com/questions/68763693/vue-routers-injection-fails-during-a-jest-unit-test
const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

const EMAIL_ADDRESS = 'test@example.com'
const PWD = 'abcdABCD1234'
const DIFFERENT_PWD = '1234abcdABCD'

describe('NewAccount.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    createTempAccountMock.mockReset()
  })

  it('has one EmailAddressInput, two PasswordInputs and one AlertMessage', () => {
    const wrapper = mount(NewAccount, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const emailAddresses = wrapper.findAllComponents(EmailAddressInput)
    expect(emailAddresses.length).toBe(1)
    const passwords = wrapper.findAllComponents(PasswordInput)
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
    createTempAccountMock.mockResolvedValue(CreateTempAccountResp.create(EMAIL_ADDRESS))

    const wrapper = mount(NewAccount, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const emailAddr = wrapper.findComponent(EmailAddressInput)
    const emailAddrInput = emailAddr.find('input')
    emailAddrInput.setValue(EMAIL_ADDRESS)

    const pwds = wrapper.findAllComponents(PasswordInput)
    const pwdInput = pwds[0].find('input')
    pwdInput.setValue(PWD)
    const pwdConfirmationInput = pwds[1].find('input')
    pwdConfirmationInput.setValue(PWD)

    const button = wrapper.find('button')
    await button.trigger('submit')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    const data = JSON.parse(`{ "name": "TempAccountCreated", "params": {"emailAddress": "${EMAIL_ADDRESS}"} }`)
    expect(routerPushMock).toHaveBeenCalledWith(data)
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

    const emailAddr = wrapper.findComponent(EmailAddressInput)
    const emailAddrInput = emailAddr.find('input')
    emailAddrInput.setValue(EMAIL_ADDRESS)

    const pwds = wrapper.findAllComponents(PasswordInput)
    const pwdInput = pwds[0].find('input')
    pwdInput.setValue(PWD)
    const pwdConfirmationInput = pwds[1].find('input')
    pwdConfirmationInput.setValue(PWD)

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

  it('does not move TempAccountCreated when password and password confirm are different', async () => {
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

    const emailAddr = wrapper.findComponent(EmailAddressInput)
    const emailAddrInput = emailAddr.find('input')
    emailAddrInput.setValue(EMAIL_ADDRESS)

    const pwds = wrapper.findAllComponents(PasswordInput)
    const pwdInput = pwds[0].find('input')
    pwdInput.setValue(PWD)
    const pwdConfirmationInput = pwds[1].find('input')
    pwdConfirmationInput.setValue(DIFFERENT_PWD)

    const button = wrapper.find('button')
    await button.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays alert message ${Message.NEW_ACCOUNT_CREATION_FAILED} when connection error happened`, async () => {
    const errDetail = 'connection error'
    createTempAccountMock.mockRejectedValue(new Error(errDetail))

    const wrapper = mount(NewAccount, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const emailAddr = wrapper.findComponent(EmailAddressInput)
    const emailAddrInput = emailAddr.find('input')
    emailAddrInput.setValue(EMAIL_ADDRESS)

    const pwds = wrapper.findAllComponents(PasswordInput)
    const pwdInput = pwds[0].find('input')
    pwdInput.setValue(PWD)
    const pwdConfirmationInput = pwds[1].find('input')
    pwdConfirmationInput.setValue(PWD)

    const button = wrapper.find('button')
    await button.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.NEW_ACCOUNT_CREATION_FAILED)
    expect(resultMessage).toContain(errDetail)
  })
})
