import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import DeleteCareerSuccessPage from '@/views/personalized/DeleteCareerSuccessPage.vue'
import TheHeader from '@/components/TheHeader.vue'
import { Message } from '@/util/Message'

const routeParam = ''
const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRoute: () => ({
    params: {
      career_id: routeParam
    }
  }),
  useRouter: () => ({
    push: routerPushMock
  })
}))

describe('DeleteCareerSuccessPage.vue', () => {
  it('has TheHeader', async () => {
    const wrapper = mount(DeleteCareerSuccessPage, {
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

  it(`displays ${Message.DELETE_CAREER_SUCCESS_MESSAGE}`, async () => {
    const wrapper = mount(DeleteCareerSuccessPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const message = wrapper.text()
    expect(message).toContain(Message.DELETE_CAREER_SUCCESS_MESSAGE)
  })
})
