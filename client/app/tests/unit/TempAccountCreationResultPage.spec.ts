import { mount, RouterLinkStub } from '@vue/test-utils'
import TempAccountCreationResultPage from '@/views/TempAccountCreationResultPage.vue'

describe('TempAccountCreationResultPage.vue', () => {
  it('renders message with props.emailAddress when passed', () => {
    const emailAddress = 'test@example.com'
    const wrapper = mount(TempAccountCreationResultPage, {
      props: { emailAddress },
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const title = wrapper.find('[data-test="title"]')
    const message = wrapper.find('[data-test="message"]')
    const noteTitle = wrapper.find('[data-test="note-title"]')
    const noteMessage = wrapper.find('[data-test="note-message"]')

    expect(title.text()).toMatch('まだ新規登録は完了していません')
    expect(message.text()).toContain(`${emailAddress}宛にメールを送信しました。`)
    expect(message.text()).toContain('メールに記載されたURLをクリックし、新規登録を完了させて下さい。')
    expect(noteTitle.text()).toMatch('メールが届かない場合')
    expect(noteMessage.text()).toContain('下記の項目についてご確認下さい。')
    expect(noteMessage.text()).toContain('本サイトのドメインのメールの受信が許可されているかどうか')
    expect(noteMessage.text()).toContain('迷惑メールに振り分けられているかどうか、もしくはゴミ箱に入っているかどうか')
    expect(noteMessage.text()).toContain('URL付きのメール受信を許可しているかどうか')
  })

  it('renders message without props.emailAddress when it is not passed', () => {
    const wrapper = mount(TempAccountCreationResultPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const title = wrapper.find('[data-test="title"]')
    const message = wrapper.find('[data-test="message"]')
    const noteTitle = wrapper.find('[data-test="note-title"]')
    const noteMessage = wrapper.find('[data-test="note-message"]')

    expect(title.text()).toMatch('まだ新規登録は完了していません')
    expect(message.text()).toContain('指定されたメールアドレスにメールを送信しました。')
    expect(message.text()).toContain('メールに記載されたURLをクリックし、新規登録を完了させて下さい。')
    expect(noteTitle.text()).toMatch('メールが届かない場合')
    expect(noteMessage.text()).toContain('下記の項目についてご確認下さい。')
    expect(noteMessage.text()).toContain('本サイトのドメインのメールの受信が許可されているかどうか')
    expect(noteMessage.text()).toContain('迷惑メールに振り分けられているかどうか、もしくはゴミ箱に入っているかどうか')
    expect(noteMessage.text()).toContain('URL付きのメール受信を許可しているかどうか')
  })
})
