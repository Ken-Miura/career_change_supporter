import { mount, RouterLinkStub } from '@vue/test-utils'
import PasswordChangeReqPage from '@/views/PasswordChangeReqPage.vue'
import EmailAddressInput from '@/components/EmailAddressInput.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { createPwdChangeReq } from '@/util/password/CreatePwdChangeReq'
import { CreatePwdChangeReqResp } from '@/util/password/CreatePwdChangeReqResp'
import { Message } from '@/util/Message'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { nextTick } from 'vue'
import { Code } from '@/util/Error'

jest.mock('@/util/password/CreatePwdChangeReq')
const createPwdChangeReqMock = createPwdChangeReq as jest.MockedFunction<typeof createPwdChangeReq>

// 参考: https://stackoverflow.com/questions/68763693/vue-routers-injection-fails-during-a-jest-unit-test
const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

const EMAIL_ADDRESS = 'test@example.com'

describe('PasswordChangeReqPage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    createPwdChangeReqMock.mockReset()
  })

  it('has one EmailAddressInput and one AlertMessage', () => {
    const wrapper = mount(PasswordChangeReqPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const emailAddresses = wrapper.findAllComponents(EmailAddressInput)
    expect(emailAddresses.length).toBe(1)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
  })

  it('has AlertMessage with a hidden attribute when created', () => {
    const wrapper = mount(PasswordChangeReqPage, {
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

  it('moves to PasswordChangeReqResultPage when email address is passed', async () => {
    createPwdChangeReqMock.mockResolvedValue(CreatePwdChangeReqResp.create())

    const wrapper = mount(PasswordChangeReqPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const emailAddr = wrapper.findComponent(EmailAddressInput)
    const emailAddrInput = emailAddr.find('input')
    emailAddrInput.setValue(EMAIL_ADDRESS)

    const button = wrapper.find('button')
    await button.trigger('submit')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('password-change-req-result')
  })

  it(`displays alert message ${Message.REACH_PASSWORD_CHANGE_REQ_LIMIT_MESSAGE} when reach password change request limit`, async () => {
    const apiErr = ApiError.create(Code.REACH_PASSWORD_CHANGE_REQ_LIMIT)
    const apiErrorResp = ApiErrorResp.create(400, apiErr)
    createPwdChangeReqMock.mockResolvedValue(apiErrorResp)

    const wrapper = mount(PasswordChangeReqPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const emailAddr = wrapper.findComponent(EmailAddressInput)
    const emailAddrInput = emailAddr.find('input')
    emailAddrInput.setValue(EMAIL_ADDRESS)

    const button = wrapper.find('button')
    await button.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.REACH_PASSWORD_CHANGE_REQ_LIMIT_MESSAGE)
    expect(resultMessage).toContain(Code.REACH_PASSWORD_CHANGE_REQ_LIMIT.toString())
  })

  it(`displays alert message ${Message.PASSWORD_CHANGE_REQUEST_FAILED} when connection error happened`, async () => {
    const errDetail = 'connection error'
    createPwdChangeReqMock.mockRejectedValue(new Error(errDetail))

    const wrapper = mount(PasswordChangeReqPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const emailAddr = wrapper.findComponent(EmailAddressInput)
    const emailAddrInput = emailAddr.find('input')
    emailAddrInput.setValue(EMAIL_ADDRESS)

    const button = wrapper.find('button')
    await button.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.PASSWORD_CHANGE_REQUEST_FAILED)
    expect(resultMessage).toContain(errDetail)
  })
})
