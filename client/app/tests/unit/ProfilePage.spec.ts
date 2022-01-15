import { RouterLinkStub, mount, flushPromises } from '@vue/test-utils'
import ProfilePage from '@/views/personalized/ProfilePage.vue'
import { ref } from '@vue/runtime-dom'
import WaitingCircle from '@/components/WaitingCircle.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { GetProfileResp } from '@/util/personalized/profile/GetProfileResp'
import { Identity } from '@/util/personalized/profile/Identity'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { Message } from '@/util/Message'

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

let identityMock = null as Identity | null
const storeCommitMock = jest.fn()
jest.mock('vuex', () => ({
  useStore: () => ({
    commit: storeCommitMock,
    state: {
      identity: identityMock
    }
  })
}))

const getProfileDoneMock = ref(false)
const getProfileFuncMock = jest.fn()
jest.mock('@/util/personalized/profile/useGetProfile', () => ({
  useGetProfile: () => ({
    getProfileDone: getProfileDoneMock,
    getProfileFunc: getProfileFuncMock
  })
}))

describe('ProfilePage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    storeCommitMock.mockClear()
    identityMock = null
    getProfileDoneMock.value = false
    getProfileFuncMock.mockReset()
  })

  it('has WaitingCircle while api call finishes', async () => {
    const profile = {
      /* eslint-disable camelcase */
      email_address: 'test@test.com',
      identity: null,
      careers: [],
      fee_per_hour_in_yen: null
    /* eslint-enable camelcase */
    }
    const resp = GetProfileResp.create(profile)
    getProfileFuncMock.mockResolvedValue(resp)
    getProfileDoneMock.value = false
    const wrapper = mount(ProfilePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const waitingCircles = wrapper.findAllComponents(WaitingCircle)
    expect(waitingCircles.length).toBe(1)
    // ユーザーに待ち時間を表すためにWaitingCircleが出ていることが確認できれば十分のため、
    // mainが出ていないことまで確認しない。
  })

  it(`displays ${Message.UNEXPECTED_ERR} if unexpected error exists`, async () => {
    const apiErrResp = ApiErrorResp.create(500, ApiError.create(Code.UNEXPECTED_ERR_USER))
    getProfileFuncMock.mockResolvedValue(apiErrResp)
    getProfileDoneMock.value = true
    const wrapper = mount(ProfilePage, {
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
    expect(resultMessage).toContain(`${Message.UNEXPECTED_ERR} (${Code.UNEXPECTED_ERR_USER})`)
  })
})
