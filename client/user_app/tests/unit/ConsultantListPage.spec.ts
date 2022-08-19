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
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const consultants = wrapper.find('[data-test="consultants-area"]')
    expect(consultants.text()).not.toContain('コンサルタントID')

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
              consultant_career_id: 0,
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
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const consultant = result.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)
    expect(consultantDiv.text()).toContain(`相談一回（１時間）の相談料：${consultant.fee_per_hour_in_yen} 円`)
    expect(consultantDiv.text()).toContain(`評価：0/5（評価件数：${consultant.num_of_rated} 件）`)
    expect(consultantDiv.text()).toContain('職務経歴概要')
    expect(consultantDiv.text()).toContain('勤務先名称')
    expect(consultantDiv.text()).toContain(`${consultant.careers[0].company_name}`)
    expect(consultantDiv.text()).toContain('職種')
    expect(consultantDiv.text()).toContain(`${consultant.careers[0].profession}`)
    expect(consultantDiv.text()).toContain('勤務地')
    expect(consultantDiv.text()).toContain(`${consultant.careers[0].office}`)

    const linkDiv = consultantDiv.find('[data-test="consultant-detail-link"]')
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
              consultant_career_id: 0,
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
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const consultant = result.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)
    expect(consultantDiv.text()).toContain(`相談一回（１時間）の相談料：${consultant.fee_per_hour_in_yen} 円`)
    expect(consultantDiv.text()).toContain(`評価：${consultant.rating}/5（評価件数：${consultant.num_of_rated} 件）`)
    expect(consultantDiv.text()).toContain('職務経歴概要')
    expect(consultantDiv.text()).toContain('勤務先名称')
    expect(consultantDiv.text()).toContain(`${consultant.careers[0].company_name}`)
    expect(consultantDiv.text()).toContain('職種')
    expect(consultantDiv.text()).toContain(`${consultant.careers[0].profession}`)
    expect(consultantDiv.text()).toContain('勤務地')
    expect(consultantDiv.text()).toContain(`${consultant.careers[0].office}`)

    const linkDiv = consultantDiv.find('[data-test="consultant-detail-link"]')
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
              consultant_career_id: 0,
              company_name: 'テスト１株式会社',
              profession: 'ITエンジニア',
              office: '東北事業所'
            } as ConsultantCareerDescription,
            {
              consultant_career_id: 1,
              company_name: 'テスト２株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription,
            {
              consultant_career_id: 2,
              company_name: 'テスト３株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription,
            {
              consultant_career_id: 3,
              company_name: 'テスト４株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription,
            {
              consultant_career_id: 4,
              company_name: 'テスト５株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription,
            {
              consultant_career_id: 5,
              company_name: 'テスト６株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription,
            {
              consultant_career_id: 6,
              company_name: 'テスト７株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription,
            {
              consultant_career_id: 7,
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
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const consultant = result.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)
    expect(consultantDiv.text()).toContain(`相談一回（１時間）の相談料：${consultant.fee_per_hour_in_yen} 円`)
    expect(consultantDiv.text()).toContain(`評価：${consultant.rating}/5（評価件数：${consultant.num_of_rated} 件）`)
    expect(consultantDiv.text()).toContain('職務経歴概要')
    expect(consultantDiv.text()).toContain('職務経歴概要1')
    expect(consultantDiv.text()).toContain('勤務先名称')
    expect(consultantDiv.text()).toContain(`${consultant.careers[0].company_name}`)
    expect(consultantDiv.text()).toContain('職種')
    expect(consultantDiv.text()).toContain(`${consultant.careers[0].profession}`)
    expect(consultantDiv.text()).toContain('勤務地')
    expect(consultantDiv.text()).toContain(`${consultant.careers[0].office}`)
    expect(consultantDiv.text()).toContain('職務経歴概要2')
    expect(consultantDiv.text()).toContain(`${consultant.careers[1].company_name}`)
    expect(consultantDiv.text()).toContain('職務経歴概要3')
    expect(consultantDiv.text()).toContain(`${consultant.careers[2].company_name}`)
    expect(consultantDiv.text()).toContain('職務経歴概要4')
    expect(consultantDiv.text()).toContain(`${consultant.careers[3].company_name}`)
    expect(consultantDiv.text()).toContain('職務経歴概要5')
    expect(consultantDiv.text()).toContain(`${consultant.careers[4].company_name}`)
    expect(consultantDiv.text()).toContain('職務経歴概要6')
    expect(consultantDiv.text()).toContain(`${consultant.careers[5].company_name}`)
    expect(consultantDiv.text()).toContain('職務経歴概要7')
    expect(consultantDiv.text()).toContain(`${consultant.careers[6].company_name}`)
    expect(consultantDiv.text()).toContain('職務経歴概要8')
    expect(consultantDiv.text()).toContain(`${consultant.careers[7].company_name}`)

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
              consultant_career_id: 0,
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
              consultant_career_id: 0,
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
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const consultant0 = result.consultants[0]
    const consultant0Div = wrapper.find(`[data-test="consultant-id-${consultant0.consultant_id}"]`)
    expect(consultant0Div.exists()).toBe(true)
    expect(consultant0Div.text()).toContain(`コンサルタントID: ${consultant0.consultant_id}`)
    expect(consultant0Div.text()).toContain(`相談一回（１時間）の相談料：${consultant0.fee_per_hour_in_yen} 円`)
    expect(consultant0Div.text()).toContain(`評価：${consultant0.rating}/5（評価件数：${consultant0.num_of_rated} 件）`)
    expect(consultant0Div.text()).toContain('職務経歴概要')
    expect(consultant0Div.text()).toContain('職務経歴概要1')
    expect(consultant0Div.text()).toContain('勤務先名称')
    expect(consultant0Div.text()).toContain(`${consultant0.careers[0].company_name}`)
    expect(consultant0Div.text()).toContain('職種')
    expect(consultant0Div.text()).toContain(`${consultant0.careers[0].profession}`)
    expect(consultant0Div.text()).toContain('勤務地')
    expect(consultant0Div.text()).toContain(`${consultant0.careers[0].office}`)
    const link0Div = consultant0Div.find('[data-test="consultant-detail-link"]')
    expect(link0Div.text()).toContain('詳細を確認する')
    const link0 = link0Div.findComponent(RouterLinkStub)
    const to0Value = `{"name": "ConsultantDetailPage", "params": {"consultant_id": ${consultant0.consultant_id}}}`
    expect(link0.props().to).toStrictEqual(JSON.parse(to0Value))
    expect(link0.attributes().target).toBe('_blank')

    const consultant1 = result.consultants[1]
    const consultant1Div = wrapper.find(`[data-test="consultant-id-${consultant1.consultant_id}"]`)
    expect(consultant1Div.exists()).toBe(true)
    expect(consultant1Div.text()).toContain(`コンサルタントID: ${consultant1.consultant_id}`)
    expect(consultant1Div.text()).toContain(`相談一回（１時間）の相談料：${consultant1.fee_per_hour_in_yen} 円`)
    expect(consultant1Div.text()).toContain(`評価：0/5（評価件数：${consultant1.num_of_rated} 件）`)
    expect(consultant1Div.text()).toContain('職務経歴概要')
    expect(consultant1Div.text()).toContain('職務経歴概要1')
    expect(consultant1Div.text()).toContain('勤務先名称')
    expect(consultant1Div.text()).toContain(`${consultant1.careers[0].company_name}`)
    expect(consultant1Div.text()).toContain('職種')
    expect(consultant1Div.text()).toContain(`${consultant1.careers[0].profession}`)
    expect(consultant1Div.text()).toContain('勤務地')
    expect(consultant1Div.text()).toContain(`${consultant1.careers[0].office}`)
    const link1Div = consultant1Div.find('[data-test="consultant-detail-link"]')
    expect(link1Div.text()).toContain('詳細を確認する')
    const link1 = link1Div.findComponent(RouterLinkStub)
    const to1Value = `{"name": "ConsultantDetailPage", "params": {"consultant_id": ${consultant1.consultant_id}}}`
    expect(link1.props().to).toStrictEqual(JSON.parse(to1Value))
    expect(link1.attributes().target).toBe('_blank')

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
              consultant_career_id: 0,
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
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const consultant = result.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)
    expect(consultantDiv.text()).toContain(`相談一回（１時間）の相談料：${consultant.fee_per_hour_in_yen} 円`)
    expect(consultantDiv.text()).toContain(`評価：${consultant.rating}/5（評価件数：${consultant.num_of_rated} 件）`)
    expect(consultantDiv.text()).toContain('職務経歴概要')
    expect(consultantDiv.text()).toContain('勤務先名称')
    expect(consultantDiv.text()).toContain(`${consultant.careers[0].company_name}`)
    expect(consultantDiv.text()).toContain('職種')
    expect(consultantDiv.text()).toContain(`${consultant.careers[0].profession}`)
    expect(consultantDiv.text()).toContain('勤務地')
    expect(consultantDiv.text()).toContain(`${consultant.careers[0].office}`)
    const linkDiv = consultantDiv.find('[data-test="consultant-detail-link"]')
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
              consultant_career_id: 0,
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
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

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
              consultant_career_id: 0,
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
              consultant_career_id: 1,
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
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

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
              consultant_career_id: 0,
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
              consultant_career_id: 0,
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
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const consultant = result2.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)

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
              consultant_career_id: 0,
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
              consultant_career_id: 0,
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
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const consultant = result2.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)

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

  it('has no next and last buttons and has first and prev buttons if user moves last page by last page index button', async () => {
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
              consultant_career_id: 0,
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
    const btnDiv = btns.find('[data-test="page-index-1"]')
    expect(btnDiv.exists()).toBe(true)
    await btnDiv.get('button').trigger('click')
    await flushPromises()

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result2.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(true)

    const consultant = result2.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)

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

  it('has next, last, first, prev and 5 index buttons case1', async () => {
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(1)
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    consultantSearchParamMock.size = getPageSize()
    const result1 = {
      total: 5,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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
      total: 5,
      consultants: [
        {
          consultant_id: 2,
          fee_per_hour_in_yen: 4000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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
    const btnDiv = btns.find('[data-test="page-index-2"]')
    expect(btnDiv.exists()).toBe(true)
    await btnDiv.get('button').trigger('click')
    await flushPromises()

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result2.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(true)

    const consultant = result2.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)

    const toFirstButton = pageMoveButtons.find('[data-test="to-first-button"]')
    expect(toFirstButton.exists()).toBe(true)
    const toPrevButton = pageMoveButtons.find('[data-test="to-prev-button"]')
    expect(toPrevButton.exists()).toBe(true)

    const zeroButtonDiv = pageMoveButtons.find('[data-test="page-index-0"]')
    expect(zeroButtonDiv.exists()).toBe(true)
    expect(zeroButtonDiv.get('button').classes()).toContain('bg-gray-600')
    const oneButtonDiv = pageMoveButtons.find('[data-test="page-index-1"]')
    expect(oneButtonDiv.exists()).toBe(true)
    expect(oneButtonDiv.get('button').classes()).toContain('bg-gray-600')
    const twoButtonDiv = pageMoveButtons.find('[data-test="page-index-2"]')
    expect(twoButtonDiv.exists()).toBe(true)
    expect(twoButtonDiv.get('button').classes()).toContain('bg-gray-400')
    const threeButtonDiv = pageMoveButtons.find('[data-test="page-index-3"]')
    expect(threeButtonDiv.exists()).toBe(true)
    expect(threeButtonDiv.get('button').classes()).toContain('bg-gray-600')
    const fourButtonDiv = pageMoveButtons.find('[data-test="page-index-4"]')
    expect(fourButtonDiv.exists()).toBe(true)
    expect(fourButtonDiv.get('button').classes()).toContain('bg-gray-600')

    const toNextButton = pageMoveButtons.find('[data-test="to-next-button"]')
    expect(toNextButton.exists()).toBe(true)
    const toLastButton = pageMoveButtons.find('[data-test="to-last-button"]')
    expect(toLastButton.exists()).toBe(true)
  })

  it('has next, last, first, prev and 5 index buttons case2', async () => {
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(1)
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    consultantSearchParamMock.size = getPageSize()
    const result1 = {
      total: 7,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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
      total: 7,
      consultants: [
        {
          consultant_id: 2,
          fee_per_hour_in_yen: 4000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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

    const btns1 = wrapper.find('[data-test="page-move-buttons"]')
    expect(btns1.exists()).toBe(true)
    const btnDiv1 = btns1.find('[data-test="page-index-2"]')
    expect(btnDiv1.exists()).toBe(true)
    await btnDiv1.get('button').trigger('click')
    await flushPromises()

    const result3 = {
      total: 7,
      consultants: [
        {
          consultant_id: 3,
          fee_per_hour_in_yen: 4000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
              company_name: 'テスト３株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription
      ]
    } as ConsultantsSearchResult
    const resp3 = PostConsultantsSearchResp.create(result3)
    postConsultantsSearchFuncMock.mockResolvedValue(resp3)

    const btns2 = wrapper.find('[data-test="page-move-buttons"]')
    expect(btns2.exists()).toBe(true)
    const btnDiv2 = btns1.find('[data-test="page-index-3"]')
    expect(btnDiv2.exists()).toBe(true)
    await btnDiv2.get('button').trigger('click')
    await flushPromises()

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result3.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(true)

    const consultant = result3.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)

    const toFirstButton = pageMoveButtons.find('[data-test="to-first-button"]')
    expect(toFirstButton.exists()).toBe(true)
    const toPrevButton = pageMoveButtons.find('[data-test="to-prev-button"]')
    expect(toPrevButton.exists()).toBe(true)

    const btn1Div = pageMoveButtons.find('[data-test="page-index-1"]')
    expect(btn1Div.exists()).toBe(true)
    expect(btn1Div.get('button').classes()).toContain('bg-gray-600')
    const btn2Div = pageMoveButtons.find('[data-test="page-index-2"]')
    expect(btn2Div.exists()).toBe(true)
    expect(btn2Div.get('button').classes()).toContain('bg-gray-600')
    const btn3Div = pageMoveButtons.find('[data-test="page-index-3"]')
    expect(btn3Div.exists()).toBe(true)
    expect(btn3Div.get('button').classes()).toContain('bg-gray-400')
    const btn4Div = pageMoveButtons.find('[data-test="page-index-4"]')
    expect(btn4Div.exists()).toBe(true)
    expect(btn4Div.get('button').classes()).toContain('bg-gray-600')
    const btn5Div = pageMoveButtons.find('[data-test="page-index-5"]')
    expect(btn5Div.exists()).toBe(true)
    expect(btn5Div.get('button').classes()).toContain('bg-gray-600')

    const toNextButton = pageMoveButtons.find('[data-test="to-next-button"]')
    expect(toNextButton.exists()).toBe(true)
    const toLastButton = pageMoveButtons.find('[data-test="to-last-button"]')
    expect(toLastButton.exists()).toBe(true)
  })

  it('has next, last and 3 index buttons case1', async () => {
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(1)
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    consultantSearchParamMock.size = getPageSize()
    const result1 = {
      total: 3,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result1.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(true)

    const consultant = result1.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)

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
    const twoButtonDiv = pageMoveButtons.find('[data-test="page-index-2"]')
    expect(twoButtonDiv.exists()).toBe(true)
    expect(twoButtonDiv.get('button').classes()).toContain('bg-gray-600')

    const toNextButton = pageMoveButtons.find('[data-test="to-next-button"]')
    expect(toNextButton.exists()).toBe(true)
    const toLastButton = pageMoveButtons.find('[data-test="to-last-button"]')
    expect(toLastButton.exists()).toBe(true)
  })

  it('has next, last and 3 index buttons case2', async () => {
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(1)
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    consultantSearchParamMock.size = getPageSize()
    const result1 = {
      total: 4,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result1.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(true)

    const consultant = result1.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)

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
    const twoButtonDiv = pageMoveButtons.find('[data-test="page-index-2"]')
    expect(twoButtonDiv.exists()).toBe(true)
    expect(twoButtonDiv.get('button').classes()).toContain('bg-gray-600')

    const toNextButton = pageMoveButtons.find('[data-test="to-next-button"]')
    expect(toNextButton.exists()).toBe(true)
    const toLastButton = pageMoveButtons.find('[data-test="to-last-button"]')
    expect(toLastButton.exists()).toBe(true)
  })

  it('moves next index if next is clicked', async () => {
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(1)
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    consultantSearchParamMock.size = getPageSize()
    const result1 = {
      total: 3,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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
      total: 3,
      consultants: [
        {
          consultant_id: 2,
          fee_per_hour_in_yen: 4000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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

    const btns1 = wrapper.find('[data-test="page-move-buttons"]')
    expect(btns1.exists()).toBe(true)
    const btnDiv1 = btns1.find('[data-test="to-next-button"]')
    expect(btnDiv1.exists()).toBe(true)
    await btnDiv1.get('button').trigger('click')
    await flushPromises()

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result2.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(true)

    const consultant = result2.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)

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
    const twoButtonDiv = pageMoveButtons.find('[data-test="page-index-2"]')
    expect(twoButtonDiv.exists()).toBe(true)
    expect(twoButtonDiv.get('button').classes()).toContain('bg-gray-600')

    const toNextButton = pageMoveButtons.find('[data-test="to-next-button"]')
    expect(toNextButton.exists()).toBe(true)
    const toLastButton = pageMoveButtons.find('[data-test="to-last-button"]')
    expect(toLastButton.exists()).toBe(true)
  })

  it('moves last index if last is clicked', async () => {
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(1)
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    consultantSearchParamMock.size = getPageSize()
    const result1 = {
      total: 3,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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
      total: 3,
      consultants: [
        {
          consultant_id: 2,
          fee_per_hour_in_yen: 4000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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

    const btns1 = wrapper.find('[data-test="page-move-buttons"]')
    expect(btns1.exists()).toBe(true)
    const btnDiv1 = btns1.find('[data-test="to-last-button"]')
    expect(btnDiv1.exists()).toBe(true)
    await btnDiv1.get('button').trigger('click')
    await flushPromises()

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result2.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(true)

    const consultant = result2.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)

    const toFirstButton = pageMoveButtons.find('[data-test="to-first-button"]')
    expect(toFirstButton.exists()).toBe(true)
    const toPrevButton = pageMoveButtons.find('[data-test="to-prev-button"]')
    expect(toPrevButton.exists()).toBe(true)

    const zeroButtonDiv = pageMoveButtons.find('[data-test="page-index-0"]')
    expect(zeroButtonDiv.exists()).toBe(true)
    expect(zeroButtonDiv.get('button').classes()).toContain('bg-gray-600')
    const oneButtonDiv = pageMoveButtons.find('[data-test="page-index-1"]')
    expect(oneButtonDiv.exists()).toBe(true)
    expect(oneButtonDiv.get('button').classes()).toContain('bg-gray-600')
    const twoButtonDiv = pageMoveButtons.find('[data-test="page-index-2"]')
    expect(twoButtonDiv.exists()).toBe(true)
    expect(twoButtonDiv.get('button').classes()).toContain('bg-gray-400')

    const toNextButton = pageMoveButtons.find('[data-test="to-next-button"]')
    expect(toNextButton.exists()).toBe(false)
    const toLastButton = pageMoveButtons.find('[data-test="to-last-button"]')
    expect(toLastButton.exists()).toBe(false)
  })

  it('moves previous index if prev is clicked', async () => {
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(1)
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    consultantSearchParamMock.size = getPageSize()
    const result1 = {
      total: 3,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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
      total: 3,
      consultants: [
        {
          consultant_id: 2,
          fee_per_hour_in_yen: 4000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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

    const btns1 = wrapper.find('[data-test="page-move-buttons"]')
    expect(btns1.exists()).toBe(true)
    const btnDiv1 = btns1.find('[data-test="to-last-button"]')
    expect(btnDiv1.exists()).toBe(true)
    await btnDiv1.get('button').trigger('click')
    await flushPromises()

    const result3 = {
      total: 3,
      consultants: [
        {
          consultant_id: 3,
          fee_per_hour_in_yen: 4000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
              company_name: 'テスト３株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription
      ]
    } as ConsultantsSearchResult
    const resp3 = PostConsultantsSearchResp.create(result3)
    postConsultantsSearchFuncMock.mockResolvedValue(resp3)

    const btns2 = wrapper.find('[data-test="page-move-buttons"]')
    expect(btns2.exists()).toBe(true)
    const btnDiv2 = btns2.find('[data-test="to-prev-button"]')
    expect(btnDiv2.exists()).toBe(true)
    await btnDiv2.get('button').trigger('click')
    await flushPromises()

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result3.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(true)

    const consultant = result3.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)

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
    const twoButtonDiv = pageMoveButtons.find('[data-test="page-index-2"]')
    expect(twoButtonDiv.exists()).toBe(true)
    expect(twoButtonDiv.get('button').classes()).toContain('bg-gray-600')

    const toNextButton = pageMoveButtons.find('[data-test="to-next-button"]')
    expect(toNextButton.exists()).toBe(true)
    const toLastButton = pageMoveButtons.find('[data-test="to-last-button"]')
    expect(toLastButton.exists()).toBe(true)
  })

  it('moves first index if first is clicked', async () => {
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(1)
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    consultantSearchParamMock.size = getPageSize()
    const result1 = {
      total: 3,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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
      total: 3,
      consultants: [
        {
          consultant_id: 2,
          fee_per_hour_in_yen: 4000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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

    const btns1 = wrapper.find('[data-test="page-move-buttons"]')
    expect(btns1.exists()).toBe(true)
    const btnDiv1 = btns1.find('[data-test="to-last-button"]')
    expect(btnDiv1.exists()).toBe(true)
    await btnDiv1.get('button').trigger('click')
    await flushPromises()

    const result3 = {
      total: 3,
      consultants: [
        {
          consultant_id: 3,
          fee_per_hour_in_yen: 4000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
              company_name: 'テスト３株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription
      ]
    } as ConsultantsSearchResult
    const resp3 = PostConsultantsSearchResp.create(result3)
    postConsultantsSearchFuncMock.mockResolvedValue(resp3)

    const btns2 = wrapper.find('[data-test="page-move-buttons"]')
    expect(btns2.exists()).toBe(true)
    const btnDiv2 = btns2.find('[data-test="to-first-button"]')
    expect(btnDiv2.exists()).toBe(true)
    await btnDiv2.get('button').trigger('click')
    await flushPromises()

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result3.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(true)

    const consultant = result3.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)

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
    const twoButtonDiv = pageMoveButtons.find('[data-test="page-index-2"]')
    expect(twoButtonDiv.exists()).toBe(true)
    expect(twoButtonDiv.get('button').classes()).toContain('bg-gray-600')

    const toNextButton = pageMoveButtons.find('[data-test="to-next-button"]')
    expect(toNextButton.exists()).toBe(true)
    const toLastButton = pageMoveButtons.find('[data-test="to-last-button"]')
    expect(toLastButton.exists()).toBe(true)
  })

  it('has prev, first and 3 index button case 1', async () => {
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(1)
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    consultantSearchParamMock.size = getPageSize()
    const result1 = {
      total: 3,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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
      total: 3,
      consultants: [
        {
          consultant_id: 2,
          fee_per_hour_in_yen: 4000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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

    const btns1 = wrapper.find('[data-test="page-move-buttons"]')
    expect(btns1.exists()).toBe(true)
    const btnDiv1 = btns1.find('[data-test="to-last-button"]')
    expect(btnDiv1.exists()).toBe(true)
    await btnDiv1.get('button').trigger('click')
    await flushPromises()

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result2.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(true)

    const consultant = result2.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)

    const toFirstButton = pageMoveButtons.find('[data-test="to-first-button"]')
    expect(toFirstButton.exists()).toBe(true)
    const toPrevButton = pageMoveButtons.find('[data-test="to-prev-button"]')
    expect(toPrevButton.exists()).toBe(true)

    const zeroButtonDiv = pageMoveButtons.find('[data-test="page-index-0"]')
    expect(zeroButtonDiv.exists()).toBe(true)
    expect(zeroButtonDiv.get('button').classes()).toContain('bg-gray-600')
    const oneButtonDiv = pageMoveButtons.find('[data-test="page-index-1"]')
    expect(oneButtonDiv.exists()).toBe(true)
    expect(oneButtonDiv.get('button').classes()).toContain('bg-gray-600')
    const twoButtonDiv = pageMoveButtons.find('[data-test="page-index-2"]')
    expect(twoButtonDiv.exists()).toBe(true)
    expect(twoButtonDiv.get('button').classes()).toContain('bg-gray-400')

    const toNextButton = pageMoveButtons.find('[data-test="to-next-button"]')
    expect(toNextButton.exists()).toBe(false)
    const toLastButton = pageMoveButtons.find('[data-test="to-last-button"]')
    expect(toLastButton.exists()).toBe(false)
  })

  it('has prev, first and 3 index button case 2', async () => {
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(1)
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    consultantSearchParamMock.size = getPageSize()
    const result1 = {
      total: 4,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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
      total: 4,
      consultants: [
        {
          consultant_id: 2,
          fee_per_hour_in_yen: 4000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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

    const btns1 = wrapper.find('[data-test="page-move-buttons"]')
    expect(btns1.exists()).toBe(true)
    const btnDiv1 = btns1.find('[data-test="to-last-button"]')
    expect(btnDiv1.exists()).toBe(true)
    await btnDiv1.get('button').trigger('click')
    await flushPromises()

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result2.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(true)

    const consultant = result2.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)

    const toFirstButton = pageMoveButtons.find('[data-test="to-first-button"]')
    expect(toFirstButton.exists()).toBe(true)
    const toPrevButton = pageMoveButtons.find('[data-test="to-prev-button"]')
    expect(toPrevButton.exists()).toBe(true)

    const zeroButtonDiv = pageMoveButtons.find('[data-test="page-index-1"]')
    expect(zeroButtonDiv.exists()).toBe(true)
    expect(zeroButtonDiv.get('button').classes()).toContain('bg-gray-600')
    const oneButtonDiv = pageMoveButtons.find('[data-test="page-index-2"]')
    expect(oneButtonDiv.exists()).toBe(true)
    expect(oneButtonDiv.get('button').classes()).toContain('bg-gray-600')
    const twoButtonDiv = pageMoveButtons.find('[data-test="page-index-3"]')
    expect(twoButtonDiv.exists()).toBe(true)
    expect(twoButtonDiv.get('button').classes()).toContain('bg-gray-400')

    const toNextButton = pageMoveButtons.find('[data-test="to-next-button"]')
    expect(toNextButton.exists()).toBe(false)
    const toLastButton = pageMoveButtons.find('[data-test="to-last-button"]')
    expect(toLastButton.exists()).toBe(false)
  })

  it('has next, last, first, prev and 4 index buttons case1', async () => {
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(1)
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    consultantSearchParamMock.size = getPageSize()
    const result1 = {
      total: 5,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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
      total: 5,
      consultants: [
        {
          consultant_id: 2,
          fee_per_hour_in_yen: 4000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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
    const btnDiv = btns.find('[data-test="to-next-button"]')
    expect(btnDiv.exists()).toBe(true)
    await btnDiv.get('button').trigger('click')
    await flushPromises()

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result2.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(true)

    const consultant = result2.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)

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
    const twoButtonDiv = pageMoveButtons.find('[data-test="page-index-2"]')
    expect(twoButtonDiv.exists()).toBe(true)
    expect(twoButtonDiv.get('button').classes()).toContain('bg-gray-600')
    const threeButtonDiv = pageMoveButtons.find('[data-test="page-index-3"]')
    expect(threeButtonDiv.exists()).toBe(true)
    expect(threeButtonDiv.get('button').classes()).toContain('bg-gray-600')

    const toNextButton = pageMoveButtons.find('[data-test="to-next-button"]')
    expect(toNextButton.exists()).toBe(true)
    const toLastButton = pageMoveButtons.find('[data-test="to-last-button"]')
    expect(toLastButton.exists()).toBe(true)
  })

  it('has next, last, first, prev and 4 index buttons case2', async () => {
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(1)
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    consultantSearchParamMock.size = getPageSize()
    const result1 = {
      total: 4,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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
      total: 4,
      consultants: [
        {
          consultant_id: 2,
          fee_per_hour_in_yen: 4000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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
    const btnDiv = btns.find('[data-test="page-index-2"]')
    expect(btnDiv.exists()).toBe(true)
    await btnDiv.get('button').trigger('click')
    await flushPromises()

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result2.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(true)

    const consultant = result2.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)

    const toFirstButton = pageMoveButtons.find('[data-test="to-first-button"]')
    expect(toFirstButton.exists()).toBe(true)
    const toPrevButton = pageMoveButtons.find('[data-test="to-prev-button"]')
    expect(toPrevButton.exists()).toBe(true)

    const oneButtonDiv = pageMoveButtons.find('[data-test="page-index-0"]')
    expect(oneButtonDiv.exists()).toBe(true)
    expect(oneButtonDiv.get('button').classes()).toContain('bg-gray-600')
    const twoButtonDiv = pageMoveButtons.find('[data-test="page-index-1"]')
    expect(twoButtonDiv.exists()).toBe(true)
    expect(twoButtonDiv.get('button').classes()).toContain('bg-gray-600')
    const threeButtonDiv = pageMoveButtons.find('[data-test="page-index-2"]')
    expect(threeButtonDiv.exists()).toBe(true)
    expect(threeButtonDiv.get('button').classes()).toContain('bg-gray-400')
    const zeroButtonDiv = pageMoveButtons.find('[data-test="page-index-3"]')
    expect(zeroButtonDiv.exists()).toBe(true)
    expect(zeroButtonDiv.get('button').classes()).toContain('bg-gray-600')

    const toNextButton = pageMoveButtons.find('[data-test="to-next-button"]')
    expect(toNextButton.exists()).toBe(true)
    const toLastButton = pageMoveButtons.find('[data-test="to-last-button"]')
    expect(toLastButton.exists()).toBe(true)
  })

  it('has next, last and 3 index buttons', async () => {
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(1)
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    consultantSearchParamMock.size = getPageSize()
    const result1 = {
      total: 3,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result1.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(true)

    const consultant = result1.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)

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
    const twoButtonDiv = pageMoveButtons.find('[data-test="page-index-2"]')
    expect(twoButtonDiv.exists()).toBe(true)
    expect(twoButtonDiv.get('button').classes()).toContain('bg-gray-600')

    const toNextButton = pageMoveButtons.find('[data-test="to-next-button"]')
    expect(toNextButton.exists()).toBe(true)
    const toLastButton = pageMoveButtons.find('[data-test="to-last-button"]')
    expect(toLastButton.exists()).toBe(true)
  })

  it('has first, prev and 3 index buttons', async () => {
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(1)
    if (!consultantSearchParamMock) {
      throw new Error('!consultantSearchParamMock')
    }
    consultantSearchParamMock.size = getPageSize()
    const result1 = {
      total: 3,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: 5000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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
      total: 3,
      consultants: [
        {
          consultant_id: 2,
          fee_per_hour_in_yen: 4000,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
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
    const btnDiv = btns.find('[data-test="to-last-button"]')
    expect(btnDiv.exists()).toBe(true)
    await btnDiv.get('button').trigger('click')
    await flushPromises()

    const totalDiv = wrapper.find('[data-test="total"]')
    expect(totalDiv.text()).toContain(`${result2.total} 件`)

    const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
    expect(sortLabelDiv.text()).toContain('ソート：')
    const sortValueDiv = wrapper.find('[data-test="sort-value"]')
    const options = sortValueDiv.findAll('option')
    expect(options[0].element.selected).toBe(true)

    const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
    expect(pageMoveButtons.exists()).toBe(true)

    const consultant = result2.consultants[0]
    const consultantDiv = wrapper.find(`[data-test="consultant-id-${consultant.consultant_id}"]`)
    expect(consultantDiv.exists()).toBe(true)
    expect(consultantDiv.text()).toContain(`コンサルタントID: ${consultant.consultant_id}`)

    const toFirstButton = pageMoveButtons.find('[data-test="to-first-button"]')
    expect(toFirstButton.exists()).toBe(true)
    const toPrevButton = pageMoveButtons.find('[data-test="to-prev-button"]')
    expect(toPrevButton.exists()).toBe(true)

    const oneButtonDiv = pageMoveButtons.find('[data-test="page-index-0"]')
    expect(oneButtonDiv.exists()).toBe(true)
    expect(oneButtonDiv.get('button').classes()).toContain('bg-gray-600')
    const twoButtonDiv = pageMoveButtons.find('[data-test="page-index-1"]')
    expect(twoButtonDiv.exists()).toBe(true)
    expect(twoButtonDiv.get('button').classes()).toContain('bg-gray-600')
    const threeButtonDiv = pageMoveButtons.find('[data-test="page-index-2"]')
    expect(threeButtonDiv.exists()).toBe(true)
    expect(threeButtonDiv.get('button').classes()).toContain('bg-gray-400')

    const toNextButton = pageMoveButtons.find('[data-test="to-next-button"]')
    expect(toNextButton.exists()).toBe(false)
    const toLastButton = pageMoveButtons.find('[data-test="to-last-button"]')
    expect(toLastButton.exists()).toBe(false)
  })

  it('displays sort param case 1', async () => {
    const result1 = {
      total: 2,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: MAX_FEE_PER_HOUR_IN_YEN,
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
          fee_per_hour_in_yen: MAX_FEE_PER_HOUR_IN_YEN - 1,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
              company_name: 'テスト２株式会社',
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

    const mock1 = {
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
    expect(postConsultantsSearchFuncMock).toHaveBeenCalledWith(mock1)
    {
      const totalDiv = wrapper.find('[data-test="total"]')
      expect(totalDiv.text()).toContain(`${result1.total} 件`)

      const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
      expect(sortLabelDiv.text()).toContain('ソート：')
      const sortValueDiv = wrapper.find('[data-test="sort-value"]')
      const options = sortValueDiv.findAll('option')
      expect(options[0].element.selected).toBe(true)

      const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
      expect(pageMoveButtons.exists()).toBe(false)

      const consultant1 = result1.consultants[0]
      const consultant1Div = wrapper.find(`[data-test="consultant-id-${consultant1.consultant_id}"]`)
      expect(consultant1Div.exists()).toBe(true)
      expect(consultant1Div.text()).toContain(`コンサルタントID: ${consultant1.consultant_id}`)

      const consultant2 = result1.consultants[1]
      const consultant2Div = wrapper.find(`[data-test="consultant-id-${consultant2.consultant_id}"]`)
      expect(consultant2Div.exists()).toBe(true)
      expect(consultant2Div.text()).toContain(`コンサルタントID: ${consultant2.consultant_id}`)
    }

    const result2 = {
      total: 2,
      consultants: [
        {
          consultant_id: 2,
          fee_per_hour_in_yen: MAX_FEE_PER_HOUR_IN_YEN - 1,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              company_name: 'テスト２株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription,
        {
          consultant_id: 1,
          fee_per_hour_in_yen: MAX_FEE_PER_HOUR_IN_YEN,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
              company_name: 'テスト１株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription
      ]
    } as ConsultantsSearchResult
    const resp2 = PostConsultantsSearchResp.create(result2)
    postConsultantsSearchFuncMock.mockResolvedValue(resp2)

    const sortSelect = wrapper.get('[data-test="sort-value"]')
    await sortSelect.setValue('fee_asc')
    await flushPromises()

    const mock2 = mock1
    mock2.sort_param = {
      key: 'fee_per_hour_in_yen',
      order: 'asc'
    } as SortParam
    expect(postConsultantsSearchFuncMock).toHaveBeenNthCalledWith(1, mock2)
    {
      const totalDiv = wrapper.find('[data-test="total"]')
      expect(totalDiv.text()).toContain(`${result2.total} 件`)

      const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
      expect(sortLabelDiv.text()).toContain('ソート：')
      const sortValueDiv = wrapper.find('[data-test="sort-value"]')
      const options = sortValueDiv.findAll('option')
      expect(options[1].element.selected).toBe(true)

      const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
      expect(pageMoveButtons.exists()).toBe(false)

      const consultant1 = result2.consultants[0]
      const consultant1Div = wrapper.find(`[data-test="consultant-id-${consultant1.consultant_id}"]`)
      expect(consultant1Div.exists()).toBe(true)
      expect(consultant1Div.text()).toContain(`コンサルタントID: ${consultant1.consultant_id}`)

      const consultant2 = result2.consultants[1]
      const consultant2Div = wrapper.find(`[data-test="consultant-id-${consultant2.consultant_id}"]`)
      expect(consultant2Div.exists()).toBe(true)
      expect(consultant2Div.text()).toContain(`コンサルタントID: ${consultant2.consultant_id}`)
    }
  })

  it('displays sort param case 2', async () => {
    const result1 = {
      total: 2,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: MIN_FEE_PER_HOUR_IN_YEN,
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
          fee_per_hour_in_yen: MIN_FEE_PER_HOUR_IN_YEN + 1,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
              company_name: 'テスト２株式会社',
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

    const mock1 = {
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
    expect(postConsultantsSearchFuncMock).toHaveBeenCalledWith(mock1)
    {
      const totalDiv = wrapper.find('[data-test="total"]')
      expect(totalDiv.text()).toContain(`${result1.total} 件`)

      const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
      expect(sortLabelDiv.text()).toContain('ソート：')
      const sortValueDiv = wrapper.find('[data-test="sort-value"]')
      const options = sortValueDiv.findAll('option')
      expect(options[0].element.selected).toBe(true)

      const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
      expect(pageMoveButtons.exists()).toBe(false)

      const consultant1 = result1.consultants[0]
      const consultant1Div = wrapper.find(`[data-test="consultant-id-${consultant1.consultant_id}"]`)
      expect(consultant1Div.exists()).toBe(true)
      expect(consultant1Div.text()).toContain(`コンサルタントID: ${consultant1.consultant_id}`)

      const consultant2 = result1.consultants[1]
      const consultant2Div = wrapper.find(`[data-test="consultant-id-${consultant2.consultant_id}"]`)
      expect(consultant2Div.exists()).toBe(true)
      expect(consultant2Div.text()).toContain(`コンサルタントID: ${consultant2.consultant_id}`)
    }

    const result2 = {
      total: 2,
      consultants: [
        {
          consultant_id: 2,
          fee_per_hour_in_yen: MIN_FEE_PER_HOUR_IN_YEN + 1,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              company_name: 'テスト２株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription,
        {
          consultant_id: 1,
          fee_per_hour_in_yen: MIN_FEE_PER_HOUR_IN_YEN,
          rating: null,
          num_of_rated: 0,
          careers: [
            {
              consultant_career_id: 0,
              company_name: 'テスト１株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription
      ]
    } as ConsultantsSearchResult
    const resp2 = PostConsultantsSearchResp.create(result2)
    postConsultantsSearchFuncMock.mockResolvedValue(resp2)

    const sortSelect = wrapper.get('[data-test="sort-value"]')
    await sortSelect.setValue('fee_desc')
    await flushPromises()

    const mock2 = mock1
    mock2.sort_param = {
      key: 'fee_per_hour_in_yen',
      order: 'desc'
    } as SortParam
    expect(postConsultantsSearchFuncMock).toHaveBeenNthCalledWith(1, mock2)
    {
      const totalDiv = wrapper.find('[data-test="total"]')
      expect(totalDiv.text()).toContain(`${result2.total} 件`)

      const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
      expect(sortLabelDiv.text()).toContain('ソート：')
      const sortValueDiv = wrapper.find('[data-test="sort-value"]')
      const options = sortValueDiv.findAll('option')
      expect(options[2].element.selected).toBe(true)

      const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
      expect(pageMoveButtons.exists()).toBe(false)

      const consultant1 = result2.consultants[0]
      const consultant1Div = wrapper.find(`[data-test="consultant-id-${consultant1.consultant_id}"]`)
      expect(consultant1Div.exists()).toBe(true)
      expect(consultant1Div.text()).toContain(`コンサルタントID: ${consultant1.consultant_id}`)

      const consultant2 = result2.consultants[1]
      const consultant2Div = wrapper.find(`[data-test="consultant-id-${consultant2.consultant_id}"]`)
      expect(consultant2Div.exists()).toBe(true)
      expect(consultant2Div.text()).toContain(`コンサルタントID: ${consultant2.consultant_id}`)
    }
  })

  it('displays sort param case 3', async () => {
    const result1 = {
      total: 2,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: MIN_FEE_PER_HOUR_IN_YEN,
          rating: 4.9,
          num_of_rated: 10,
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
          fee_per_hour_in_yen: MIN_FEE_PER_HOUR_IN_YEN,
          rating: 5.0,
          num_of_rated: 10,
          careers: [
            {
              consultant_career_id: 0,
              company_name: 'テスト２株式会社',
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

    const mock1 = {
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
    expect(postConsultantsSearchFuncMock).toHaveBeenCalledWith(mock1)
    {
      const totalDiv = wrapper.find('[data-test="total"]')
      expect(totalDiv.text()).toContain(`${result1.total} 件`)

      const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
      expect(sortLabelDiv.text()).toContain('ソート：')
      const sortValueDiv = wrapper.find('[data-test="sort-value"]')
      const options = sortValueDiv.findAll('option')
      expect(options[0].element.selected).toBe(true)

      const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
      expect(pageMoveButtons.exists()).toBe(false)

      const consultant1 = result1.consultants[0]
      const consultant1Div = wrapper.find(`[data-test="consultant-id-${consultant1.consultant_id}"]`)
      expect(consultant1Div.exists()).toBe(true)
      expect(consultant1Div.text()).toContain(`コンサルタントID: ${consultant1.consultant_id}`)

      const consultant2 = result1.consultants[1]
      const consultant2Div = wrapper.find(`[data-test="consultant-id-${consultant2.consultant_id}"]`)
      expect(consultant2Div.exists()).toBe(true)
      expect(consultant2Div.text()).toContain(`コンサルタントID: ${consultant2.consultant_id}`)
    }

    const result2 = {
      total: 2,
      consultants: [
        {
          consultant_id: 2,
          fee_per_hour_in_yen: MIN_FEE_PER_HOUR_IN_YEN,
          rating: 5.0,
          num_of_rated: 10,
          careers: [
            {
              company_name: 'テスト２株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription,
        {
          consultant_id: 1,
          fee_per_hour_in_yen: MIN_FEE_PER_HOUR_IN_YEN,
          rating: 4.9,
          num_of_rated: 10,
          careers: [
            {
              consultant_career_id: 0,
              company_name: 'テスト１株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription
      ]
    } as ConsultantsSearchResult
    const resp2 = PostConsultantsSearchResp.create(result2)
    postConsultantsSearchFuncMock.mockResolvedValue(resp2)

    const sortSelect = wrapper.get('[data-test="sort-value"]')
    await sortSelect.setValue('rating_desc')
    await flushPromises()

    const mock2 = mock1
    mock2.sort_param = {
      key: 'rating',
      order: 'desc'
    } as SortParam
    expect(postConsultantsSearchFuncMock).toHaveBeenNthCalledWith(1, mock2)
    {
      const totalDiv = wrapper.find('[data-test="total"]')
      expect(totalDiv.text()).toContain(`${result2.total} 件`)

      const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
      expect(sortLabelDiv.text()).toContain('ソート：')
      const sortValueDiv = wrapper.find('[data-test="sort-value"]')
      const options = sortValueDiv.findAll('option')
      expect(options[3].element.selected).toBe(true)

      const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
      expect(pageMoveButtons.exists()).toBe(false)

      const consultant1 = result2.consultants[0]
      const consultant1Div = wrapper.find(`[data-test="consultant-id-${consultant1.consultant_id}"]`)
      expect(consultant1Div.exists()).toBe(true)
      expect(consultant1Div.text()).toContain(`コンサルタントID: ${consultant1.consultant_id}`)

      const consultant2 = result2.consultants[1]
      const consultant2Div = wrapper.find(`[data-test="consultant-id-${consultant2.consultant_id}"]`)
      expect(consultant2Div.exists()).toBe(true)
      expect(consultant2Div.text()).toContain(`コンサルタントID: ${consultant2.consultant_id}`)
    }
  })

  it('displays sort param case 4', async () => {
    const result1 = {
      total: 2,
      consultants: [
        {
          consultant_id: 2,
          fee_per_hour_in_yen: MIN_FEE_PER_HOUR_IN_YEN,
          rating: 5.0,
          num_of_rated: 10,
          careers: [
            {
              company_name: 'テスト２株式会社',
              profession: null,
              office: null
            } as ConsultantCareerDescription
          ]
        } as ConsultantDescription,
        {
          consultant_id: 1,
          fee_per_hour_in_yen: MIN_FEE_PER_HOUR_IN_YEN,
          rating: 4.9,
          num_of_rated: 10,
          careers: [
            {
              consultant_career_id: 0,
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

    const mock1 = {
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
    expect(postConsultantsSearchFuncMock).toHaveBeenCalledWith(mock1)
    {
      const totalDiv = wrapper.find('[data-test="total"]')
      expect(totalDiv.text()).toContain(`${result1.total} 件`)

      const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
      expect(sortLabelDiv.text()).toContain('ソート：')
      const sortValueDiv = wrapper.find('[data-test="sort-value"]')
      const options = sortValueDiv.findAll('option')
      expect(options[0].element.selected).toBe(true)

      const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
      expect(pageMoveButtons.exists()).toBe(false)

      const consultant1 = result1.consultants[0]
      const consultant1Div = wrapper.find(`[data-test="consultant-id-${consultant1.consultant_id}"]`)
      expect(consultant1Div.exists()).toBe(true)
      expect(consultant1Div.text()).toContain(`コンサルタントID: ${consultant1.consultant_id}`)

      const consultant2 = result1.consultants[1]
      const consultant2Div = wrapper.find(`[data-test="consultant-id-${consultant2.consultant_id}"]`)
      expect(consultant2Div.exists()).toBe(true)
      expect(consultant2Div.text()).toContain(`コンサルタントID: ${consultant2.consultant_id}`)
    }

    const result2 = {
      total: 2,
      consultants: [
        {
          consultant_id: 1,
          fee_per_hour_in_yen: MIN_FEE_PER_HOUR_IN_YEN,
          rating: 4.9,
          num_of_rated: 10,
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
          fee_per_hour_in_yen: MIN_FEE_PER_HOUR_IN_YEN,
          rating: 5.0,
          num_of_rated: 10,
          careers: [
            {
              consultant_career_id: 0,
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

    const sortSelect = wrapper.get('[data-test="sort-value"]')
    await sortSelect.setValue('rating_asc')
    await flushPromises()

    const mock2 = mock1
    mock2.sort_param = {
      key: 'rating',
      order: 'asc'
    } as SortParam
    expect(postConsultantsSearchFuncMock).toHaveBeenNthCalledWith(1, mock2)
    {
      const totalDiv = wrapper.find('[data-test="total"]')
      expect(totalDiv.text()).toContain(`${result2.total} 件`)

      const sortLabelDiv = wrapper.find('[data-test="sort-label"]')
      expect(sortLabelDiv.text()).toContain('ソート：')
      const sortValueDiv = wrapper.find('[data-test="sort-value"]')
      const options = sortValueDiv.findAll('option')
      expect(options[4].element.selected).toBe(true)

      const pageMoveButtons = wrapper.find('[data-test="page-move-buttons"]')
      expect(pageMoveButtons.exists()).toBe(false)

      const consultant1 = result2.consultants[0]
      const consultant1Div = wrapper.find(`[data-test="consultant-id-${consultant1.consultant_id}"]`)
      expect(consultant1Div.exists()).toBe(true)
      expect(consultant1Div.text()).toContain(`コンサルタントID: ${consultant1.consultant_id}`)

      const consultant2 = result2.consultants[1]
      const consultant2Div = wrapper.find(`[data-test="consultant-id-${consultant2.consultant_id}"]`)
      expect(consultant2Div.exists()).toBe(true)
      expect(consultant2Div.text()).toContain(`コンサルタントID: ${consultant2.consultant_id}`)
    }
  })
})
