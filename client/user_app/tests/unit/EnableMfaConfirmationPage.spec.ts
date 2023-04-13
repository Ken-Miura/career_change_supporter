import { mount, flushPromises, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import EnableMfaConfirmationPage from '@/views/personalized/EnableMfaConfirmationPage.vue'
import { GetTempMfaSecretResp } from '@/util/personalized/enable-mfa-confirmation/GetTempMfaSecretResp'
import { TempMfaSecret } from '@/util/personalized/enable-mfa-confirmation/TempMfaSecret'
import TheHeader from '@/components/TheHeader.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'

const getTempMfaSecretDoneMock = ref(true)
const getTempMfaSecretFuncMock = jest.fn()
jest.mock('@/util/personalized/enable-mfa-confirmation/useGetTempMfaSecret', () => ({
  useGetTempMfaSecret: () => ({
    getTempMfaSecretDone: getTempMfaSecretDoneMock,
    getTempMfaSecretFunc: getTempMfaSecretFuncMock
  })
}))

const postEnableMfaReqDoneMock = ref(true)
const postEnableMfaReqFuncMock = jest.fn()
jest.mock('@/util/personalized/enable-mfa-confirmation/usePostEnableMfaReq', () => ({
  usePostEnableMfaReq: () => ({
    postEnableMfaReqDone: postEnableMfaReqDoneMock,
    postEnableMfaReqFunc: postEnableMfaReqFuncMock
  })
}))

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

const recoveryCodeMock = null as string | null
const storeCommitMock = jest.fn()
jest.mock('vuex', () => ({
  useStore: () => ({
    commit: storeCommitMock,
    state: {
      recoveryCode: recoveryCodeMock
    }
  })
}))

const tempMfaSecret = {
  // base64_encoded_image = 発行者: Issuer、アカウント名: 413、そして下記のbase32_encoded_secretを元に生成されたQRコード
  base64_encoded_image: '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',
  base32_encoded_secret: 'HU7YU2643SZJMWFW5MUOMWNMHSGLA3S6'
} as TempMfaSecret

describe('EnableMfaConfirmationPage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    getTempMfaSecretDoneMock.value = true
    getTempMfaSecretFuncMock.mockReset()
    postEnableMfaReqDoneMock.value = true
    postEnableMfaReqFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader while calling getTempMfaSecret', async () => {
    getTempMfaSecretDoneMock.value = false
    const resp = GetTempMfaSecretResp.create(tempMfaSecret)
    getTempMfaSecretFuncMock.mockResolvedValue(resp)
    const wrapper = mount(EnableMfaConfirmationPage, {
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

  it('has WaitingCircle and TheHeader while calling postEnableMfaReq', async () => {
    postEnableMfaReqDoneMock.value = false
    const resp = GetTempMfaSecretResp.create(tempMfaSecret)
    getTempMfaSecretFuncMock.mockResolvedValue(resp)
    const wrapper = mount(EnableMfaConfirmationPage, {
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
})
