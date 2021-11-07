import { refresh } from '@/util/refresh/Refresh'
import { mount, RouterLinkStub } from '@vue/test-utils'
import TermsOfUseAgreement from '@/views/personalized/TermsOfUseAgreement.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import TermsOfUse from '@/components/TermsOfUse.vue'

jest.mock('@/util/refresh/Refresh')
const refreshMock = refresh as jest.MockedFunction<typeof refresh>

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

describe('TermsOfUseAgreement.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    refreshMock.mockReset()
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
})
