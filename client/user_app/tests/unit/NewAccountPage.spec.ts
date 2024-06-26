import { mount, RouterLinkStub } from '@vue/test-utils'
import NewAccountPage from '@/views/NewAccountPage.vue'
import EmailAddressInput from '@/components/EmailAddressInput.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import PasswordInput from '@/components/PasswordInput.vue'
import { CreateTempAccountResp } from '@/util/temp-account/CreateTempAccountResp'
import { Message } from '@/util/Message'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { nextTick, ref } from 'vue'
import { Code } from '@/util/Error'
import WaitingCircle from '@/components/WaitingCircle.vue'

const createTempAccountDoneMock = ref(true)
const createTempAccountFuncMock = jest.fn()
jest.mock('@/util/temp-account/useCreateTempAccount', () => ({
  useCreateTempAccount: () => ({
    createTempAccountDone: createTempAccountDoneMock,
    createTempAccountFunc: createTempAccountFuncMock
  })
}))

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

describe('NewAccountPage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    createTempAccountDoneMock.value = true
    createTempAccountFuncMock.mockReset()
  })

  it('has one EmailAddressInput, two PasswordInputs and one AlertMessage', () => {
    const wrapper = mount(NewAccountPage, {
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
    const wrapper = mount(NewAccountPage, {
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

  it('displays header and WaitingCircle, no AlertMessage while requesting', async () => {
    createTempAccountDoneMock.value = false
    createTempAccountFuncMock.mockResolvedValue(CreateTempAccountResp.create())

    const wrapper = mount(NewAccountPage, {
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

  it('moves to TempAccountCreationResultPage when email address and password are passed', async () => {
    createTempAccountFuncMock.mockResolvedValue(CreateTempAccountResp.create())

    const wrapper = mount(NewAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const emailAddr = wrapper.findComponent(EmailAddressInput)
    const emailAddrInput = emailAddr.find('input')
    await emailAddrInput.setValue(EMAIL_ADDRESS)

    const pwds = wrapper.findAllComponents(PasswordInput)
    const pwdInput = pwds[0].find('input')
    await pwdInput.setValue(PWD)
    const pwdConfirmationInput = pwds[1].find('input')
    await pwdConfirmationInput.setValue(PWD)

    const button = wrapper.find('button')
    await button.trigger('submit')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/temp-account-creation-result')
  })

  it(`displays alert message ${Message.REACH_TEMP_ACCOUNTS_LIMIT_MESSAGE} when reach new account limit`, async () => {
    const apiErr = ApiError.create(Code.REACH_TEMP_ACCOUNTS_LIMIT)
    const apiErrorResp = ApiErrorResp.create(400, apiErr)
    createTempAccountFuncMock.mockResolvedValue(apiErrorResp)

    const wrapper = mount(NewAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const emailAddr = wrapper.findComponent(EmailAddressInput)
    const emailAddrInput = emailAddr.find('input')
    await emailAddrInput.setValue(EMAIL_ADDRESS)

    const pwds = wrapper.findAllComponents(PasswordInput)
    const pwdInput = pwds[0].find('input')
    await pwdInput.setValue(PWD)
    const pwdConfirmationInput = pwds[1].find('input')
    await pwdConfirmationInput.setValue(PWD)

    const button = wrapper.find('button')
    await button.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.REACH_TEMP_ACCOUNTS_LIMIT_MESSAGE)
    expect(resultMessage).toContain(Code.REACH_TEMP_ACCOUNTS_LIMIT.toString())
  })

  it('does not move TempAccountCreationResultPage when password and password confirm are different', async () => {
    const apiErr = ApiError.create(Code.REACH_TEMP_ACCOUNTS_LIMIT)
    const apiErrorResp = ApiErrorResp.create(400, apiErr)
    createTempAccountFuncMock.mockResolvedValue(apiErrorResp)

    const wrapper = mount(NewAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const emailAddr = wrapper.findComponent(EmailAddressInput)
    const emailAddrInput = emailAddr.find('input')
    await emailAddrInput.setValue(EMAIL_ADDRESS)

    const pwds = wrapper.findAllComponents(PasswordInput)
    const pwdInput = pwds[0].find('input')
    await pwdInput.setValue(PWD)
    const pwdConfirmationInput = pwds[1].find('input')
    await pwdConfirmationInput.setValue(DIFFERENT_PWD)

    const button = wrapper.find('button')
    await button.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays alert message ${Message.TEMP_ACCOUNT_CREATION_FAILED} when connection error happened`, async () => {
    const errDetail = 'connection error'
    createTempAccountFuncMock.mockRejectedValue(new Error(errDetail))

    const wrapper = mount(NewAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const emailAddr = wrapper.findComponent(EmailAddressInput)
    const emailAddrInput = emailAddr.find('input')
    await emailAddrInput.setValue(EMAIL_ADDRESS)

    const pwds = wrapper.findAllComponents(PasswordInput)
    const pwdInput = pwds[0].find('input')
    await pwdInput.setValue(PWD)
    const pwdConfirmationInput = pwds[1].find('input')
    await pwdConfirmationInput.setValue(PWD)

    const button = wrapper.find('button')
    await button.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.TEMP_ACCOUNT_CREATION_FAILED)
    expect(resultMessage).toContain(errDetail)
  })
})
