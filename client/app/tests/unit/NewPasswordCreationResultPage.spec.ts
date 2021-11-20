import { mount, RouterLinkStub } from '@vue/test-utils'
import NewPasswordCreationResultPage from '@/views/NewPasswordCreationResultPage.vue'

describe('NewPasswordCreationResultPage.vue', () => {
  it('renders message with props.emailAddress when passed', () => {
    const emailAddress = 'test@example.com'
    const wrapper = mount(NewPasswordCreationResultPage, {
      props: { emailAddress },
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const mainTag = wrapper.find('main')
    const h3Tag = mainTag.find('h3')
    expect(h3Tag.text()).toMatch(`${emailAddress}宛にメールを送信しました。メールを確認し、パスワード変更を完了させて下さい（メールが届いていない場合、迷惑メールに振り分けられていないか、もしくは本サイトのドメインのメールの受信が許可されているかご確認下さい）`)
  })

  it('does not render message when no props passed', () => {
    const wrapper = mount(NewPasswordCreationResultPage, {
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
