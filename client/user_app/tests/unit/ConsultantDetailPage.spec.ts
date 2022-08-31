import flushPromises from 'flush-promises'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { RouterLinkStub, mount } from '@vue/test-utils'
import ConsultantDetailPage from '@/views/personalized/ConsultantDetailPage.vue'
import { GetConsultantDetailResp } from '@/util/personalized/consultant-detail/GetConsultantDetailResp'
import { ConsultantDetail } from '@/util/personalized/consultant-detail/ConsultantDetail'
import { ConsultantCareerDetail } from '@/util/personalized/consultant-detail/ConsultantCareerDetail'
import { LESS_THAN_THREE_YEARS } from '@/util/personalized/consultant-detail/YearsOfService'
import { Message } from '@/util/Message'

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

const getConsultantDetailDoneMock = ref(true)
const getConsultantDetailFuncMock = jest.fn()
jest.mock('@/util/personalized/consultant-detail/useGetConsultantDetail', () => ({
  useGetConsultantDetail: () => ({
    getConsultantDetailDone: getConsultantDetailDoneMock,
    getConsultantDetailFunc: getConsultantDetailFuncMock
  })
}))

const consultant1 = {
  consultant_id: 1,
  fee_per_hour_in_yen: 3000,
  rating: null,
  num_of_rated: 0,
  careers: [{
    counsultant_career_detail_id: 1,
    company_name: 'テスト（株）',
    department_name: null,
    office: null,
    years_of_service: LESS_THAN_THREE_YEARS,
    employed: true,
    contract_type: 'regular',
    profession: null,
    annual_income_in_man_yen: null,
    is_manager: false,
    position_name: null,
    is_new_graduate: true,
    note: null
  } as ConsultantCareerDetail]
} as ConsultantDetail

describe('ConsultantDetailPage.vue', () => {
  beforeEach(() => {
    routeParam = '1'
    routerPushMock.mockClear()
    getConsultantDetailDoneMock.value = true
    getConsultantDetailFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    getConsultantDetailDoneMock.value = false
    const resp = GetConsultantDetailResp.create(consultant1)
    getConsultantDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultantDetailPage, {
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
    const resp = GetConsultantDetailResp.create(consultant1)
    getConsultantDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultantDetailPage, {
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

  it('displays AlertMessage when error has happened on opening ConsultantDetailPage', async () => {
    const errDetail = 'connection error'
    getConsultantDetailFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(ConsultantDetailPage, {
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
})
