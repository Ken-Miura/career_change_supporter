import { RouterLinkStub, mount, flushPromises } from '@vue/test-utils'
import ProfilePage from '@/views/personalized/ProfilePage.vue'
import { ref } from '@vue/runtime-dom'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { GetProfileResp } from '@/util/personalized/profile/GetProfileResp'
import { Identity } from '@/util/personalized/profile/Identity'

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
    // mainが出ていないことも確認？
  })
})
