import { mount, RouterLinkStub } from '@vue/test-utils'
import AdminMenuPage from '@/views/personalized/AdminMenuPage.vue'

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

describe('AdminMenuPage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
  })

  it('moves to create-identity-request-list when the button is pushed', async () => {
    const wrapper = mount(AdminMenuPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const button = wrapper.find('[data-test="move-to-create-identity-request-list-page-button"]')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/create-identity-request-list')
  })

  it('moves to update-identity-request-list when the button is pushed', async () => {
    const wrapper = mount(AdminMenuPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const button = wrapper.find('[data-test="move-to-update-identity-request-list-page-button"]')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/update-identity-request-list')
  })

  it('moves to create-career-request-list when the button is pushed', async () => {
    const wrapper = mount(AdminMenuPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const button = wrapper.find('[data-test="move-to-create-career-request-list-page-button"]')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/create-career-request-list')
  })

  it('moves to user-account-search when the button is pushed', async () => {
    const wrapper = mount(AdminMenuPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const button = wrapper.find('[data-test="move-to-user-account-search-page-button"]')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/user-account-search')
  })
})
