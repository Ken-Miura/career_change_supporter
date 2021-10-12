import { mount, RouterLinkStub } from '@vue/test-utils'
import TempAccountCreated from '@/views/TempAccountCreated.vue'

describe('TempAccountCreated.vue', () => {
  it('renders message with props.emailAddress when passed', () => {
    const emailAddress = 'test@example.com'
    const wrapper = mount(TempAccountCreated, {
      props: { emailAddress },
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const mainTag = wrapper.find('main')
    const h3Tag = mainTag.find('h3')
    expect(h3Tag.text()).toMatch(`${emailAddress}宛にメールを送信しました。メールを確認し、新規登録を完了させて下さい。`)
  })

  it('does not render message when no props passed', () => {
    const wrapper = mount(TempAccountCreated, {
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
