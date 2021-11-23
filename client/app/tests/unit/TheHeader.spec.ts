import TheHeader from '@/components/TheHeader.vue'
import { logout } from '@/util/logout/Logout'
import { RouterLinkStub, mount } from '@vue/test-utils'

jest.mock('@/util/logout/Logout')
const logoutMock = logout as jest.MockedFunction<typeof logout>

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

describe('TheHeader.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    logoutMock.mockReset()
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

  it('has list with a hidden attribute when created', () => {
    const wrapper = mount(TheHeader, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const list = wrapper.find('[data-test="div"]')
    expect(list.exists)
    const classes = list.classes()
    expect(classes).toContain('hidden')
  })

  it('adds/remove hidden attribute on list every time menu button is pushed', async () => {
    const wrapper = mount(TheHeader, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const list = wrapper.find('[data-test="div"]')
    expect(list.exists)
    const firstState = list.classes()
    expect(firstState).toContain('hidden')

    const menuButton = wrapper.find('button')
    await menuButton.trigger('click')

    const secondState = list.classes()
    expect(secondState).not.toContain('hidden')

    await menuButton.trigger('click')

    const thirdState = list.classes()
    expect(thirdState).toContain('hidden')
  })

  it('moves LoginPage when logoutHandle is pushed', async () => {
    const wrapper = mount(TheHeader, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const logoutHandle = wrapper.find('[data-test="p"]')
    await logoutHandle.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('login')
  })

  it('moves LoginPage when connection error occurred', async () => {
    const errDetail = 'connection error'
    logoutMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(TheHeader, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const logoutHandle = wrapper.find('[data-test="p"]')
    await logoutHandle.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('login')
  })
})
