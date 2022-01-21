import { RouterLinkStub, mount, flushPromises } from '@vue/test-utils'
import { ref } from '@vue/runtime-dom'
import WaitingCircle from '@/components/WaitingCircle.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { GetRewardsResp } from '@/util/personalized/reward/GetRewardsResp'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { Message } from '@/util/Message'
import TheHeader from '@/components/TheHeader.vue'
import RewardPage from '@/views/personalized/RewardPage.vue'

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

const getRewardsDoneMock = ref(false)
const getRewardsFuncMock = jest.fn()
jest.mock('@/util/personalized/reward/useGetRewards', () => ({
  useGetRewards: () => ({
    getRewardsDone: getRewardsDoneMock,
    getRewardsFunc: getRewardsFuncMock
  })
}))

describe('RewardPage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    getRewardsDoneMock.value = false
    getRewardsFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader while api call finishes', async () => {
    const reward = {
      /* eslint-disable camelcase */
      bank_account: null,
      rewards_of_the_month: null,
      latest_two_transfers: []
    /* eslint-enable camelcase */
    }
    const resp = GetRewardsResp.create(reward)
    getRewardsFuncMock.mockResolvedValue(resp)
    getRewardsDoneMock.value = false
    const wrapper = mount(RewardPage, {
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

  it(`displays ${Message.UNEXPECTED_ERR} if unexpected error exists`, async () => {
    const apiErrResp = ApiErrorResp.create(500, ApiError.create(Code.UNEXPECTED_ERR_USER))
    getRewardsFuncMock.mockResolvedValue(apiErrResp)
    getRewardsDoneMock.value = true
    const wrapper = mount(RewardPage, {
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

  it(`displays alert message ${Message.UNEXPECTED_ERR} when connection error happened`, async () => {
    const errDetail = 'connection error'
    getRewardsFuncMock.mockRejectedValue(new Error(errDetail))
    getRewardsDoneMock.value = true
    const wrapper = mount(RewardPage, {
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

  it(`moves to login if ${Code.UNAUTHORIZED} is returned`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    getRewardsFuncMock.mockResolvedValue(apiErrResp)
    getRewardsDoneMock.value = true
    mount(RewardPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('login')
  })

  it(`moves to terms of use if ${Code.NOT_TERMS_OF_USE_AGREED_YET} is returned`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    getRewardsFuncMock.mockResolvedValue(apiErrResp)
    getRewardsDoneMock.value = true
    mount(RewardPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('terms-of-use')
  })

  it('has TheHeader after api call finishes', async () => {
    const reward = {
      /* eslint-disable camelcase */
      bank_account: null,
      rewards_of_the_month: null,
      latest_two_transfers: []
    /* eslint-enable camelcase */
    }
    const resp = GetRewardsResp.create(reward)
    getRewardsFuncMock.mockResolvedValue(resp)
    getRewardsDoneMock.value = true
    const wrapper = mount(RewardPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const headers = wrapper.findAllComponents(TheHeader)
    expect(headers.length).toBe(1)
  })

  it(', if no setting information found, displays that', async () => {
    const reward = {
      /* eslint-disable camelcase */
      bank_account: null,
      rewards_of_the_month: null,
      latest_two_transfers: []
    /* eslint-enable camelcase */
    }
    const resp = GetRewardsResp.create(reward)
    getRewardsFuncMock.mockResolvedValue(resp)
    getRewardsDoneMock.value = true
    const wrapper = mount(RewardPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const noBankAccountSetDiv = wrapper.find('[data-test="no-bank-account-set"]')
    expect(noBankAccountSetDiv.exists)
    const noBankAccountSetMessage = noBankAccountSetDiv.text()
    expect(noBankAccountSetMessage).toContain('報酬の入金口座が設定されていません。')

    const noRewardsOfTheMonthSetDiv = wrapper.find('[data-test="no-rewards-of-the-month-set"]')
    expect(noRewardsOfTheMonthSetDiv.exists)
    const noRewardsOfTheMonthSetMessage = noRewardsOfTheMonthSetDiv.text()
    expect(noRewardsOfTheMonthSetMessage).toContain('まだ相談を受け付けていません。')

    const noLatestTwoTransfersSetDiv = wrapper.find('[data-test="no-latest-two-transfers-set"]')
    expect(noLatestTwoTransfersSetDiv.exists)
    const noLatestTwoTransfersSetMessage = noLatestTwoTransfersSetDiv.text()
    expect(noLatestTwoTransfersSetMessage).toContain('入金情報はありません。')
  })

  it(', if no setting information found, displays that', async () => {
    const reward = {
      /* eslint-disable camelcase */
      bank_account: null,
      rewards_of_the_month: null,
      latest_two_transfers: []
    /* eslint-enable camelcase */
    }
    const resp = GetRewardsResp.create(reward)
    getRewardsFuncMock.mockResolvedValue(resp)
    getRewardsDoneMock.value = true
    const wrapper = mount(RewardPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const noBankAccountSetDiv = wrapper.find('[data-test="no-bank-account-set"]')
    expect(noBankAccountSetDiv.exists)
    const noBankAccountSetMessage = noBankAccountSetDiv.text()
    expect(noBankAccountSetMessage).toContain('報酬の入金口座が設定されていません。')
  })
})
