import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import SubmitFeePerHourInYenSuccessPage from '@/views/personalized/SubmitFeePerHourInYenSuccessPage.vue'
import TheHeader from '@/components/TheHeader.vue'
import { Message } from '@/util/Message'

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

describe('SubmitFeePerHourInYenSuccessPage.vue', () => {
  it('has TheHeader', async () => {
    const wrapper = mount(SubmitFeePerHourInYenSuccessPage, {
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

  it(`displays ${Message.SUBMIT_FEE_PER_HOUR_IN_YEN_SUCCESS_MESSAGE}`, async () => {
    const wrapper = mount(SubmitFeePerHourInYenSuccessPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const message = wrapper.text()
    expect(message).toContain(Message.SUBMIT_FEE_PER_HOUR_IN_YEN_SUCCESS_MESSAGE)
  })
})
