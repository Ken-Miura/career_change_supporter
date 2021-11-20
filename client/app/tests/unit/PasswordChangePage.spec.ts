import { mount, RouterLinkStub } from '@vue/test-utils'
import PasswordChangePage from '@/views/PasswordChangePage.vue'
import EmailAddressInput from '@/components/EmailAddressInput.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import PasswordInput from '@/components/PasswordInput.vue'
import { createNewPassword } from '@/util/password/CreateNewPassword'
import { CreateNewPasswordResp } from '@/util/password/CreateNewPasswordResp'
import { Message } from '@/util/Message'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { nextTick } from 'vue'
import { Code } from '@/util/Error'

jest.mock('@/util/password/CreateNewPassword')
const createNewPasswordMock = createNewPassword as jest.MockedFunction<typeof createNewPassword>

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

describe('PasswordChangePage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    createNewPasswordMock.mockReset()
  })

  it('has one EmailAddressInput, two PasswordInputs and one AlertMessage', () => {
    const wrapper = mount(PasswordChangePage, {
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
    const wrapper = mount(PasswordChangePage, {
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

  it('moves to NewPasswordCreationResultPage when email address and password are passed', async () => {
    createNewPasswordMock.mockResolvedValue(CreateNewPasswordResp.create(EMAIL_ADDRESS))

    const wrapper = mount(PasswordChangePage, {
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
    const data = JSON.parse(`{ "name": "NewPasswordCreationResultPage", "params": {"emailAddress": "${EMAIL_ADDRESS}"} }`)
    expect(routerPushMock).toHaveBeenCalledWith(data)
  })

  it(`displays alert message ${Message.REACH_NEW_PASSWORDS_LIMIT_MESSAGE} when reach new password limit`, async () => {
    const apiErr = ApiError.create(Code.REACH_NEW_PASSWORDS_LIMIT)
    const apiErrorResp = ApiErrorResp.create(400, apiErr)
    createNewPasswordMock.mockResolvedValue(apiErrorResp)

    const wrapper = mount(PasswordChangePage, {
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
    expect(resultMessage).toContain(Message.REACH_NEW_PASSWORDS_LIMIT_MESSAGE)
    expect(resultMessage).toContain(Code.REACH_NEW_PASSWORDS_LIMIT)
  })

  it('does not move NewPasswordCreationResultPage when password and password confirm are different', async () => {
    const apiErr = ApiError.create(Code.REACH_NEW_PASSWORDS_LIMIT)
    const apiErrorResp = ApiErrorResp.create(400, apiErr)
    createNewPasswordMock.mockResolvedValue(apiErrorResp)

    const wrapper = mount(PasswordChangePage, {
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

  it(`displays alert message ${Message.NEW_PASSWORD_CREATION_FAILED} when connection error happened`, async () => {
    const errDetail = 'connection error'
    createNewPasswordMock.mockRejectedValue(new Error(errDetail))

    const wrapper = mount(PasswordChangePage, {
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
    expect(resultMessage).toContain(Message.NEW_PASSWORD_CREATION_FAILED)
    expect(resultMessage).toContain(errDetail)
  })
})
