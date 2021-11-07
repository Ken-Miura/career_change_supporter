import { refresh } from '@/util/refresh/Refresh'
import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import TermsOfUseAgreement from '@/views/personalized/TermsOfUseAgreement.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import TermsOfUse from '@/components/TermsOfUse.vue'
import { agreeTermsOfUse } from '@/util/terms-of-use/AgreeTermsOfUse'

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
})
