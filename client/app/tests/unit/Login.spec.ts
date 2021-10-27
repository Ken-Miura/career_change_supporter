import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import Login from '@/views/Login.vue'
import { refresh } from '@/util/refresh/Refresh'
import EmailAddress from '@/components/EmailAddress.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import Password from '@/components/Password.vue'

jest.mock('@/util/refresh/Refresh')
const refreshMock = refresh as jest.MockedFunction<typeof refresh>

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

describe('Login.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    refreshMock.mockReset()
  })

  it('has one EmailAddress, one Password and one AlertMessage', () => {
    const wrapper = mount(Login, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const emailAddresses = wrapper.findAllComponents(EmailAddress)
    expect(emailAddresses.length).toBe(1)
    const passwords = wrapper.findAllComponents(Password)
    expect(passwords.length).toBe(1)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
  })

  it('has AlertMessage with a hidden attribute when created', () => {
    const wrapper = mount(Login, {
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

  it('moves to profile when session has already existed', async () => {
    refreshMock.mockResolvedValue('SUCCESS')

    mount(Login, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('profile')
  })

  it('does not move when session has not existed yet', async () => {
    refreshMock.mockResolvedValue('FAILURE')

    mount(Login, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })
})
