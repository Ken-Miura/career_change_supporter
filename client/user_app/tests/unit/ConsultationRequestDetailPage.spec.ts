import { mount, RouterLinkStub } from '@vue/test-utils'
import flushPromises from 'flush-promises'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import ConsultationRequestDetailPage from '@/views/personalized/ConsultationRequestDetailPage.vue'
import { ref } from 'vue'
import { GetConsultationRequestDetailResp } from '@/util/personalized/consultation-request-detail/GetConsultationRequestDetailResp'
import { ConsultationRequestDetail } from '@/util/personalized/consultation-request-detail/ConsultationRequestDetail'
import { ConsultationDateTime } from '@/util/personalized/ConsultationDateTime'

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

describe('ConsultationRequestDetailPage.vue', () => {
  beforeEach(() => {
    routeParam = '23'
    routerPushMock.mockClear()
    getConsultationRequestDetailDoneMock.value = true
    getConsultationRequestDetailFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    getConsultationRequestDetailDoneMock.value = false
    const result = {
      consultation_req_id: parseInt(routeParam),
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
})
