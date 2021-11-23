import TheHeader from '@/components/TheHeader.vue'
import { RouterLinkStub, mount } from '@vue/test-utils'

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

describe('TheHeader.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
  })

  it('has one button, one list and one logout handle', () => {
    const wrapper = mount(TheHeader, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const buttons = wrapper.findAll('button')
    expect(buttons.length).toBe(1)
    const list = wrapper.find('[data-test="div"]')
    expect(list.exists)
    const logoutHandle = wrapper.find('[data-test="p"]')
    expect(logoutHandle.exists)
  })
})
