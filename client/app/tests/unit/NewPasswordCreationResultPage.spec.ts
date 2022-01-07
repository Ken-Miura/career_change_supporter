import { mount, RouterLinkStub } from '@vue/test-utils'
import NewPasswordCreationResultPage from '@/views/NewPasswordCreationResultPage.vue'

describe('NewPasswordCreationResultPage.vue', () => {
  it('renders expected message', () => {
    const wrapper = mount(NewPasswordCreationResultPage, {
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

    expect(title.text()).toMatch('まだパスワード変更は完了していません')
    expect(message.text()).toContain('指定されたメールアドレスにメールを送信しました。')
    expect(message.text()).toContain('メールに記載されたURLをクリックし、パスワード変更を完了させて下さい。')
    expect(noteTitle.text()).toMatch('メールが届かない場合')
    expect(noteMessage.text()).toContain('下記の項目についてご確認下さい。')
    expect(noteMessage.text()).toContain('本サイトのドメインのメールの受信が許可されているかどうか')
    expect(noteMessage.text()).toContain('迷惑メールに振り分けられているかどうか、もしくはゴミ箱に入っているかどうか')
    expect(noteMessage.text()).toContain('URL付きのメール受信を許可しているかどうか')
  })
})
