import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import SubmitBankAccountSuccessPage from '@/views/personalized/SubmitBankAccountSuccessPage.vue'
import TheHeader from '@/components/TheHeader.vue'
import { Message } from '@/util/Message'

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

describe('SubmitBankAccountSuccessPage.vue', () => {
  it('has TheHeader', async () => {
    const wrapper = mount(SubmitBankAccountSuccessPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const headers = wrapper.findAllComponents(TheHeader)
    expect(headers.length).toBe(1)
  })

  it(`displays ${Message.SUBMIT_BANK_ACCOUNT_SUCCESS_MESSAGE}`, async () => {
    const wrapper = mount(SubmitBankAccountSuccessPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const message = wrapper.text()
    expect(message).toContain(Message.SUBMIT_BANK_ACCOUNT_SUCCESS_MESSAGE)
  })
})
