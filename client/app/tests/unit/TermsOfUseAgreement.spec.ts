import { refresh } from '@/util/refresh/Refresh'
import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import TermsOfUseAgreement from '@/views/personalized/TermsOfUseAgreement.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import TermsOfUse from '@/components/TermsOfUse.vue'
import { agreeTermsOfUse } from '@/util/terms-of-use/AgreeTermsOfUse'
import { AgreeTermsOfUseResp } from '@/util/terms-of-use/AgreeTermsOfUseResp'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { Message } from '@/util/Message'
import { nextTick } from 'vue'

jest.mock('@/util/refresh/Refresh')
const refreshMock = refresh as jest.MockedFunction<typeof refresh>

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

jest.mock('@/util/terms-of-use/AgreeTermsOfUse')
const agreeTermsOfUseMock = agreeTermsOfUse as jest.MockedFunction<typeof agreeTermsOfUse>

describe('TermsOfUseAgreement.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    refreshMock.mockReset()
    agreeTermsOfUseMock.mockReset()
  })

  it('has one TermsOfUse and one AlertMessage', () => {
    const wrapper = mount(TermsOfUseAgreement, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const termsOfUses = wrapper.findAllComponents(TermsOfUse)
    expect(termsOfUses.length).toBe(1)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
  })

  it('has AlertMessage with a hidden attribute when created', () => {
    const wrapper = mount(TermsOfUseAgreement, {
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

  it('has AlertMessage with a hidden attribute and does not move when refresh is success', async () => {
    refreshMock.mockResolvedValue('SUCCESS')

    const wrapper = mount(TermsOfUseAgreement, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).toContain('hidden')

    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it('moves to login when refresh is failure', async () => {
    refreshMock.mockResolvedValue('FAILURE')

    mount(TermsOfUseAgreement, {
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

  it('moves to login when connection error happens', async () => {
    const errDetail = 'connection error'
    refreshMock.mockRejectedValue(new Error(errDetail))

    mount(TermsOfUseAgreement, {
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

  it('moves to profile after user agrees terms of use', async () => {
    refreshMock.mockResolvedValue('SUCCESS')
    agreeTermsOfUseMock.mockResolvedValue(AgreeTermsOfUseResp.create())

    const wrapper = mount(TermsOfUseAgreement, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const button = wrapper.find('button')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('profile')
  })

  it('moves to profile when user has already agreed terms of use', async () => {
    refreshMock.mockResolvedValue('SUCCESS')
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ALREADY_AGREED_TERMS_OF_USE))
    agreeTermsOfUseMock.mockResolvedValue(apiErrResp)

    const wrapper = mount(TermsOfUseAgreement, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const button = wrapper.find('button')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('profile')
  })

  it('moves to login when session has already exipired', async () => {
    refreshMock.mockResolvedValue('SUCCESS')
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    agreeTermsOfUseMock.mockResolvedValue(apiErrResp)

    const wrapper = mount(TermsOfUseAgreement, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const button = wrapper.find('button')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('login')
  })

  it(`displays alert message ${Message.UNEXPECTED_ERR} when connection error happens`, async () => {
    refreshMock.mockResolvedValue('SUCCESS')
    const errDetail = 'connection error'
    agreeTermsOfUseMock.mockRejectedValue(new Error(errDetail))

    const wrapper = mount(TermsOfUseAgreement, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const button = wrapper.find('button')
    await button.trigger('click')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.UNEXPECTED_ERR)
  })
})
