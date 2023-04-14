import { mount, flushPromises, RouterLinkStub } from '@vue/test-utils'
import EnableMfaSuccessPage from '@/views/personalized/EnableMfaSuccessPage.vue'

let recoveryCodeMock = null as string | null
jest.mock('vuex', () => ({
  useStore: () => ({
    state: {
      recoveryCode: recoveryCodeMock
    }
  })
}))

describe('EnableMfaSuccessPage.vue', () => {
  beforeEach(() => {
    recoveryCodeMock = null
  })

  it('displays recovery code', async () => {
    recoveryCodeMock = 'c85e1bb9a3bc4df2a14174569f2bc41d'
    const wrapper = mount(EnableMfaSuccessPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const label = wrapper.find('[data-test="label"]')
    expect(label.text()).toContain('二段階認証を有効化しました。')
    const description = wrapper.find('[data-test="description"]')
    expect(description.text()).toContain('認証アプリを含んだ端末を紛失した際に利用するリカバリーコードを下記に記載します。端末の紛失に備えて下記のリカバリーコードをコピー&ペーストし、安全な場所に保管して下さい。')
    const recoveryCode = wrapper.find('[data-test="recovery-code"]')
    expect(recoveryCode.text()).toContain(recoveryCodeMock)
  })
})
