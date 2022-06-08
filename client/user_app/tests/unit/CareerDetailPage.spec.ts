import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import CareerDetailPage from '@/views/personalized/CareerDetailPage.vue'
import TheHeader from '@/components/TheHeader.vue'
import { DeleteCareerResp } from '@/util/personalized/career-deletion-confirm/DeleteCareerResp'
import { refresh } from '@/util/personalized/refresh/Refresh'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { Message } from '@/util/Message'
import { GetCareerResp } from '@/util/personalized/career-detail/GetCareerResp'
import { Career } from '@/util/personalized/Career'

jest.mock('@/util/personalized/refresh/Refresh')
const refreshMock = refresh as jest.MockedFunction<typeof refresh>

const getCareerDoneMock = ref(true)
const getCareerFuncMock = jest.fn()
jest.mock('@/util/personalized/career-detail/useGetCareer', () => ({
  useGetCareer: () => ({
    getCareerDone: getCareerDoneMock,
    getCareerFunc: getCareerFuncMock
  })
}))

let routeParam = ''
const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRoute: () => ({
    params: {
      career_id: routeParam
    }
  }),
  useRouter: () => ({
    push: routerPushMock
  })
}))

describe('CareerDetailPage.vue', () => {
  beforeEach(() => {
    routeParam = '1'
    refreshMock.mockReset()
    getCareerDoneMock.value = true
    getCareerFuncMock.mockReset()
    routerPushMock.mockClear()
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    getCareerDoneMock.value = false
    const resp = DeleteCareerResp.create()
    getCareerFuncMock.mockResolvedValue(resp)
    const wrapper = mount(CareerDetailPage, {
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
    refreshMock.mockRejectedValue(new Error(errDetail))
    const career = {
      company_name: 'テスト株式会社',
      department_name: null,
      office: null,
      career_start_date: {
        year: 1999,
        month: 4,
        day: 1
      },
      career_end_date: null,
      contract_type: 'regular',
      profession: null,
      annual_income_in_man_yen: null,
      is_manager: false,
      position_name: null,
      is_new_graduate: false,
      note: null
    } as Career
    const resp = GetCareerResp.create(career)
    getCareerFuncMock.mockResolvedValue(resp)
    const wrapper = mount(CareerDetailPage, {
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
