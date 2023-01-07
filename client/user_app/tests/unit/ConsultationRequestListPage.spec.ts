import flushPromises from 'flush-promises'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { mount, RouterLinkStub } from '@vue/test-utils'
import ConsultationRequestListPage from '@/views/personalized/ConsultationRequestListPage.vue'
import { GetConsultationRequestsResp } from '@/util/personalized/consultation-request-list/GetConsultationRequestsResp'
import { ConsultationRequestDescription, ConsultationRequestsResult } from '@/util/personalized/consultation-request-list/ConsultationRequestsResult'
import { Message } from '@/util/Message'
import { Code } from '@/util/Error'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { MAX_NUM_OF_CONSULTATION_REQUESTS } from '@/util/personalized/consultation-request-list/MaxNumOfConsultationRequests'

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

const getConsultationRequestsDoneMock = ref(true)
const getConsultationRequestsFuncMock = jest.fn()
jest.mock('@/util/personalized/consultation-request-list/useGetConsultationRequests', () => ({
  useGetConsultationRequests: () => ({
    getConsultationRequestsDone: getConsultationRequestsDoneMock,
    getConsultationRequestsFunc: getConsultationRequestsFuncMock
  })
}))

describe('ConsultantListPage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    getConsultationRequestsDoneMock.value = true
    getConsultationRequestsFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    getConsultationRequestsDoneMock.value = false
    const result = {
      consultation_requests: [] as ConsultationRequestDescription[]
    } as ConsultationRequestsResult
    const resp = GetConsultationRequestsResp.create(result)
    getConsultationRequestsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestListPage, {
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

  it('has TheHeader, has no AlertMessage and WaitingCircle if request is done successfully', async () => {
    const result = {
      consultation_requests: [] as ConsultationRequestDescription[]
    } as ConsultationRequestsResult
    const resp = GetConsultationRequestsResp.create(result)
    getConsultationRequestsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const headers = wrapper.findAllComponents(TheHeader)
    expect(headers.length).toBe(1)
    const waitingCircles = wrapper.findAllComponents(WaitingCircle)
    expect(waitingCircles.length).toBe(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(0)
  })

  it('displays AlertMessage when error has happened on opening ConsultantListPage', async () => {
    const errDetail = 'connection error'
    getConsultationRequestsFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(ConsultationRequestListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.UNEXPECTED_ERR)
    expect(resultMessage).toContain(errDetail)
  })

  it(`moves to login if refresh returns ${Code.UNAUTHORIZED}`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    getConsultationRequestsFuncMock.mockResolvedValue(apiErrResp)
    mount(ConsultationRequestListPage, {
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

  it(`moves to terms-of-use if refresh returns ${Code.NOT_TERMS_OF_USE_AGREED_YET}`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    getConsultationRequestsFuncMock.mockResolvedValue(apiErrResp)
    mount(ConsultationRequestListPage, {
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

  it('displays "相談申し込みはありません。" when there are no consultation requests', async () => {
    const result = {
      consultation_requests: [] as ConsultationRequestDescription[]
    } as ConsultationRequestsResult
    const resp = GetConsultationRequestsResp.create(result)
    getConsultationRequestsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const consultationReqListLabel = wrapper.find('[data-test="consultation-request-list-label"]')
    expect(consultationReqListLabel.text()).toContain('相談申し込み一覧')
    const consultationReqListDescription = wrapper.find('[data-test="consultation-request-list-description"]')
    expect(consultationReqListDescription.text()).toContain(`相談申し込みの内容を確認し、申し込みの了承または拒否をして下さい。相談申し込みは、最大で${MAX_NUM_OF_CONSULTATION_REQUESTS}件表示されます。`)
    const noConsultationReqFound = wrapper.find('[data-test="no-consultation-request-found"]')
    expect(noConsultationReqFound.text()).toContain('相談申し込みはありません。')
  })

  it('displays 1 consultation req when there is one consultation request', async () => {
    const consultationReqId = 432
    const userId = 5321
    const result = {
      consultation_requests: [
        {
          consultation_req_id: consultationReqId,
          user_account_id: userId
        } as ConsultationRequestDescription
      ] as ConsultationRequestDescription[]
    } as ConsultationRequestsResult
    const resp = GetConsultationRequestsResp.create(result)
    getConsultationRequestsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const consultationReqDesc = wrapper.find(`[data-test="consultation-req-id-${consultationReqId}"]`)
    const consultationReqIdLabel = consultationReqDesc.find('[data-test="consultation-req-id"]')
    expect(consultationReqIdLabel.text()).toContain(`相談申し込み番号: ${consultationReqId}`)
    const userIdLabel = consultationReqDesc.find('[data-test="user-id"]')
    expect(userIdLabel.text()).toContain(`ユーザーID（${userId}）からの相談申し込み`)
  })

  it('moves to ConsultationRequestDetailPage with consultation_request_id when button is pushed', async () => {
    const consultationReqId = 432
    const userId = 5321
    const result = {
      consultation_requests: [
        {
          consultation_req_id: consultationReqId,
          user_account_id: userId
        } as ConsultationRequestDescription
      ] as ConsultationRequestDescription[]
    } as ConsultationRequestsResult
    const resp = GetConsultationRequestsResp.create(result)
    getConsultationRequestsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const consultationReqDesc = wrapper.find(`[data-test="consultation-req-id-${consultationReqId}"]`)
    const consultationReqIdLabel = consultationReqDesc.find('[data-test="consultation-req-id"]')
    expect(consultationReqIdLabel.text()).toContain(`相談申し込み番号: ${consultationReqId}`)
    const userIdLabel = consultationReqDesc.find('[data-test="user-id"]')
    expect(userIdLabel.text()).toContain(`ユーザーID（${userId}）からの相談申し込み`)

    const btn = consultationReqDesc.find('[data-test="move-to-consultation-req-detail-page-btn"]')
    await btn.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    const data = { name: 'ConsultationRequestDetailPage', params: { consultation_req_id: consultationReqId } }
    expect(routerPushMock).toHaveBeenCalledWith(data)
  })

  it('displays 20 consultation reqs when there are 20 consultation requests', async () => {
    const result = {
      consultation_requests: createDummy20ConsultationReqs()
    } as ConsultationRequestsResult
    const resp = GetConsultationRequestsResp.create(result)
    getConsultationRequestsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    for (let i = 0; i < 20; i++) {
      const consultationReqDesc = wrapper.find(`[data-test="consultation-req-id-${i + 1}"]`)
      expect(consultationReqDesc.exists()).toBe(true)
    }
  })
})

function createDummy20ConsultationReqs (): ConsultationRequestDescription[] {
  const result = []
  for (let i = 0; i < 20; i++) {
    result.push({
      consultation_req_id: i + 1,
      user_account_id: i + 1
    } as ConsultationRequestDescription)
  }
  return result
}
