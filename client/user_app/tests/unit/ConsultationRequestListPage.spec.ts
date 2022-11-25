import flushPromises from 'flush-promises'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import { mount, RouterLinkStub } from '@vue/test-utils'
import ConsultationRequestListPage from '@/views/personalized/ConsultationRequestListPage.vue'
import { GetConsultationRequestsResp } from '@/util/personalized/consultation-request-list/GetConsultationRequestsResp'
import { ConsultationRequestDescription, ConsultationRequestsResult } from '@/util/personalized/consultation-request-list/ConsultationRequestsResult'

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
})
