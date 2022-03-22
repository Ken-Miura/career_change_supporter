import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import CreateIdentityRequestListPage from '@/views/personalized/CreateIdentityRequestListPage.vue'
import { GetCreateIdentityRequestsResp } from '@/util/personalized/create-identity-request-list/GetCreateIdentityRequestsResp'
import { CreateIdentityRequestItem } from '@/util/personalized/create-identity-request-list/CreateIdentityRequestItem'
import AlertMessage from '@/components/AlertMessage.vue'
import { Message } from '@/util/Message'

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

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
})
