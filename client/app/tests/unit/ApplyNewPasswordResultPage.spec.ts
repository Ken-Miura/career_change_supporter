import { mount, RouterLinkStub } from '@vue/test-utils'
import ApplyNewPasswordResultPage from '@/views/ApplyNewPasswordResultPage.vue'

let message = null as string | null
jest.mock('vuex', () => ({
  useStore: () => ({
    state: {
      applyNewPasswordResultMessage: message
    }
  })
}))

describe('ApplyNewPasswordResultPage.vue', () => {
  beforeEach(() => {
    message = null
  })

  it('just renders message when passed', () => {
    message = 'メッセージ'
    const wrapper = mount(ApplyNewPasswordResultPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const mainTag = wrapper.find('main')
    const h3Tag = mainTag.find('h3')
    expect(h3Tag.text()).toMatch(`${message}`)
  })

  it('does not render message when no props passed', () => {
    const wrapper = mount(ApplyNewPasswordResultPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const mainTag = wrapper.find('main')
    const h3Tag = mainTag.find('h3')
    expect(h3Tag.exists()).toBe(false)
  })
})
