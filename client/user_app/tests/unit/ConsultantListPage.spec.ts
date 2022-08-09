import { getPageSize, PAGE_SIZE } from '@/util/PageSize'
import { AnnualInComeInManYenParam, CareerParam, ConsultantSearchParam, FeePerHourInYenParam, SortParam } from '@/util/personalized/ConsultantSearchParam'
import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import ConsultantListPage from '@/views/personalized/ConsultantListPage.vue'
import { PostConsultantsSearchResp } from '@/util/personalized/consultant-list/PostConsultantsSearchResp'
import { ConsultantCareerDescription, ConsultantDescription, ConsultantsSearchResult } from '@/util/personalized/consultant-list/ConsultantsSearchResult'
import { Message } from '@/util/Message'
import { Code } from '@/util/Error'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { MAX_ANNUAL_INCOME_IN_MAN_YEN, MIN_ANNUAL_INCOME_IN_MAN_YEN } from '@/util/AnnualIncome'
import { MAX_FEE_PER_HOUR_IN_YEN, MIN_FEE_PER_HOUR_IN_YEN } from '@/util/Fee'

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

  it(`displays ${Message.INVALID_DEPARTMENT_NAME_LENGTH_MESSAGE} if ${Code.INVALID_DEPARTMENT_NAME_LENGTH} is returned`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.career_param.department_name = ''
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_DEPARTMENT_NAME_LENGTH))
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
    expect(resultMessage).toContain(Message.INVALID_DEPARTMENT_NAME_LENGTH_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_DEPARTMENT_NAME_LENGTH.toString())
  })

  it(`displays ${Message.ILLEGAL_CHAR_IN_DEPARTMENT_NAME_MESSAGE} if ${Code.ILLEGAL_CHAR_IN_DEPARTMENT_NAME} is returned`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.career_param.department_name = '\' OR 1=1--'
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CHAR_IN_DEPARTMENT_NAME))
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
    expect(resultMessage).toContain(Message.ILLEGAL_CHAR_IN_DEPARTMENT_NAME_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CHAR_IN_DEPARTMENT_NAME.toString())
  })

  it(`displays ${Message.INVALID_OFFICE_LENGTH_MESSAGE} if ${Code.INVALID_OFFICE_LENGTH} is returned`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.career_param.office = ''
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_OFFICE_LENGTH))
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
    expect(resultMessage).toContain(Message.INVALID_OFFICE_LENGTH_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_OFFICE_LENGTH.toString())
  })

  it(`displays ${Message.ILLEGAL_CHAR_IN_OFFICE_MESSAGE} if ${Code.ILLEGAL_CHAR_IN_OFFICE} is returned`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.career_param.office = '\' OR 1=1--'
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CHAR_IN_OFFICE))
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
    expect(resultMessage).toContain(Message.ILLEGAL_CHAR_IN_OFFICE_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CHAR_IN_OFFICE.toString())
  })

  it(`displays ${Message.ILLEGAL_YEARS_OF_SERVICE_MESSAGE} if ${Code.ILLEGAL_YEARS_OF_SERVICE} is returned`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.career_param.years_of_service = '\' OR 1=1--'
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_YEARS_OF_SERVICE))
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
    expect(resultMessage).toContain(Message.ILLEGAL_YEARS_OF_SERVICE_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_YEARS_OF_SERVICE.toString())
  })

  it(`displays ${Message.ILLEGAL_CONTRACT_TYPE_MESSAGE} if ${Code.ILLEGAL_CONTRACT_TYPE} is returned`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.career_param.contract_type = '\' OR 1=1--'
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CONTRACT_TYPE))
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
    expect(resultMessage).toContain(Message.ILLEGAL_CONTRACT_TYPE_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CONTRACT_TYPE.toString())
  })

  it(`displays ${Message.INVALID_PROFESSION_LENGTH_MESSAGE} if ${Code.INVALID_PROFESSION_LENGTH} is returned`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.career_param.profession = ''
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_PROFESSION_LENGTH))
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
    expect(resultMessage).toContain(Message.INVALID_PROFESSION_LENGTH_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_PROFESSION_LENGTH.toString())
  })

  it(`displays ${Message.ILLEGAL_CHAR_IN_PROFESSION_MESSAGE} if ${Code.ILLEGAL_CHAR_IN_PROFESSION} is returned`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.career_param.profession = '\' OR 1=1--'
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CHAR_IN_PROFESSION))
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
    expect(resultMessage).toContain(Message.ILLEGAL_CHAR_IN_PROFESSION_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CHAR_IN_PROFESSION.toString())
  })

  it(`displays ${Message.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN_MESSAGE} if ${Code.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN} is returned case 1`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.career_param.annual_income_in_man_yen.equal_or_more = MIN_ANNUAL_INCOME_IN_MAN_YEN - 1
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN))
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
    expect(resultMessage).toContain(Message.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN.toString())
  })

  it(`displays ${Message.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN_MESSAGE} if ${Code.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN} is returned case 2`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.career_param.annual_income_in_man_yen.equal_or_less = MAX_ANNUAL_INCOME_IN_MAN_YEN + 1
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN))
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
    expect(resultMessage).toContain(Message.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN.toString())
  })

  it(`displays ${Message.EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_ANNUAL_INCOME_IN_MAN_YEN_MESSAGE} if ${Code.EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_ANNUAL_INCOME_IN_MAN_YEN} is returned`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.career_param.annual_income_in_man_yen.equal_or_more = MIN_ANNUAL_INCOME_IN_MAN_YEN + 1
    consultantSearchParamMock.career_param.annual_income_in_man_yen.equal_or_less = MIN_ANNUAL_INCOME_IN_MAN_YEN
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_ANNUAL_INCOME_IN_MAN_YEN))
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
    expect(resultMessage).toContain(Message.EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_ANNUAL_INCOME_IN_MAN_YEN_MESSAGE)
    expect(resultMessage).toContain(Code.EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_ANNUAL_INCOME_IN_MAN_YEN.toString())
  })

  it(`displays ${Message.INVALID_POSITION_NAME_LENGTH_MESSAGE} if ${Code.INVALID_POSITION_NAME_LENGTH} is returned`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.career_param.position_name = ''
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_POSITION_NAME_LENGTH))
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
    expect(resultMessage).toContain(Message.INVALID_POSITION_NAME_LENGTH_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_POSITION_NAME_LENGTH.toString())
  })

  it(`displays ${Message.ILLEGAL_CHAR_IN_POSITION_NAME_MESSAGE} if ${Code.ILLEGAL_CHAR_IN_POSITION_NAME} is returned`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.career_param.position_name = '\' OR 1=1--'
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CHAR_IN_POSITION_NAME))
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
    expect(resultMessage).toContain(Message.ILLEGAL_CHAR_IN_POSITION_NAME_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CHAR_IN_POSITION_NAME.toString())
  })

  it(`displays ${Message.INVALID_NOTE_LENGTH_MESSAGE} if ${Code.INVALID_NOTE_LENGTH} is returned`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.career_param.note = ''
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_NOTE_LENGTH))
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
    expect(resultMessage).toContain(Message.INVALID_NOTE_LENGTH_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_NOTE_LENGTH.toString())
  })

  it(`displays ${Message.ILLEGAL_CHAR_IN_NOTE_MESSAGE} if ${Code.ILLEGAL_CHAR_IN_NOTE} is returned`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.career_param.note = '\' OR 1=1--'
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CHAR_IN_NOTE))
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
    expect(resultMessage).toContain(Message.ILLEGAL_CHAR_IN_NOTE_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CHAR_IN_NOTE.toString())
  })

  it(`displays ${Message.ILLEGAL_FEE_PER_HOUR_IN_YEN_MESSAGE} if ${Code.ILLEGAL_FEE_PER_HOUR_IN_YEN} is returned case 1`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.fee_per_hour_in_yen_param.equal_or_more = MIN_FEE_PER_HOUR_IN_YEN - 1
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_FEE_PER_HOUR_IN_YEN))
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
    expect(resultMessage).toContain(Message.ILLEGAL_FEE_PER_HOUR_IN_YEN_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_FEE_PER_HOUR_IN_YEN.toString())
  })

  it(`displays ${Message.ILLEGAL_FEE_PER_HOUR_IN_YEN_MESSAGE} if ${Code.ILLEGAL_FEE_PER_HOUR_IN_YEN} is returned case 2`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.fee_per_hour_in_yen_param.equal_or_less = MAX_FEE_PER_HOUR_IN_YEN + 1
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_FEE_PER_HOUR_IN_YEN))
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
    expect(resultMessage).toContain(Message.ILLEGAL_FEE_PER_HOUR_IN_YEN_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_FEE_PER_HOUR_IN_YEN.toString())
  })

  it(`displays ${Message.EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_FEE_PER_HOUR_IN_YEN_MESSAGE} if ${Code.EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_FEE_PER_HOUR_IN_YEN} is returned`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.fee_per_hour_in_yen_param.equal_or_more = MAX_FEE_PER_HOUR_IN_YEN
    consultantSearchParamMock.fee_per_hour_in_yen_param.equal_or_less = MAX_FEE_PER_HOUR_IN_YEN - 1
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_FEE_PER_HOUR_IN_YEN))
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
    expect(resultMessage).toContain(Message.EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_FEE_PER_HOUR_IN_YEN_MESSAGE)
    expect(resultMessage).toContain(Code.EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_FEE_PER_HOUR_IN_YEN.toString())
  })

  it(`displays ${Message.INVALID_SORT_KEY_MESSAGE} if ${Code.INVALID_SORT_KEY} is returned`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.sort_param = {
      key: '\' OR 1=1--',
      order: 'asc'
    } as SortParam
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_SORT_KEY))
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
    expect(resultMessage).toContain(Message.INVALID_SORT_KEY_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_SORT_KEY.toString())
  })

  it(`displays ${Message.INVALID_SORT_ORDER_MESSAGE} if ${Code.INVALID_SORT_ORDER} is returned`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.sort_param = {
      key: 'fee_per_hour_in_yen',
      order: '\' OR 1=1--'
    } as SortParam
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_SORT_ORDER))
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
    expect(resultMessage).toContain(Message.INVALID_SORT_ORDER_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_SORT_ORDER.toString())
  })

  it(`displays ${Message.INVALID_CONSULTANT_SEARCH_PARAM_FROM_MESSAGE} if ${Code.INVALID_CONSULTANT_SEARCH_PARAM_FROM} is returned`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.from = -1
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_CONSULTANT_SEARCH_PARAM_FROM))
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
    expect(resultMessage).toContain(Message.INVALID_CONSULTANT_SEARCH_PARAM_FROM_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_CONSULTANT_SEARCH_PARAM_FROM.toString())
  })

  it(`displays ${Message.INVALID_CONSULTANT_SEARCH_PARAM_SIZE_MESSAGE} if ${Code.INVALID_CONSULTANT_SEARCH_PARAM_SIZE} is returned`, async () => {
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    // モックで返却されるコードが決まっているので、パラメータをしてする必要はない。
    // しかし、どのような値が該当のコードを返すか示すためにエラーになるパラメータを指定しておく
    consultantSearchParamMock.size = getPageSize() + 1
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_CONSULTANT_SEARCH_PARAM_SIZE))
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
    expect(resultMessage).toContain(Message.INVALID_CONSULTANT_SEARCH_PARAM_SIZE_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_CONSULTANT_SEARCH_PARAM_SIZE.toString())
  })

  it(`displays ${Message.NO_IDENTITY_REGISTERED_MESSAGE} if ${Code.NO_IDENTITY_REGISTERED} is returned`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_IDENTITY_REGISTERED))
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
    expect(resultMessage).toContain(Message.NO_IDENTITY_REGISTERED_MESSAGE)
    expect(resultMessage).toContain(Code.NO_IDENTITY_REGISTERED.toString())
  })

  it('displays total 0, initial sort value and has no page move buttons if total is 0', async () => {
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

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    expect(sortValueDiv.text()).toContain('指定なし')

    const consultants = wrapper.findAll('[data-test="consultant"]')
    expect(consultants.length).toBe(0)

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(false)
  })

  it('displays 1 consultant and has no page move buttons if total is 1', async () => {
    const result = {
      total: 1,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              company_name: 'テスト株式会社',
              profession: 'ITエンジニア',
              office: '東北事業所'
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription
      ]
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

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    expect(sortValueDiv.text()).toContain('指定なし')

    const consultants = wrapper.findAll('[data-test="consultant"]')
    expect(consultants.length).toBe(1)
    const consultant = result.consultants[0]
    expect(consultants[0].text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)
    expect(consultants[0].text()).toContain(`相談一回（１時間）の相談料：${consultant.fee_per_hour_in_yen} 円`)
    expect(consultants[0].text()).toContain(`評価：0/5（評価件数：${consultant.num_of_rated} 件）`)
    expect(consultants[0].text()).toContain('職務経歴概要')
    expect(consultants[0].text()).toContain('勤務先名称')
    expect(consultants[0].text()).toContain(`${consultant.careers[0].company_name}`)
    expect(consultants[0].text()).toContain('職種')
    expect(consultants[0].text()).toContain(`${consultant.careers[0].profession}`)
    expect(consultants[0].text()).toContain('勤務地')
    expect(consultants[0].text()).toContain(`${consultant.careers[0].office}`)

    const linkDiv = consultants[0].find('[data-test="consultant-detail-link"]')
    expect(linkDiv.text()).toContain('詳細を確認する')
    const link = linkDiv.findComponent(RouterLinkStub)
    const toValue = `{"name": "ConsultantDetailPage", "params": {"consultant_id": ${consultant.consultant_id}}}`
    expect(link.props().to).toStrictEqual(JSON.parse(toValue))
    expect(link.attributes().target).toBe('_blank')

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(false)
  })

  it('displays 1 consultant with rating and has no page move buttons if total is 1', async () => {
    const result = {
      total: 1,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: 4.5,
          num_of_rated: 325,
          careers: [
            {
              company_name: 'テスト株式会社',
              profession: 'ITエンジニア',
              office: '東北事業所'
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription
      ]
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

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    expect(sortValueDiv.text()).toContain('指定なし')

    const consultants = wrapper.findAll('[data-test="consultant"]')
    expect(consultants.length).toBe(1)
    const consultant = result.consultants[0]
    expect(consultants[0].text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)
    expect(consultants[0].text()).toContain(`相談一回（１時間）の相談料：${consultant.fee_per_hour_in_yen} 円`)
    expect(consultants[0].text()).toContain(`評価：${consultant.rating}/5（評価件数：${consultant.num_of_rated} 件）`)
    expect(consultants[0].text()).toContain('職務経歴概要')
    expect(consultants[0].text()).toContain('勤務先名称')
    expect(consultants[0].text()).toContain(`${consultant.careers[0].company_name}`)
    expect(consultants[0].text()).toContain('職種')
    expect(consultants[0].text()).toContain(`${consultant.careers[0].profession}`)
    expect(consultants[0].text()).toContain('勤務地')
    expect(consultants[0].text()).toContain(`${consultant.careers[0].office}`)

    const linkDiv = consultants[0].find('[data-test="consultant-detail-link"]')
    expect(linkDiv.text()).toContain('詳細を確認する')
    const link = linkDiv.findComponent(RouterLinkStub)
    const toValue = `{"name": "ConsultantDetailPage", "params": {"consultant_id": ${consultant.consultant_id}}}`
    expect(link.props().to).toStrictEqual(JSON.parse(toValue))
    expect(link.attributes().target).toBe('_blank')

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(false)
  })

  it('displays 1 consultant with 8 careers and has no page move buttons if total is 1', async () => {
    const result = {
      total: 1,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: 4.5,
          num_of_rated: 325,
          careers: [
            {
              company_name: 'テスト１株式会社',
              profession: 'ITエンジニア',
              office: '東北事業所'
            } as ConsultantCareerDescription,
            {
              company_name: 'テスト２株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription,
            {
              company_name: 'テスト３株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription,
            {
              company_name: 'テスト４株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription,
            {
              company_name: 'テスト５株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription,
            {
              company_name: 'テスト６株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription,
            {
              company_name: 'テスト７株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription,
            {
              company_name: 'テスト８株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription
      ]
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

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    expect(sortValueDiv.text()).toContain('指定なし')

    const consultants = wrapper.findAll('[data-test="consultant"]')
    expect(consultants.length).toBe(1)
    const consultant = result.consultants[0]
    expect(consultants[0].text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)
    expect(consultants[0].text()).toContain(`相談一回（１時間）の相談料：${consultant.fee_per_hour_in_yen} 円`)
    expect(consultants[0].text()).toContain(`評価：${consultant.rating}/5（評価件数：${consultant.num_of_rated} 件）`)
    expect(consultants[0].text()).toContain('職務経歴概要')
    expect(consultants[0].text()).toContain('職務経歴概要1')
    expect(consultants[0].text()).toContain('勤務先名称')
    expect(consultants[0].text()).toContain(`${consultant.careers[0].company_name}`)
    expect(consultants[0].text()).toContain('職種')
    expect(consultants[0].text()).toContain(`${consultant.careers[0].profession}`)
    expect(consultants[0].text()).toContain('勤務地')
    expect(consultants[0].text()).toContain(`${consultant.careers[0].office}`)
    expect(consultants[0].text()).toContain('職務経歴概要2')
    expect(consultants[0].text()).toContain(`${consultant.careers[1].company_name}`)
    expect(consultants[0].text()).toContain('職務経歴概要3')
    expect(consultants[0].text()).toContain(`${consultant.careers[2].company_name}`)
    expect(consultants[0].text()).toContain('職務経歴概要4')
    expect(consultants[0].text()).toContain(`${consultant.careers[3].company_name}`)
    expect(consultants[0].text()).toContain('職務経歴概要5')
    expect(consultants[0].text()).toContain(`${consultant.careers[4].company_name}`)
    expect(consultants[0].text()).toContain('職務経歴概要6')
    expect(consultants[0].text()).toContain(`${consultant.careers[5].company_name}`)
    expect(consultants[0].text()).toContain('職務経歴概要7')
    expect(consultants[0].text()).toContain(`${consultant.careers[6].company_name}`)
    expect(consultants[0].text()).toContain('職務経歴概要8')
    expect(consultants[0].text()).toContain(`${consultant.careers[7].company_name}`)

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(false)
  })

  it('displays 2 consultants and has no page move buttons if total is 2', async () => {
    const result = {
      total: 2,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: 4.5,
          num_of_rated: 325,
          careers: [
            {
              company_name: 'テスト１株式会社',
              profession: 'ITエンジニア',
              office: '東北事業所'
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription,
        {
          consultant_id: 2,
          fee_per_hour_in_yen: 4000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              company_name: 'テスト２株式会社',
              profession: 'インフラエンジニア',
              office: '九州事業所'
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription
      ]
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

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    expect(sortValueDiv.text()).toContain('指定なし')

    const consultants = wrapper.find('[data-test="consultants-area"]')
    const consultant0 = result.consultants[0]
    expect(consultants.text()).toContain(`コンサルタントID: ${consultant0.consultant_id}`)
    expect(consultants.text()).toContain(`相談一回（１時間）の相談料：${consultant0.fee_per_hour_in_yen} 円`)
    expect(consultants.text()).toContain(`評価：${consultant0.rating}/5（評価件数：${consultant0.num_of_rated} 件）`)
    expect(consultants.text()).toContain('職務経歴概要')
    expect(consultants.text()).toContain('職務経歴概要1')
    expect(consultants.text()).toContain('勤務先名称')
    expect(consultants.text()).toContain(`${consultant0.careers[0].company_name}`)
    expect(consultants.text()).toContain('職種')
    expect(consultants.text()).toContain(`${consultant0.careers[0].profession}`)
    expect(consultants.text()).toContain('勤務地')
    expect(consultants.text()).toContain(`${consultant0.careers[0].office}`)
    const consultant1 = result.consultants[1]
    expect(consultants.text()).toContain(`コンサルタントID: ${consultant1.consultant_id}`)
    expect(consultants.text()).toContain(`相談一回（１時間）の相談料：${consultant1.fee_per_hour_in_yen} 円`)
    expect(consultants.text()).toContain(`評価：0/5（評価件数：${consultant1.num_of_rated} 件）`)
    expect(consultants.text()).toContain(`${consultant1.careers[0].company_name}`)
    expect(consultants.text()).toContain(`${consultant1.careers[0].profession}`)
    expect(consultants.text()).toContain(`${consultant1.careers[0].office}`)

    expect(consultants.text()).toContain('詳細を確認する')
    const links = consultants.findAllComponents(RouterLinkStub)
    expect(links.length).toBe(2)

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(false)
  })

  it('has no page move buttons if total num of consultants is equal to page size', async () => {
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(1)
    const result = {
      total: 1,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: 4.5,
          num_of_rated: 325,
          careers: [
            {
              company_name: 'テスト株式会社',
              profession: 'ITエンジニア',
              office: '東北事業所'
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription
      ]
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

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    expect(sortValueDiv.text()).toContain('指定なし')

    const consultants = wrapper.findAll('[data-test="consultant"]')
    expect(consultants.length).toBe(1)
    const consultant = result.consultants[0]
    expect(consultants[0].text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)
    expect(consultants[0].text()).toContain(`相談一回（１時間）の相談料：${consultant.fee_per_hour_in_yen} 円`)
    expect(consultants[0].text()).toContain(`評価：${consultant.rating}/5（評価件数：${consultant.num_of_rated} 件）`)
    expect(consultants[0].text()).toContain('職務経歴概要')
    expect(consultants[0].text()).toContain('勤務先名称')
    expect(consultants[0].text()).toContain(`${consultant.careers[0].company_name}`)
    expect(consultants[0].text()).toContain('職種')
    expect(consultants[0].text()).toContain(`${consultant.careers[0].profession}`)
    expect(consultants[0].text()).toContain('勤務地')
    expect(consultants[0].text()).toContain(`${consultant.careers[0].office}`)

    const linkDiv = consultants[0].find('[data-test="consultant-detail-link"]')
    expect(linkDiv.text()).toContain('詳細を確認する')
    const link = linkDiv.findComponent(RouterLinkStub)
    const toValue = `{"name": "ConsultantDetailPage", "params": {"consultant_id": ${consultant.consultant_id}}}`
    expect(link.props().to).toStrictEqual(JSON.parse(toValue))
    expect(link.attributes().target).toBe('_blank')

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(false)
  })

  it('has next and last buttons and has no first and prev buttons if total num of consultants is page size or more and on first page case 1', async () => {
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(1)
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    consultantSearchParamMock.size = getPageSize()
    const result = {
      total: 2,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              company_name: 'テスト１株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription
      ]
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

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    expect(sortValueDiv.text()).toContain('指定なし')

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(true)

    const toFirstButton = pageMoveButtons.find('[data-test="to-first-button"]')
    expect(toFirstButton.exists()).toBe(false)
    const toPrevButton = pageMoveButtons.find('[data-test="to-prev-button"]')
    expect(toPrevButton.exists()).toBe(false)

    const zeroButtonDiv = pageMoveButtons.find('[data-test="page-index-0"]')
    expect(zeroButtonDiv.exists()).toBe(true)
    expect(zeroButtonDiv.get('button').classes()).toContain('bg-gray-400')
    const oneButtonDiv = pageMoveButtons.find('[data-test="page-index-1"]')
    expect(oneButtonDiv.exists()).toBe(true)
    expect(oneButtonDiv.get('button').classes()).toContain('bg-gray-600')

    const toNextButton = pageMoveButtons.find('[data-test="to-next-button"]')
    expect(toNextButton.exists()).toBe(true)
    const toLastButton = pageMoveButtons.find('[data-test="to-last-button"]')
    expect(toLastButton.exists()).toBe(true)
  })

  it('has next and last buttons and has no first and prev buttons if total num of consultants is page size or more and on first page case 2', async () => {
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(2)
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    consultantSearchParamMock.size = getPageSize()
    const result = {
      total: 3,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              company_name: 'テスト１株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription,
        {
          consultant_id: 2,
          fee_per_hour_in_yen: 5000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              company_name: 'テスト２株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription
      ]
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

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    expect(sortValueDiv.text()).toContain('指定なし')

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(true)

    const toFirstButton = pageMoveButtons.find('[data-test="to-first-button"]')
    expect(toFirstButton.exists()).toBe(false)
    const toPrevButton = pageMoveButtons.find('[data-test="to-prev-button"]')
    expect(toPrevButton.exists()).toBe(false)

    const zeroButtonDiv = pageMoveButtons.find('[data-test="page-index-0"]')
    expect(zeroButtonDiv.exists()).toBe(true)
    expect(zeroButtonDiv.get('button').classes()).toContain('bg-gray-400')
    const oneButtonDiv = pageMoveButtons.find('[data-test="page-index-1"]')
    expect(oneButtonDiv.exists()).toBe(true)
    expect(oneButtonDiv.get('button').classes()).toContain('bg-gray-600')

    const toNextButton = pageMoveButtons.find('[data-test="to-next-button"]')
    expect(toNextButton.exists()).toBe(true)
    const toLastButton = pageMoveButtons.find('[data-test="to-last-button"]')
    expect(toLastButton.exists()).toBe(true)
  })

  it('has no next and last buttons and has first and prev buttons if user moves last page by next button', async () => {
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(1)
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    consultantSearchParamMock.size = getPageSize()
    const result1 = {
      total: 2,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              company_name: 'テスト１株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription
      ]
    } as ConsultantsSearchResult
    const resp1 = PostConsultantsSearchResp.create(result1)
    postConsultantsSearchFuncMock.mockResolvedValue(resp1)
    const wrapper = mount(ConsultantListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const result2 = {
      total: 2,
      consultants: [
        {
          consultant_id: 2,
          fee_per_hour_in_yen: 4000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              company_name: 'テスト２株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription
      ]
    } as ConsultantsSearchResult
    const resp2 = PostConsultantsSearchResp.create(result2)
    postConsultantsSearchFuncMock.mockResolvedValue(resp2)

    const btns = wrapper.find('[data-test="page-move-buttons"]')
    expect(btns.exists()).toBe(true)
    const btn = btns.find('[data-test="to-next-button"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result2.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    expect(sortValueDiv.text()).toContain('指定なし')

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(true)

    const toFirstButton = pageMoveButtons.find('[data-test="to-first-button"]')
    expect(toFirstButton.exists()).toBe(true)
    const toPrevButton = pageMoveButtons.find('[data-test="to-prev-button"]')
    expect(toPrevButton.exists()).toBe(true)

    const zeroButtonDiv = pageMoveButtons.find('[data-test="page-index-0"]')
    expect(zeroButtonDiv.exists()).toBe(true)
    expect(zeroButtonDiv.get('button').classes()).toContain('bg-gray-600')
    const oneButtonDiv = pageMoveButtons.find('[data-test="page-index-1"]')
    expect(oneButtonDiv.exists()).toBe(true)
    expect(oneButtonDiv.get('button').classes()).toContain('bg-gray-400')

    const toNextButton = pageMoveButtons.find('[data-test="to-next-button"]')
    expect(toNextButton.exists()).toBe(false)
    const toLastButton = pageMoveButtons.find('[data-test="to-last-button"]')
    expect(toLastButton.exists()).toBe(false)
  })

  it('has no next and last buttons and has first and prev buttons if user moves last page by last button', async () => {
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(1)
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    consultantSearchParamMock.size = getPageSize()
    const result1 = {
      total: 2,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              company_name: 'テスト１株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription
      ]
    } as ConsultantsSearchResult
    const resp1 = PostConsultantsSearchResp.create(result1)
    postConsultantsSearchFuncMock.mockResolvedValue(resp1)
    const wrapper = mount(ConsultantListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const result2 = {
      total: 2,
      consultants: [
        {
          consultant_id: 2,
          fee_per_hour_in_yen: 4000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              company_name: 'テスト２株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription
      ]
    } as ConsultantsSearchResult
    const resp2 = PostConsultantsSearchResp.create(result2)
    postConsultantsSearchFuncMock.mockResolvedValue(resp2)

    const btns = wrapper.find('[data-test="page-move-buttons"]')
    expect(btns.exists()).toBe(true)
    const btn = btns.find('[data-test="to-last-button"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result2.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    expect(sortValueDiv.text()).toContain('指定なし')

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(true)

    const toFirstButton = pageMoveButtons.find('[data-test="to-first-button"]')
    expect(toFirstButton.exists()).toBe(true)
    const toPrevButton = pageMoveButtons.find('[data-test="to-prev-button"]')
    expect(toPrevButton.exists()).toBe(true)

    const zeroButtonDiv = pageMoveButtons.find('[data-test="page-index-0"]')
    expect(zeroButtonDiv.exists()).toBe(true)
    expect(zeroButtonDiv.get('button').classes()).toContain('bg-gray-600')
    const oneButtonDiv = pageMoveButtons.find('[data-test="page-index-1"]')
    expect(oneButtonDiv.exists()).toBe(true)
    expect(oneButtonDiv.get('button').classes()).toContain('bg-gray-400')

    const toNextButton = pageMoveButtons.find('[data-test="to-next-button"]')
    expect(toNextButton.exists()).toBe(false)
    const toLastButton = pageMoveButtons.find('[data-test="to-last-button"]')
    expect(toLastButton.exists()).toBe(false)
  })
})
