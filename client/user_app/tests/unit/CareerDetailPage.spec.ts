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
import { Code } from '@/util/Error'
import { ApiError, ApiErrorResp } from '@/util/ApiError'

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

const career1 = {
  company_name: 'テスト1株式会社',
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

  it('displays AlertMessage when error has happened on refresh', async () => {
    const errDetail = 'connection error'
    refreshMock.mockRejectedValue(new Error(errDetail))
    const resp = GetCareerResp.create(career1)
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

  it(`moves to login if refresh returns ${Code.UNAUTHORIZED}`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    refreshMock.mockResolvedValue(apiErrResp)
    const resp = GetCareerResp.create(career1)
    getCareerFuncMock.mockResolvedValue(resp)
    mount(CareerDetailPage, {
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
    refreshMock.mockResolvedValue(apiErrResp)
    const resp = GetCareerResp.create(career1)
    getCareerFuncMock.mockResolvedValue(resp)
    mount(CareerDetailPage, {
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

  it(`displays ${Message.NO_CAREER_TO_HANDLE_FOUND_MESSAGE} if ${Code.NO_CAREER_TO_HANDLE_FOUND} is returned`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_CAREER_TO_HANDLE_FOUND))
    getCareerFuncMock.mockResolvedValue(apiErrResp)
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
    expect(resultMessage).toContain(Message.NO_CAREER_TO_HANDLE_FOUND_MESSAGE)
    expect(resultMessage).toContain(Code.NO_CAREER_TO_HANDLE_FOUND.toString())
  })

  it('displays AlertMessage when error has happened on getCareer', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const errDetail = 'connection error'
    getCareerFuncMock.mockRejectedValue(new Error(errDetail))
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

  it(`moves to login if getCareer returns ${Code.UNAUTHORIZED}`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    getCareerFuncMock.mockResolvedValue(apiErrResp)
    mount(CareerDetailPage, {
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

  it(`moves to terms-of-use if getCareer returns ${Code.NOT_TERMS_OF_USE_AGREED_YET}`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    getCareerFuncMock.mockResolvedValue(apiErrResp)
    mount(CareerDetailPage, {
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

  it('moves to CareerDeletionConfirmPage if button is clicked', async () => {
    routeParam = '4321'
    refreshMock.mockResolvedValue(RefreshResp.create())
    const resp = GetCareerResp.create(career1)
    getCareerFuncMock.mockResolvedValue(resp)
    const wrapper = mount(CareerDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const button = wrapper.find('[data-test="move-to-career-deletion-confirm-page-button"]')
    await button.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledTimes(1)
    const data = JSON.parse(`{"name": "CareerDeletionConfirmPage", "params": {"career_id": ${routeParam}}}`)
    expect(routerPushMock).toHaveBeenCalledWith(data)
  })
})
