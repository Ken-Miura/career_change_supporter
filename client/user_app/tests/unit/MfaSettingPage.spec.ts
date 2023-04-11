import { mount, RouterLinkStub, flushPromises } from '@vue/test-utils'
import TheHeader from '@/components/TheHeader.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import MfaSettingPage from '@/views/personalized/MfaSettingPage.vue'
import { ref } from 'vue'
import { refresh } from '@/util/personalized/refresh/Refresh'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { Message } from '@/util/Message'
import { PostTempMfaSecretResp } from '@/util/personalized/mfa-setting/PostTempMfaSecretResp'
import { PostDisableMfaReqResp } from '@/util/personalized/mfa-setting/PostDisableMfaReqResp'

jest.mock('@/util/personalized/refresh/Refresh')
const refreshMock = refresh as jest.MockedFunction<typeof refresh>

const postTempMfaSecretDoneMock = ref(true)
const postTempMfaSecretFuncMock = jest.fn()
jest.mock('@/util/personalized/mfa-setting/usePostTempMfaSecret', () => ({
  usePostTempMfaSecret: () => ({
    postTempMfaSecretDone: postTempMfaSecretDoneMock,
    postTempMfaSecretFunc: postTempMfaSecretFuncMock
  })
}))

const postDisableMfaReqDoneMock = ref(true)
const postDisableMfaReqFuncMock = jest.fn()
jest.mock('@/util/personalized/mfa-setting/usePostDisableMfaReq', () => ({
  usePostDisableMfaReq: () => ({
    postDisableMfaReqDone: postDisableMfaReqDoneMock,
    postDisableMfaReqFunc: postDisableMfaReqFuncMock
  })
}))

let mfaEnabled = 'false'
const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRoute: () => ({
    query: {
      'mfa-enabled': mfaEnabled
    }
  }),
  useRouter: () => ({
    push: routerPushMock
  })
}))

describe('MfaSettingPage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    refreshMock.mockReset()
    postTempMfaSecretDoneMock.value = true
    postTempMfaSecretFuncMock.mockReset()
    postDisableMfaReqDoneMock.value = true
    postDisableMfaReqFuncMock.mockReset()
    mfaEnabled = 'false'
  })

  it('has WaitingCircle and TheHeader while calling postTempMfaSecret', async () => {
    postTempMfaSecretDoneMock.value = false
    refreshMock.mockResolvedValue(RefreshResp.create())
    const wrapper = mount(MfaSettingPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const waitingCircles = wrapper.findAllComponents(WaitingCircle)
    expect(waitingCircles.length).toBe(1)
    const headers = wrapper.findAllComponents(TheHeader)
    expect(headers.length).toBe(1)
    // ユーザーに待ち時間を表すためにWaitingCircleが出ていることが確認できれば十分のため、
    // mainが出ていないことまで確認しない。
  })

  it('has WaitingCircle and TheHeader while calling postDisableMfaReq', async () => {
    postDisableMfaReqDoneMock.value = false
    refreshMock.mockResolvedValue(RefreshResp.create())
    const wrapper = mount(MfaSettingPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const waitingCircles = wrapper.findAllComponents(WaitingCircle)
    expect(waitingCircles.length).toBe(1)
    const headers = wrapper.findAllComponents(TheHeader)
    expect(headers.length).toBe(1)
    // ユーザーに待ち時間を表すためにWaitingCircleが出ていることが確認できれば十分のため、
    // mainが出ていないことまで確認しない。
  })

  it('displays status as 無効 and button as 有効化する', async () => {
    mfaEnabled = 'false'
    refreshMock.mockResolvedValue(RefreshResp.create())
    const wrapper = mount(MfaSettingPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(0)

    const headers = wrapper.findAllComponents(TheHeader)
    expect(headers.length).toBe(1)

    const mfaSettingLabel = wrapper.find('[data-test="mfa-setting-label"]')
    expect(mfaSettingLabel.text()).toContain('二段回認証設定')
    const mfaSettingDescription = wrapper.find('[data-test="mfa-setting-description"]')
    expect(mfaSettingDescription.text()).toContain('二段回認証の設定を変更します。本サービスにおける二段階認証には認証アプリを利用します。二段階認証を有効化するためには、事前にスマートフォンにGoogle Authenticator (iOS版リンク、Android OS版リンク) またはそれに準ずる認証アプリをインストールして下さい。')
    const mfaEnabledLabel = wrapper.find('[data-test="mfa-enabled-label"]')
    expect(mfaEnabledLabel.text()).toContain('現在の二段回認証の設定')
    const mfaEnabledValue = wrapper.find('[data-test="mfa-enabled-value"]')
    expect(mfaEnabledValue.text()).toContain('無効')
    const changeMfaSettingButton = wrapper.find('[data-test="change-mfa-setting-button"]')
    expect(changeMfaSettingButton.text()).toContain('有効化する')
  })

  it('displays status as 有効 and button as 無効化する', async () => {
    mfaEnabled = 'true'
    refreshMock.mockResolvedValue(RefreshResp.create())
    const wrapper = mount(MfaSettingPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(0)

    const headers = wrapper.findAllComponents(TheHeader)
    expect(headers.length).toBe(1)

    const mfaSettingLabel = wrapper.find('[data-test="mfa-setting-label"]')
    expect(mfaSettingLabel.text()).toContain('二段回認証設定')
    const mfaSettingDescription = wrapper.find('[data-test="mfa-setting-description"]')
    expect(mfaSettingDescription.text()).toContain('二段回認証の設定を変更します。本サービスにおける二段階認証には認証アプリを利用します。二段階認証を有効化するためには、事前にスマートフォンにGoogle Authenticator (iOS版リンク、Android OS版リンク) またはそれに準ずる認証アプリをインストールして下さい。')
    const mfaEnabledLabel = wrapper.find('[data-test="mfa-enabled-label"]')
    expect(mfaEnabledLabel.text()).toContain('現在の二段回認証の設定')
    const mfaEnabledValue = wrapper.find('[data-test="mfa-enabled-value"]')
    expect(mfaEnabledValue.text()).toContain('有効')
    const changeMfaSettingButton = wrapper.find('[data-test="change-mfa-setting-button"]')
    expect(changeMfaSettingButton.text()).toContain('無効化する')
  })

  it(`moves to login if ${Code.UNAUTHORIZED} is returned on opening page`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    refreshMock.mockResolvedValue(apiErrResp)
    mount(MfaSettingPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/login')
  })

  it(`moves to terms-of-use if ${Code.NOT_TERMS_OF_USE_AGREED_YET} is returned on opening page`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    refreshMock.mockResolvedValue(apiErrResp)
    mount(MfaSettingPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/terms-of-use')
  })

  it(`displays alert message ${Message.UNEXPECTED_ERR} when connection error happened`, async () => {
    const errDetail = 'connection error'
    refreshMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(MfaSettingPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.UNEXPECTED_ERR)
    expect(resultMessage).toContain(errDetail)
  })

  it('moves enable-mfa-confirmation if 有効化する is clicked', async () => {
    mfaEnabled = 'false'
    refreshMock.mockResolvedValue(RefreshResp.create())
    postTempMfaSecretFuncMock.mockResolvedValue(PostTempMfaSecretResp.create())
    const wrapper = mount(MfaSettingPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const changeMfaSettingButton = wrapper.find('[data-test="change-mfa-setting-button"]')
    await changeMfaSettingButton.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/enable-mfa-confirmation')
  })

  it('moves disable-mfa-success if 無効化する is clicked', async () => {
    mfaEnabled = 'true'
    refreshMock.mockResolvedValue(RefreshResp.create())
    postDisableMfaReqFuncMock.mockResolvedValue(PostDisableMfaReqResp.create())
    const wrapper = mount(MfaSettingPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const changeMfaSettingButton = wrapper.find('[data-test="change-mfa-setting-button"]')
    await changeMfaSettingButton.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/disable-mfa-success')
  })
})
