import { getPageSize, PAGE_SIZE } from '@/util/PageSize'
import { AnnualInComeInManYenParam, CareerParam, ConsultantSearchParam, FeePerHourInYenParam } from '@/util/personalized/ConsultantSearchParam'
import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import ConsultantListPage from '@/views/personalized/ConsultantListPage.vue'
import { PostConsultantsSearchResp } from '@/util/personalized/consultant-list/PostConsultantsSearchResp'
import { ConsultantsSearchResult } from '@/util/personalized/consultant-list/ConsultantsSearchResult'
import { Message } from '@/util/Message'
import { Code } from '@/util/Error'
import { ApiError, ApiErrorResp } from '@/util/ApiError'

jest.mock('@/util/PageSize')
const getPageSizeMock = getPageSize as jest.MockedFunction<typeof getPageSize>

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

let consultantSearchParamMock = null as ConsultantSearchParam | null
jest.mock('vuex', () => ({
  useStore: () => ({
    state: {
      consultantSearchParam: consultantSearchParamMock
    }
  })
}))

const postConsultantsSearchDoneMock = ref(false)
const postConsultantsSearchFuncMock = jest.fn()
jest.mock('@/util/personalized/consultant-list/usePostConsultantsSearch', () => ({
  usePostConsultantsSearch: () => ({
    postConsultantsSearchDone: postConsultantsSearchDoneMock,
    postConsultantsSearchFunc: postConsultantsSearchFuncMock
  })
}))

describe('ConsultantListPage.vue', () => {
  beforeEach(() => {
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(PAGE_SIZE)
    routerPushMock.mockClear()
    consultantSearchParamMock = {
      career_param: {
        company_name: null,
        department_name: null,
        office: null,
        years_of_service: null,
        employed: null,
        contract_type: null,
        profession: null,
        annual_income_in_man_yen: {
          equal_or_more: null,
          equal_or_less: null
        } as AnnualInComeInManYenParam,
        is_manager: null,
        position_name: null,
        is_new_graduate: null,
        note: null
      } as CareerParam,
      fee_per_hour_in_yen_param: {
        equal_or_more: null,
        equal_or_less: null
      } as FeePerHourInYenParam,
      sort_param: null,
      from: 0,
      size: getPageSize()
    } as ConsultantSearchParam
    postConsultantsSearchDoneMock.value = true
    postConsultantsSearchFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    postConsultantsSearchDoneMock.value = false
    const result = {
      total: 0,
      consultants: []
    } as ConsultantsSearchResult
    const resp = PostConsultantsSearchResp.create(result)
    postConsultantsSearchFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultantListPage, {
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
      total: 0,
      consultants: []
    } as ConsultantsSearchResult
    const resp = PostConsultantsSearchResp.create(result)
    postConsultantsSearchFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultantListPage, {
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
    postConsultantsSearchFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(ConsultantListPage, {
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
    postConsultantsSearchFuncMock.mockResolvedValue(apiErrResp)
    mount(ConsultantListPage, {
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
    postConsultantsSearchFuncMock.mockResolvedValue(apiErrResp)
    mount(ConsultantListPage, {
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

  it(`displays ${Message.NO_CONSULTANT_SEARCH_PARAM_FOUND_MESSAGE} when search param is not passed`, async () => {
    consultantSearchParamMock = null
    const result = {
      total: 0,
      consultants: []
    } as ConsultantsSearchResult
    const resp = PostConsultantsSearchResp.create(result)
    postConsultantsSearchFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultantListPage, {
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
    expect(resultMessage).toContain(Message.NO_CONSULTANT_SEARCH_PARAM_FOUND_MESSAGE)
  })

  it(`displays ${Message.INVALID_COMPANY_NAME_LENGTH_MESSAGE} if ${Code.INVALID_COMPANY_NAME_LENGTH} is returned`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.career_param.company_name = ''
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_COMPANY_NAME_LENGTH))
    postConsultantsSearchFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(ConsultantListPage, {
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
    expect(resultMessage).toContain(Message.INVALID_COMPANY_NAME_LENGTH_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_COMPANY_NAME_LENGTH.toString())
  })

  it(`displays ${Message.ILLEGAL_CHAR_IN_COMPANY_NAME_MESSAGE} if ${Code.ILLEGAL_CHAR_IN_COMPANY_NAME} is returned`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.career_param.company_name = '\' OR 1=1--'
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CHAR_IN_COMPANY_NAME))
    postConsultantsSearchFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(ConsultantListPage, {
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
    expect(resultMessage).toContain(Message.ILLEGAL_CHAR_IN_COMPANY_NAME_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CHAR_IN_COMPANY_NAME.toString())
  })
})
