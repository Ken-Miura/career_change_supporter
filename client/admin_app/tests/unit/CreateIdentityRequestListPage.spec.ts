import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import CreateIdentityRequestListPage from '@/views/personalized/CreateIdentityRequestListPage.vue'
import { GetCreateIdentityRequestsResp } from '@/util/personalized/create-identity-request-list/GetCreateIdentityRequestsResp'
import { CreateIdentityRequestItem } from '@/util/personalized/create-identity-request-list/CreateIdentityRequestItem'
import AlertMessage from '@/components/AlertMessage.vue'
import { Message } from '@/util/Message'
import { Code } from '@/util/Error'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { getNumOfItems, NUM_OF_ITEMS } from '@/util/NumOfItems'

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

jest.mock('@/util/NumOfItems')
const getNumOfItemsMock = getNumOfItems as jest.MockedFunction<typeof getNumOfItems>

const waitingRequestDoneMock = ref(false)
const getCreateIdentityRequestsFuncMock = jest.fn()
jest.mock('@/util/personalized/create-identity-request-list/useGetCreateIdentityRequests', () => ({
  useGetCreateIdentityRequests: () => ({
    waitingRequestDone: waitingRequestDoneMock,
    getCreateIdentityRequestsFunc: getCreateIdentityRequestsFuncMock
  })
}))

describe('CreateIdentityRequestListPage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    getNumOfItemsMock.mockReset()
    getNumOfItemsMock.mockReturnValue(NUM_OF_ITEMS)
    waitingRequestDoneMock.value = false
    getCreateIdentityRequestsFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    const date = new Date(Date.UTC(2022, 0, 1, 23, 59, 59))
    const item = {
      account_id: 1,
      name: '佐藤 次郎',
      requested_at: date
    } as CreateIdentityRequestItem
    const items = [item]
    const resp = GetCreateIdentityRequestsResp.create(items)
    getCreateIdentityRequestsFuncMock.mockResolvedValue(resp)
    waitingRequestDoneMock.value = true
    const wrapper = mount(CreateIdentityRequestListPage, {
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

  it('displays AlertMessage when error has happened', async () => {
    const errDetail = 'connection error'
    getCreateIdentityRequestsFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(CreateIdentityRequestListPage, {
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
    getCreateIdentityRequestsFuncMock.mockResolvedValue(apiErrResp)
    mount(CreateIdentityRequestListPage, {
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

  it('disables previous button just after opening page', async () => {
    const date = new Date(Date.UTC(2022, 0, 1, 23, 59, 59))
    const item = {
      account_id: 1,
      name: '佐藤 次郎',
      requested_at: date
    } as CreateIdentityRequestItem
    const items = [item]
    const resp = GetCreateIdentityRequestsResp.create(items)
    getCreateIdentityRequestsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(CreateIdentityRequestListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const button = wrapper.find('[data-test="prev-button"]')
    expect(button.attributes()).toHaveProperty('disabled')
  })

  it('disables next button if items returned are less than displayable items per page', async () => {
    const date = new Date(Date.UTC(2022, 0, 1, 23, 59, 59))
    const item = {
      account_id: 1,
      name: '佐藤 次郎',
      requested_at: date
    } as CreateIdentityRequestItem
    const items = [item]
    const resp = GetCreateIdentityRequestsResp.create(items)
    getCreateIdentityRequestsFuncMock.mockResolvedValue(resp)
    getNumOfItemsMock.mockReset()
    getNumOfItemsMock.mockReturnValue(2)
    const wrapper = mount(CreateIdentityRequestListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const button = wrapper.find('[data-test="next-button"]')
    expect(button.attributes()).toHaveProperty('disabled')
  })

  it('does not disable next button if items returned are equal to displayable items per page', async () => {
    const date1Utc = Date.UTC(2022, 0, 1, 23, 59, 59)
    const date1 = new Date(date1Utc)
    const item1 = {
      account_id: 1,
      name: '佐藤 次郎',
      requested_at: date1
    } as CreateIdentityRequestItem
    // 気にするのは順序のみで、date1Utcよりあとであればなんでもよいので適当の数字を足す。
    const date2Utc = date1Utc + 60
    const date2 = new Date(date2Utc)
    const item2 = {
      account_id: 1,
      name: '田中 太郎',
      requested_at: date2
    } as CreateIdentityRequestItem
    const items = [item1, item2]
    const resp = GetCreateIdentityRequestsResp.create(items)
    getCreateIdentityRequestsFuncMock.mockResolvedValue(resp)
    getNumOfItemsMock.mockReset()
    getNumOfItemsMock.mockReturnValue(2)
    const wrapper = mount(CreateIdentityRequestListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const button = wrapper.find('[data-test="next-button"]')
    expect(button.attributes()).not.toHaveProperty('disabled')
  })

  // 現在の実装では、取得する際に指定する最大Item数とページに表示する最大Item数が同じため（最大50件サーバから取得し、最大50件を1ページに表示するため）
  // このケースは存在しないが、一応テストしておく
  it('does not disable next button if items returned are greater than displayable items per page', async () => {
    const date1Utc = Date.UTC(2022, 0, 1, 23, 59, 59)
    const date1 = new Date(date1Utc)
    const item1 = {
      account_id: 1,
      name: '佐藤 次郎',
      requested_at: date1
    } as CreateIdentityRequestItem
    // 気にするのは順序のみで、date1Utcよりあとであればなんでもよいので適当の数字を足す。
    const date2Utc = date1Utc + 60
    const date2 = new Date(date2Utc)
    const item2 = {
      account_id: 1,
      name: '田中 太郎',
      requested_at: date2
    } as CreateIdentityRequestItem
    const date3Utc = date2Utc + 60
    const date3 = new Date(date3Utc)
    const item3 = {
      account_id: 3,
      name: '鈴木 圭一',
      requested_at: date3
    } as CreateIdentityRequestItem
    const items = [item1, item2, item3]
    const resp = GetCreateIdentityRequestsResp.create(items)
    getCreateIdentityRequestsFuncMock.mockResolvedValue(resp)
    getNumOfItemsMock.mockReset()
    getNumOfItemsMock.mockReturnValue(2)
    const wrapper = mount(CreateIdentityRequestListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const button = wrapper.find('[data-test="next-button"]')
    expect(button.attributes()).not.toHaveProperty('disabled')
  })
})
