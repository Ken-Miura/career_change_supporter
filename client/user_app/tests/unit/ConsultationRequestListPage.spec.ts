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
})
