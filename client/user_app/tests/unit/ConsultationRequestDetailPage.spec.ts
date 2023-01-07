import { mount, RouterLinkStub } from '@vue/test-utils'
import flushPromises from 'flush-promises'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import ConsultationRequestDetailPage from '@/views/personalized/ConsultationRequestDetailPage.vue'
import { ref } from 'vue'
import { GetConsultationRequestDetailResp } from '@/util/personalized/consultation-request-detail/GetConsultationRequestDetailResp'
import { ConsultationRequestDetail } from '@/util/personalized/consultation-request-detail/ConsultationRequestDetail'
import { ConsultationDateTime } from '@/util/personalized/ConsultationDateTime'
import { Message } from '@/util/Message'
import { Code } from '@/util/Error'
import { ApiError, ApiErrorResp } from '@/util/ApiError'

let routeParam = ''
const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRoute: () => ({
    params: {
      consultant_id: routeParam
    }
  }),
  useRouter: () => ({
    push: routerPushMock
  })
}))

const getConsultationRequestDetailDoneMock = ref(true)
const getConsultationRequestDetailFuncMock = jest.fn()
jest.mock('@/util/personalized/consultation-request-detail/useGetConsultationRequestDetail', () => ({
  useGetConsultationRequestDetail: () => ({
    getConsultationRequestDetailDone: getConsultationRequestDetailDoneMock,
    getConsultationRequestDetailFunc: getConsultationRequestDetailFuncMock
  })
}))

function createDummyConsultationRequestDetail (consultationReq: number): ConsultationRequestDetail {
  return {
    consultation_req_id: consultationReq,
    user_account_id: 432,
    user_rating: null,
    num_of_rated_of_user: 0,
    fee_per_hour_in_yen: 7000,
    first_candidate_in_jst: {
      year: 2023,
      month: 1,
      day: 14,
      hour: 7
    } as ConsultationDateTime,
    second_candidate_in_jst: {
      year: 2023,
      month: 1,
      day: 14,
      hour: 8
    } as ConsultationDateTime,
    third_candidate_in_jst: {
      year: 2023,
      month: 1,
      day: 14,
      hour: 9
    } as ConsultationDateTime
  } as ConsultationRequestDetail
}

describe('ConsultationRequestDetailPage.vue', () => {
  beforeEach(() => {
    routeParam = '23'
    routerPushMock.mockClear()
    getConsultationRequestDetailDoneMock.value = true
    getConsultationRequestDetailFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    getConsultationRequestDetailDoneMock.value = false
    const result = createDummyConsultationRequestDetail(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
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
    const result = createDummyConsultationRequestDetail(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
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

  it('displays AlertMessage when error has happened on opening ConsultationRequestDetailPage', async () => {
    const errDetail = 'connection error'
    getConsultationRequestDetailFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(ConsultationRequestDetailPage, {
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
    getConsultationRequestDetailFuncMock.mockResolvedValue(apiErrResp)
    mount(ConsultationRequestDetailPage, {
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
    getConsultationRequestDetailFuncMock.mockResolvedValue(apiErrResp)
    mount(ConsultationRequestDetailPage, {
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
