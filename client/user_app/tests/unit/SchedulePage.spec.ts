import flushPromises from 'flush-promises'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import { ref } from 'vue'
import { RouterLinkStub, mount } from '@vue/test-utils'
import SchedulePage from '@/views/personalized/SchedulePage.vue'
import { GetConsultationsResp } from '@/util/personalized/schedule/GetConsultationsResp'
import { ConsultationsResult } from '@/util/personalized/schedule/ConsultationsResult'

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

const getConsultationsDoneMock = ref(true)
const getConsultationsFuncMock = jest.fn()
jest.mock('@/util/personalized/schedule/useGetConsultations', () => ({
  useGetConsultations: () => ({
    getConsultationsDone: getConsultationsDoneMock,
    getConsultationsFunc: getConsultationsFuncMock
  })
}))

describe('SchedulePage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    getConsultationsDoneMock.value = true
    getConsultationsFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader while api call finishes', async () => {
    getConsultationsDoneMock.value = false
    const consultationsResult = {
      user_side_consultations: [],
      consultant_side_consultations: []
    } as ConsultationsResult
    const resp = GetConsultationsResp.create(consultationsResult)
    getConsultationsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(SchedulePage, {
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
