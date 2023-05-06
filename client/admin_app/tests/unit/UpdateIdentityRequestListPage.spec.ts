import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { nextTick, ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import UpdateIdentityRequestListPage from '@/views/personalized/UpdateIdentityRequestListPage.vue'
import { GetUpdateIdentityRequestsResp } from '@/util/personalized/update-identity-request-list/GetUpdateIdentityRequestsResp'
import { UpdateIdentityRequestItem } from '@/util/personalized/update-identity-request-list/UpdateIdentityRequestItem'
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
const getUpdateIdentityRequestsFuncMock = jest.fn()
jest.mock('@/util/personalized/update-identity-request-list/useGetUpdateIdentityRequests', () => ({
  useGetUpdateIdentityRequests: () => ({
    waitingRequestDone: waitingRequestDoneMock,
    getUpdateIdentityRequestsFunc: getUpdateIdentityRequestsFuncMock
  })
}))

describe('UpdateIdentityRequestListPage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    getNumOfItemsMock.mockReset()
    getNumOfItemsMock.mockReturnValue(NUM_OF_ITEMS)
    waitingRequestDoneMock.value = false
    getUpdateIdentityRequestsFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    const date = new Date(Date.UTC(2022, 0, 1, 23, 59, 59))
    const item = {
      user_account_id: 1,
      name: '佐藤 次郎',
      requested_at: date
    } as UpdateIdentityRequestItem
    const items = [item]
    const resp = GetUpdateIdentityRequestsResp.create(items)
    getUpdateIdentityRequestsFuncMock.mockResolvedValue(resp)
    waitingRequestDoneMock.value = true
    const wrapper = mount(UpdateIdentityRequestListPage, {
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

  it('does not have AlertMessage when created', () => {
    const wrapper = mount(UpdateIdentityRequestListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(0)
  })

  it('displays AlertMessage when error has happened', async () => {
    const errDetail = 'connection error'
    getUpdateIdentityRequestsFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(UpdateIdentityRequestListPage, {
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
    getUpdateIdentityRequestsFuncMock.mockResolvedValue(apiErrResp)
    mount(UpdateIdentityRequestListPage, {
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

  it('disables previous button just after opening page', async () => {
    const date = new Date(Date.UTC(2022, 0, 1, 23, 59, 59))
    const item = {
      user_account_id: 1,
      name: '佐藤 次郎',
      requested_at: date
    } as UpdateIdentityRequestItem
    const items = [item]
    const resp = GetUpdateIdentityRequestsResp.create(items)
    getUpdateIdentityRequestsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(UpdateIdentityRequestListPage, {
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

  it('displays an item just after opening page', async () => {
    const date = new Date(Date.UTC(2022, 0, 1, 23, 59, 59))
    const item = {
      user_account_id: 1,
      name: '佐藤 次郎',
      requested_at: date
    } as UpdateIdentityRequestItem
    const items = [item]
    const resp = GetUpdateIdentityRequestsResp.create(items)
    getUpdateIdentityRequestsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(UpdateIdentityRequestListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const list = wrapper.find('[data-test="list"]')
    // ラベル
    expect(list.text()).toContain('依頼時刻')
    expect(list.text()).toContain('氏名')
    // Item
    expect(list.text()).toContain(`${item.requested_at.getFullYear()}年${(item.requested_at.getMonth() + 1).toString().padStart(2, '0')}月${item.requested_at.getDate().toString().padStart(2, '0')}日${item.requested_at.getHours().toString().padStart(2, '0')}時${item.requested_at.getMinutes().toString().padStart(2, '0')}分${item.requested_at.getSeconds().toString().padStart(2, '0')}秒`)
    expect(list.text()).toContain(item.name)
    // 詳細へのボタン
    expect(list.text()).toContain('詳細を確認する')
  })

  it('moves to UpdateIdentityRequestDetailPage with user_account_id if 詳細を確認する is pushed', async () => {
    const date = new Date(Date.UTC(2022, 0, 1, 23, 59, 59))
    const item = {
      user_account_id: 1,
      name: '佐藤 次郎',
      requested_at: date
    } as UpdateIdentityRequestItem
    const items = [item]
    const resp = GetUpdateIdentityRequestsResp.create(items)
    getUpdateIdentityRequestsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(UpdateIdentityRequestListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const itemsDiv = wrapper.find('[data-test="items"]')
    const button = itemsDiv.find('button')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    const data = JSON.parse(`{"name": "UpdateIdentityRequestDetailPage", "params": {"user_account_id": ${item.user_account_id}}}`)
    expect(routerPushMock).toHaveBeenCalledWith(data)
  })

  it('disables next button if items returned are less than displayable items per page', async () => {
    const date = new Date(Date.UTC(2022, 0, 1, 23, 59, 59))
    const item = {
      user_account_id: 1,
      name: '佐藤 次郎',
      requested_at: date
    } as UpdateIdentityRequestItem
    const items = [item]
    const resp = GetUpdateIdentityRequestsResp.create(items)
    getUpdateIdentityRequestsFuncMock.mockResolvedValue(resp)
    getNumOfItemsMock.mockReset()
    getNumOfItemsMock.mockReturnValue(2)
    const wrapper = mount(UpdateIdentityRequestListPage, {
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
      user_account_id: 1,
      name: '佐藤 次郎',
      requested_at: date1
    } as UpdateIdentityRequestItem
    // 気にするのは順序のみで、date1Utcよりあとであればなんでもよいので適当の数字を足す。
    const date2Utc = date1Utc + 60
    const date2 = new Date(date2Utc)
    const item2 = {
      user_account_id: 2,
      name: '田中 太郎',
      requested_at: date2
    } as UpdateIdentityRequestItem
    const items = [item1, item2]
    const resp = GetUpdateIdentityRequestsResp.create(items)
    getUpdateIdentityRequestsFuncMock.mockResolvedValue(resp)
    getNumOfItemsMock.mockReset()
    getNumOfItemsMock.mockReturnValue(2)
    const wrapper = mount(UpdateIdentityRequestListPage, {
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
      user_account_id: 1,
      name: '佐藤 次郎',
      requested_at: date1
    } as UpdateIdentityRequestItem
    // 気にするのは順序のみで、date1Utcよりあとであればなんでもよいので適当の数字を足す。
    const date2Utc = date1Utc + 60
    const date2 = new Date(date2Utc)
    const item2 = {
      user_account_id: 2,
      name: '田中 太郎',
      requested_at: date2
    } as UpdateIdentityRequestItem
    const date3Utc = date2Utc + 60
    const date3 = new Date(date3Utc)
    const item3 = {
      user_account_id: 3,
      name: '鈴木 圭一',
      requested_at: date3
    } as UpdateIdentityRequestItem
    const items = [item1, item2, item3]
    const resp = GetUpdateIdentityRequestsResp.create(items)
    getUpdateIdentityRequestsFuncMock.mockResolvedValue(resp)
    getNumOfItemsMock.mockReset()
    getNumOfItemsMock.mockReturnValue(2)
    const wrapper = mount(UpdateIdentityRequestListPage, {
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

  it('moves to second page if next is pushed, then moves to first page if previous is pushed', async () => {
    getNumOfItemsMock.mockReset()
    getNumOfItemsMock.mockReturnValue(2)
    const date1Utc = Date.UTC(2022, 0, 1, 23, 59, 59)
    const date1 = new Date(date1Utc)
    const item1 = {
      user_account_id: 1,
      name: '佐藤 次郎',
      requested_at: date1
    } as UpdateIdentityRequestItem
    // 気にするのは順序のみで、date1Utcよりあとであればなんでもよいので適当の数字を足す。
    const date2Utc = date1Utc + 60
    const date2 = new Date(date2Utc)
    const item2 = {
      user_account_id: 2,
      name: '田中 太郎',
      requested_at: date2
    } as UpdateIdentityRequestItem
    const firstPageItems = [item1, item2]
    const resp1 = GetUpdateIdentityRequestsResp.create(firstPageItems)
    getUpdateIdentityRequestsFuncMock.mockResolvedValue(resp1)
    const wrapper = mount(UpdateIdentityRequestListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    // ページ表示直後の確認
    const list1 = wrapper.find('[data-test="list"]')
    // ラベル
    expect(list1.text()).toContain('依頼時刻')
    expect(list1.text()).toContain('氏名')
    // Item
    expect(list1.text()).toContain(`${item1.requested_at.getFullYear()}年${(item1.requested_at.getMonth() + 1).toString().padStart(2, '0')}月${item1.requested_at.getDate().toString().padStart(2, '0')}日${item1.requested_at.getHours().toString().padStart(2, '0')}時${item1.requested_at.getMinutes().toString().padStart(2, '0')}分${item1.requested_at.getSeconds().toString().padStart(2, '0')}秒`)
    expect(list1.text()).toContain(item1.name)
    expect(list1.text()).toContain(`${item2.requested_at.getFullYear()}年${(item2.requested_at.getMonth() + 1).toString().padStart(2, '0')}月${item2.requested_at.getDate().toString().padStart(2, '0')}日${item2.requested_at.getHours().toString().padStart(2, '0')}時${item2.requested_at.getMinutes().toString().padStart(2, '0')}分${item2.requested_at.getSeconds().toString().padStart(2, '0')}秒`)
    expect(list1.text()).toContain(item2.name)
    // 詳細へのボタン
    expect(list1.text()).toContain('詳細を確認する')

    const date3Utc = date2Utc + 60
    const date3 = new Date(date3Utc)
    const item3 = {
      user_account_id: 3,
      name: '鈴木 圭一',
      requested_at: date3
    } as UpdateIdentityRequestItem
    const secondPageItems = [item3]
    getUpdateIdentityRequestsFuncMock.mockReset()
    const resp2 = GetUpdateIdentityRequestsResp.create(secondPageItems)
    getUpdateIdentityRequestsFuncMock.mockResolvedValue(resp2)
    const nextButton = wrapper.find('[data-test="next-button"]')
    await nextButton.trigger('click')
    await flushPromises()
    await nextTick()

    // next押下直後の確認
    const list2 = wrapper.find('[data-test="list"]')
    // ラベル
    expect(list2.text()).toContain('依頼時刻')
    expect(list2.text()).toContain('氏名')
    // Item
    expect(list2.text()).toContain(`${item3.requested_at.getFullYear()}年${(item3.requested_at.getMonth() + 1).toString().padStart(2, '0')}月${item3.requested_at.getDate().toString().padStart(2, '0')}日${item3.requested_at.getHours().toString().padStart(2, '0')}時${item3.requested_at.getMinutes().toString().padStart(2, '0')}分${item3.requested_at.getSeconds().toString().padStart(2, '0')}秒`)
    expect(list2.text()).toContain(item3.name)
    // 詳細へのボタン
    expect(list2.text()).toContain('詳細を確認する')
    // 最終ページに移動したとき、Disabledになっていることを確認
    expect(nextButton.attributes()).toHaveProperty('disabled')

    getUpdateIdentityRequestsFuncMock.mockReset()
    getUpdateIdentityRequestsFuncMock.mockResolvedValue(resp1)
    const prevButton = wrapper.find('[data-test="prev-button"]')
    await prevButton.trigger('click')
    await flushPromises()
    await nextTick()

    // previous押下直後の確認
    const list3 = wrapper.find('[data-test="list"]')
    // ラベル
    expect(list3.text()).toContain('依頼時刻')
    expect(list3.text()).toContain('氏名')
    // Item
    expect(list3.text()).toContain(`${item1.requested_at.getFullYear()}年${(item1.requested_at.getMonth() + 1).toString().padStart(2, '0')}月${item1.requested_at.getDate().toString().padStart(2, '0')}日${item1.requested_at.getHours().toString().padStart(2, '0')}時${item1.requested_at.getMinutes().toString().padStart(2, '0')}分${item1.requested_at.getSeconds().toString().padStart(2, '0')}秒`)
    expect(list3.text()).toContain(item1.name)
    expect(list3.text()).toContain(`${item2.requested_at.getFullYear()}年${(item2.requested_at.getMonth() + 1).toString().padStart(2, '0')}月${item2.requested_at.getDate().toString().padStart(2, '0')}日${item2.requested_at.getHours().toString().padStart(2, '0')}時${item2.requested_at.getMinutes().toString().padStart(2, '0')}分${item2.requested_at.getSeconds().toString().padStart(2, '0')}秒`)
    expect(list3.text()).toContain(item2.name)
    // 詳細へのボタン
    expect(list3.text()).toContain('詳細を確認する')
    // 1ページ目に移動したとき、Disabledになっていることを確認
    expect(prevButton.attributes()).toHaveProperty('disabled')
  })
})
