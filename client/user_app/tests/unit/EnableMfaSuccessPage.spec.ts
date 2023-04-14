import { mount, flushPromises, RouterLinkStub } from '@vue/test-utils'
import EnableMfaSuccessPage from '@/views/personalized/EnableMfaSuccessPage.vue'

// このページ自体では使っていないが、依存しているコンポーネントのTheHeaderが使っているのでモック化しておく
const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

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
    routerPushMock.mockClear()
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

  it('displays error message if recovery code is not found', async () => {
    recoveryCodeMock = null
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
    const noRecoveryCodeFoundLabel = wrapper.find('[data-test="no-recovery-code-found-label"]')
    expect(noRecoveryCodeFoundLabel.text()).toContain('リカバリーコードを表示できません')
    const noRecoveryCodeFoundValue = wrapper.find('[data-test="no-recovery-code-found-value"]')
    expect(noRecoveryCodeFoundValue.text()).toContain('リカバリーコードは一度しか表示されません。リカバリーコードをコピー&ペーストして保管していない場合、二段階認証を無効化し、再度有効化する手順を実施して下さい。')
  })
})
