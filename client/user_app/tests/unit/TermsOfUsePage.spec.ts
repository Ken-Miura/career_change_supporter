import { mount, RouterLinkStub } from '@vue/test-utils'
import TermsOfUsePage from '@/views/personalized/TermsOfUsePage.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import TermsOfUse from '@/components/TermsOfUse.vue'
import { agreeTermsOfUse } from '@/util/personalized/terms-of-use/AgreeTermsOfUse'
import { AgreeTermsOfUseResp } from '@/util/personalized/terms-of-use/AgreeTermsOfUseResp'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { Message } from '@/util/Message'
import { nextTick } from 'vue'

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

jest.mock('@/util/personalized/terms-of-use/AgreeTermsOfUse')
const agreeTermsOfUseMock = agreeTermsOfUse as jest.MockedFunction<typeof agreeTermsOfUse>

describe('TermsOfUsePage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    agreeTermsOfUseMock.mockReset()
  })

  it('has one TermsOfUse and one AlertMessage', () => {
    const wrapper = mount(TermsOfUsePage, {
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
    const wrapper = mount(TermsOfUsePage, {
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

  it('moves to profile after user agrees terms of use', async () => {
    agreeTermsOfUseMock.mockResolvedValue(AgreeTermsOfUseResp.create())

    const wrapper = mount(TermsOfUsePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const area = wrapper.find('[data-test="terms-of-use-agreement-area"]')
    const button = area.find('button')
    expect(button.text()).toContain('同意する')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/profile')
  })

  it('moves to home after user rejects terms of use', async () => {
    agreeTermsOfUseMock.mockResolvedValue(AgreeTermsOfUseResp.create())

    const wrapper = mount(TermsOfUsePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const area = wrapper.find('[data-test="terms-of-use-agreement-area"]')
    const link = area.findComponent(RouterLinkStub)
    expect(link.text()).toContain('同意しない')
    expect(link.props().to).toBe('/')
  })

  it('moves to profile when user has already agreed terms of use', async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ALREADY_AGREED_TERMS_OF_USE))
    agreeTermsOfUseMock.mockResolvedValue(apiErrResp)

    const wrapper = mount(TermsOfUsePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const button = wrapper.find('button')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/profile')
  })

  it('moves to login when session has already exipired', async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    agreeTermsOfUseMock.mockResolvedValue(apiErrResp)

    const wrapper = mount(TermsOfUsePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const button = wrapper.find('button')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/login')
  })

  it(`displays alert message ${Message.UNEXPECTED_ERR} when connection error happens`, async () => {
    const errDetail = 'connection error'
    agreeTermsOfUseMock.mockRejectedValue(new Error(errDetail))

    const wrapper = mount(TermsOfUsePage, {
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
