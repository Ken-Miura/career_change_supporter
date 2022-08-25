import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import ConsultantsSearchPage from '@/views/personalized/ConsultantsSearchPage.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { refresh } from '@/util/personalized/refresh/Refresh'
import TheHeader from '@/components/TheHeader.vue'
import { Message } from '@/util/Message'
import { getPageSize, PAGE_SIZE } from '@/util/PageSize'
import { AnnualInComeInManYenParam, CareerParam, ConsultantSearchParam, FeePerHourInYenParam, YearsOfServiceParam } from '@/util/personalized/ConsultantSearchParam'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { SET_CONSULTANT_SEARCH_PARAM } from '@/store/mutationTypes'
import { MAX_ANNUAL_INCOME_IN_MAN_YEN, MIN_ANNUAL_INCOME_IN_MAN_YEN } from '@/util/AnnualIncome'
import { MAX_FEE_PER_HOUR_IN_YEN, MIN_FEE_PER_HOUR_IN_YEN } from '@/util/Fee'

jest.mock('@/util/personalized/refresh/Refresh')
const refreshMock = refresh as jest.MockedFunction<typeof refresh>

jest.mock('@/util/PageSize')
const getPageSizeMock = getPageSize as jest.MockedFunction<typeof getPageSize>

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

const storeCommitMock = jest.fn()
jest.mock('vuex', () => ({
  useStore: () => ({
    commit: storeCommitMock
  })
}))

describe('ConsultantsSearchPage.vue', () => {
  beforeEach(() => {
    refreshMock.mockReset()
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(PAGE_SIZE)
    routerPushMock.mockClear()
    storeCommitMock.mockClear()
  })

  it('has one TheHeader, one submit button and one AlertMessage', () => {
    const wrapper = mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const headers = wrapper.findAllComponents(TheHeader)
    expect(headers.length).toBe(1)
    const submitButton = wrapper.find('[data-test="submit-button"]')
    expect(submitButton.exists()).toBe(true)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
  })

  it('has labels and inputs for search param', () => {
    const wrapper = mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const companyNameLabel = wrapper.find('[data-test="company-name-label"]')
    expect(companyNameLabel.exists()).toBe(true)
    expect(companyNameLabel.text()).toContain('勤務先名称（例 xxx株式会社）')
    const companyNameInput = wrapper.find('[data-test="company-name-input"]').find('input')
    expect(companyNameInput.exists()).toBe(true)

    const departmentNameLabel = wrapper.find('[data-test="department-name-label"]')
    expect(departmentNameLabel.exists()).toBe(true)
    expect(departmentNameLabel.text()).toContain('部署名')
    const departmentNameInput = wrapper.find('[data-test="department-name-input"]').find('input')
    expect(departmentNameInput.exists()).toBe(true)

    const officeLabel = wrapper.find('[data-test="office-label"]')
    expect(officeLabel.exists()).toBe(true)
    expect(officeLabel.text()).toContain('勤務地（例 xxx事業所）')
    const officeInput = wrapper.find('[data-test="office-input"]').find('input')
    expect(officeInput.exists()).toBe(true)

    const yearsOfServiceLabel = wrapper.find('[data-test="years-of-service-label"]')
    expect(yearsOfServiceLabel.exists()).toBe(true)
    expect(yearsOfServiceLabel.text()).toContain('在籍年数')
    const yearsOfServiceSelect = wrapper.find('[data-test="years-of-service-equal-or-more-select"]').find('select')
    expect(yearsOfServiceSelect.exists()).toBe(true)

    const employedLabel = wrapper.find('[data-test="employed-label"]')
    expect(employedLabel.exists()).toBe(true)
    expect(employedLabel.text()).toContain('在籍の有無')
    const employedSelect = wrapper.find('[data-test="employed-select"]').find('select')
    expect(employedSelect.exists()).toBe(true)

    const contractTypeLabel = wrapper.find('[data-test="contract-type-label"]')
    expect(contractTypeLabel.exists()).toBe(true)
    expect(contractTypeLabel.text()).toContain('雇用形態')
    const contractTypeSelect = wrapper.find('[data-test="contract-type-select"]').find('select')
    expect(contractTypeSelect.exists()).toBe(true)

    const professionLabel = wrapper.find('[data-test="profession-label"]')
    expect(professionLabel.exists()).toBe(true)
    expect(professionLabel.text()).toContain('職種（例 ITエンジニア）')
    const professionInput = wrapper.find('[data-test="profession-input"]').find('input')
    expect(professionInput.exists()).toBe(true)

    const annualIncomeInManYenLabel = wrapper.find('[data-test="annual-income-in-man-yen-label"]')
    expect(annualIncomeInManYenLabel.exists()).toBe(true)
    expect(annualIncomeInManYenLabel.text()).toContain('年収（単位：万円）')
    const annualIncomeInManYenEqualOrMoreInput = wrapper.find('[data-test="annual-income-in-man-yen-equal-or-more-input"]').find('input')
    expect(annualIncomeInManYenEqualOrMoreInput.exists()).toBe(true)
    const annualIncomeInManYenEqualOrMoreLabel = wrapper.find('[data-test="annual-income-in-man-yen-equal-or-more-label"]')
    expect(annualIncomeInManYenEqualOrMoreLabel.exists()).toBe(true)
    expect(annualIncomeInManYenEqualOrMoreLabel.text()).toContain('万円以上')
    const annualIncomeInManYenEqualOrLessInput = wrapper.find('[data-test="annual-income-in-man-yen-equal-or-less-input"]').find('input')
    expect(annualIncomeInManYenEqualOrLessInput.exists()).toBe(true)
    const annualIncomeInManYenEqualOrLessLabel = wrapper.find('[data-test="annual-income-in-man-yen-equal-or-less-label"]')
    expect(annualIncomeInManYenEqualOrLessLabel.exists()).toBe(true)
    expect(annualIncomeInManYenEqualOrLessLabel.text()).toContain('万円以下')

    const isManagerLabel = wrapper.find('[data-test="is-manager-label"]')
    expect(isManagerLabel.exists()).toBe(true)
    expect(isManagerLabel.text()).toContain('管理職区分')
    const isManagerSelect = wrapper.find('[data-test="is-manager-select"]').find('select')
    expect(isManagerSelect.exists()).toBe(true)

    const positionNameLabel = wrapper.find('[data-test="position-name-label"]')
    expect(positionNameLabel.exists()).toBe(true)
    expect(positionNameLabel.text()).toContain('職位（例 係長）')
    const positionNameInput = wrapper.find('[data-test="position-name-input"]').find('input')
    expect(positionNameInput.exists()).toBe(true)

    const isNewGraduateLabel = wrapper.find('[data-test="is-new-graduate-label"]')
    expect(isNewGraduateLabel.exists()).toBe(true)
    expect(isNewGraduateLabel.text()).toContain('入社区分')
    const isNewGraduateSelect = wrapper.find('[data-test="is-new-graduate-select"]').find('select')
    expect(isNewGraduateSelect.exists()).toBe(true)

    const noteLabel = wrapper.find('[data-test="note-label"]')
    expect(noteLabel.exists()).toBe(true)
    expect(noteLabel.text()).toContain('備考')
    const noteInput = wrapper.find('[data-test="note-input"]').find('textarea')
    expect(noteInput.exists()).toBe(true)

    const feePerHourInYenLabel = wrapper.find('[data-test="fee-per-hour-in-yen-label"]')
    expect(feePerHourInYenLabel.exists()).toBe(true)
    expect(feePerHourInYenLabel.text()).toContain('相談一回（１時間）の相談料（単位：円）')
    const feePerHourInYenEqualOrMoreInput = wrapper.find('[data-test="fee-per-hour-in-yen-equal-or-more-input"]').find('input')
    expect(feePerHourInYenEqualOrMoreInput.exists()).toBe(true)
    const feePerHourInYenEqualOrMoreLabel = wrapper.find('[data-test="fee-per-hour-in-yen-equal-or-more-label"]')
    expect(feePerHourInYenEqualOrMoreLabel.exists()).toBe(true)
    expect(feePerHourInYenEqualOrMoreLabel.text()).toContain('円以上')
    const feePerHourInYenEqualOrLessInput = wrapper.find('[data-test="fee-per-hour-in-yen-equal-or-less-input"]').find('input')
    expect(feePerHourInYenEqualOrLessInput.exists()).toBe(true)
    const feePerHourInYenEqualOrLessLabel = wrapper.find('[data-test="fee-per-hour-in-yen-equal-or-less-label"]')
    expect(feePerHourInYenEqualOrLessLabel.exists()).toBe(true)
    expect(feePerHourInYenEqualOrLessLabel.text()).toContain('円以下')
  })

  it('has AlertMessage with a hidden attribute when created', () => {
    const wrapper = mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).toContain('hidden')
  })

  it(`moves to login if ${Code.UNAUTHORIZED} is returned on opening ConsultantsSearchPage`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    refreshMock.mockResolvedValue(apiErrResp)
    mount(ConsultantsSearchPage, {
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

  it(`moves to terms of use if ${Code.NOT_TERMS_OF_USE_AGREED_YET} is returned on opening ConsultantsSearchPage`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    refreshMock.mockResolvedValue(apiErrResp)
    mount(ConsultantsSearchPage, {
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

  it(`displays alert message ${Message.UNEXPECTED_ERR} when connection error happened on opening ConsultantsSearchPage`, async () => {
    const errDetail = 'connection error'
    refreshMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.UNEXPECTED_ERR)
    expect(resultMessage).toContain(errDetail)
  })

  it('clears search param on opening ConsultantsSearchPage', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    expect(storeCommitMock).toHaveBeenCalledTimes(1)
    expect(storeCommitMock).toHaveBeenNthCalledWith(1, SET_CONSULTANT_SEARCH_PARAM, null)
  })

  it('moves to consultant-list and pass empty param if no param specified', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const wrapper = mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    expect(storeCommitMock).toHaveBeenNthCalledWith(1, SET_CONSULTANT_SEARCH_PARAM, null)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/consultant-list')
    expect(storeCommitMock).toHaveBeenCalledTimes(2)
    const consultantSearchParam = {
      career_param: {
        company_name: null,
        department_name: null,
        office: null,
        years_of_service: {
          equal_or_more: null,
          less_than: null
        } as YearsOfServiceParam,
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
    expect(storeCommitMock).toHaveBeenNthCalledWith(2, SET_CONSULTANT_SEARCH_PARAM, consultantSearchParam)
  })

  it('moves to consultant-list and pass specified params if all params are specified', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const wrapper = mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    expect(storeCommitMock).toHaveBeenNthCalledWith(1, SET_CONSULTANT_SEARCH_PARAM, null)

    const companyName = 'テスト株式会社'
    const companyNameInput = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyNameInput.setValue(companyName)

    const departmentName = 'ソフトウェア開発'
    const departmentNameInput = wrapper.find('[data-test="department-name-input"]').find('input')
    await departmentNameInput.setValue(departmentName)

    const office = '町田事業所'
    const officeInput = wrapper.find('[data-test="office-input"]').find('input')
    await officeInput.setValue(office)

    const equalOrMoreYearsOfService = '3'
    const equalOrMoreYearsOfServiceSelect = wrapper.find('[data-test="years-of-service-equal-or-more-select"]').find('select')
    await equalOrMoreYearsOfServiceSelect.setValue(equalOrMoreYearsOfService)

    const lessThanYearsOfService = '20'
    const lessThanYearsOfServiceSelect = wrapper.find('[data-test="years-of-service-less-than-select"]').find('select')
    await lessThanYearsOfServiceSelect.setValue(lessThanYearsOfService)

    const employed = 'true'
    const employedSelect = wrapper.find('[data-test="employed-select"]').find('select')
    await employedSelect.setValue(employed)

    const contractType = 'regular'
    const contractTypeSelect = wrapper.find('[data-test="contract-type-select"]').find('select')
    await contractTypeSelect.setValue(contractType)

    const profession = 'ITエンジニア'
    const professionInput = wrapper.find('[data-test="profession-input"]').find('input')
    await professionInput.setValue(profession)

    const annualIncomeInManYenEqualOrMore = MIN_ANNUAL_INCOME_IN_MAN_YEN
    const annualIncomeInManYenEqualOrMoreInput = wrapper.find('[data-test="annual-income-in-man-yen-equal-or-more-input"]').find('input')
    await annualIncomeInManYenEqualOrMoreInput.setValue(annualIncomeInManYenEqualOrMore)

    const annualIncomeInManYenEqualOrLess = MAX_ANNUAL_INCOME_IN_MAN_YEN
    const annualIncomeInManYenEqualOrLessInput = wrapper.find('[data-test="annual-income-in-man-yen-equal-or-less-input"]').find('input')
    await annualIncomeInManYenEqualOrLessInput.setValue(annualIncomeInManYenEqualOrLess)

    const isManager = 'false' as string
    const isManagerSelect = wrapper.find('[data-test="is-manager-select"]').find('select')
    await isManagerSelect.setValue(isManager)

    const positionName = '主任'
    const positionNameInput = wrapper.find('[data-test="position-name-input"]').find('input')
    await positionNameInput.setValue(positionName)

    const isNewGraduate = 'true'
    const isNewGraduateSelect = wrapper.find('[data-test="is-new-graduate-select"]').find('select')
    await isNewGraduateSelect.setValue(isNewGraduate)

    const note = '備考'
    const noteInput = wrapper.find('[data-test="note-input"]').find('textarea')
    await noteInput.setValue(note)

    const feePerHourInYenEqualOrMore = MIN_FEE_PER_HOUR_IN_YEN
    const feePerHourInYenEqualOrMoreInput = wrapper.find('[data-test="fee-per-hour-in-yen-equal-or-more-input"]').find('input')
    await feePerHourInYenEqualOrMoreInput.setValue(feePerHourInYenEqualOrMore)

    const feePerHourInYenEqualOrLess = MAX_FEE_PER_HOUR_IN_YEN
    const feePerHourInYenEqualOrLessInput = wrapper.find('[data-test="fee-per-hour-in-yen-equal-or-less-input"]').find('input')
    await feePerHourInYenEqualOrLessInput.setValue(feePerHourInYenEqualOrLess)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/consultant-list')
    expect(storeCommitMock).toHaveBeenCalledTimes(2)
    const consultantSearchParam = {
      career_param: {
        company_name: companyName,
        department_name: departmentName,
        office: office,
        years_of_service: {
          equal_or_more: parseInt(equalOrMoreYearsOfServiceSelect.element.value),
          less_than: parseInt(lessThanYearsOfServiceSelect.element.value)
        } as YearsOfServiceParam,
        employed: employed === 'true',
        contract_type: contractType,
        profession: profession,
        annual_income_in_man_yen: {
          equal_or_more: annualIncomeInManYenEqualOrMore,
          equal_or_less: annualIncomeInManYenEqualOrLess
        } as AnnualInComeInManYenParam,
        is_manager: isManager === 'true',
        position_name: positionName,
        is_new_graduate: isNewGraduate === 'true',
        note: note
      } as CareerParam,
      fee_per_hour_in_yen_param: {
        equal_or_more: feePerHourInYenEqualOrMore,
        equal_or_less: feePerHourInYenEqualOrLess
      } as FeePerHourInYenParam,
      sort_param: null,
      from: 0,
      size: getPageSize()
    } as ConsultantSearchParam
    expect(storeCommitMock).toHaveBeenNthCalledWith(2, SET_CONSULTANT_SEARCH_PARAM, consultantSearchParam)
  })

  it(`displays ${Message.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN_MESSAGE} if illegal annual income in man yen is passed case 1`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const wrapper = mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    expect(storeCommitMock).toHaveBeenNthCalledWith(1, SET_CONSULTANT_SEARCH_PARAM, null)

    const annualIncomeInManYenEqualOrMore = MIN_ANNUAL_INCOME_IN_MAN_YEN - 1
    const annualIncomeInManYenEqualOrMoreInput = wrapper.find('[data-test="annual-income-in-man-yen-equal-or-more-input"]').find('input')
    await annualIncomeInManYenEqualOrMoreInput.setValue(annualIncomeInManYenEqualOrMore)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    expect(storeCommitMock).toHaveBeenCalledTimes(1)

    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    expect(alertMessage.text()).toContain(Message.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN_MESSAGE)
  })

  it(`displays ${Message.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN_MESSAGE} if illegal annual income in man yen is passed case 2`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const wrapper = mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    expect(storeCommitMock).toHaveBeenNthCalledWith(1, SET_CONSULTANT_SEARCH_PARAM, null)

    const annualIncomeInManYenEqualOrLess = MAX_ANNUAL_INCOME_IN_MAN_YEN + 1
    const annualIncomeInManYenEqualOrLessInput = wrapper.find('[data-test="annual-income-in-man-yen-equal-or-less-input"]').find('input')
    await annualIncomeInManYenEqualOrLessInput.setValue(annualIncomeInManYenEqualOrLess)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    expect(storeCommitMock).toHaveBeenCalledTimes(1)

    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    expect(alertMessage.text()).toContain(Message.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN_MESSAGE)
  })

  it('moves to consultant-list if same value is specified on annualIncomeInManYen', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const wrapper = mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    expect(storeCommitMock).toHaveBeenNthCalledWith(1, SET_CONSULTANT_SEARCH_PARAM, null)

    const annualIncomeInManYenEqualOrMore = MIN_ANNUAL_INCOME_IN_MAN_YEN
    const annualIncomeInManYenEqualOrMoreInput = wrapper.find('[data-test="annual-income-in-man-yen-equal-or-more-input"]').find('input')
    await annualIncomeInManYenEqualOrMoreInput.setValue(annualIncomeInManYenEqualOrMore)

    const annualIncomeInManYenEqualOrLess = annualIncomeInManYenEqualOrMore
    const annualIncomeInManYenEqualOrLessInput = wrapper.find('[data-test="annual-income-in-man-yen-equal-or-less-input"]').find('input')
    await annualIncomeInManYenEqualOrLessInput.setValue(annualIncomeInManYenEqualOrLess)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/consultant-list')
    expect(storeCommitMock).toHaveBeenCalledTimes(2)
    const consultantSearchParam = {
      career_param: {
        company_name: null,
        department_name: null,
        office: null,
        years_of_service: {
          equal_or_more: null,
          less_than: null
        } as YearsOfServiceParam,
        employed: null,
        contract_type: null,
        profession: null,
        annual_income_in_man_yen: {
          equal_or_more: annualIncomeInManYenEqualOrMore,
          equal_or_less: annualIncomeInManYenEqualOrLess
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
    expect(storeCommitMock).toHaveBeenNthCalledWith(2, SET_CONSULTANT_SEARCH_PARAM, consultantSearchParam)
  })

  it(`displays ${Message.EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_ANNUAL_INCOME_IN_MAN_YEN_MESSAGE} if illegal annual incomes in man yen is passed`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const wrapper = mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    expect(storeCommitMock).toHaveBeenNthCalledWith(1, SET_CONSULTANT_SEARCH_PARAM, null)

    const annualIncomeInManYenEqualOrMore = MIN_ANNUAL_INCOME_IN_MAN_YEN + 1
    const annualIncomeInManYenEqualOrMoreInput = wrapper.find('[data-test="annual-income-in-man-yen-equal-or-more-input"]').find('input')
    await annualIncomeInManYenEqualOrMoreInput.setValue(annualIncomeInManYenEqualOrMore)

    const annualIncomeInManYenEqualOrLess = MIN_ANNUAL_INCOME_IN_MAN_YEN
    const annualIncomeInManYenEqualOrLessInput = wrapper.find('[data-test="annual-income-in-man-yen-equal-or-less-input"]').find('input')
    await annualIncomeInManYenEqualOrLessInput.setValue(annualIncomeInManYenEqualOrLess)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    expect(storeCommitMock).toHaveBeenCalledTimes(1)

    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    expect(alertMessage.text()).toContain(Message.EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_ANNUAL_INCOME_IN_MAN_YEN_MESSAGE)
  })

  it(`displays ${Message.ILLEGAL_FEE_PER_HOUR_IN_YEN_MESSAGE} if illegal fee is passed case 1`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const wrapper = mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    expect(storeCommitMock).toHaveBeenNthCalledWith(1, SET_CONSULTANT_SEARCH_PARAM, null)

    const feePerHourInYenEqualOrMore = MIN_FEE_PER_HOUR_IN_YEN - 1
    const feePerHourInYenEqualOrMoreInput = wrapper.find('[data-test="fee-per-hour-in-yen-equal-or-more-input"]').find('input')
    await feePerHourInYenEqualOrMoreInput.setValue(feePerHourInYenEqualOrMore)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    expect(storeCommitMock).toHaveBeenCalledTimes(1)

    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    expect(alertMessage.text()).toContain(Message.ILLEGAL_FEE_PER_HOUR_IN_YEN_MESSAGE)
  })

  it(`displays ${Message.ILLEGAL_FEE_PER_HOUR_IN_YEN_MESSAGE} if illegal fee is passed case 2`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const wrapper = mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    expect(storeCommitMock).toHaveBeenNthCalledWith(1, SET_CONSULTANT_SEARCH_PARAM, null)

    const feePerHourInYenEqualOrLess = MAX_FEE_PER_HOUR_IN_YEN + 1
    const feePerHourInYenEqualOrLessInput = wrapper.find('[data-test="fee-per-hour-in-yen-equal-or-less-input"]').find('input')
    await feePerHourInYenEqualOrLessInput.setValue(feePerHourInYenEqualOrLess)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    expect(storeCommitMock).toHaveBeenCalledTimes(1)

    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    expect(alertMessage.text()).toContain(Message.ILLEGAL_FEE_PER_HOUR_IN_YEN_MESSAGE)
  })

  it('moves to consultant-list if same value is specified on fee', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const wrapper = mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    expect(storeCommitMock).toHaveBeenNthCalledWith(1, SET_CONSULTANT_SEARCH_PARAM, null)

    const feePerHourInYenEqualOrMore = MAX_FEE_PER_HOUR_IN_YEN
    const feePerHourInYenEqualOrMoreInput = wrapper.find('[data-test="fee-per-hour-in-yen-equal-or-more-input"]').find('input')
    await feePerHourInYenEqualOrMoreInput.setValue(feePerHourInYenEqualOrMore)

    const feePerHourInYenEqualOrLess = feePerHourInYenEqualOrMore
    const feePerHourInYenEqualOrLessInput = wrapper.find('[data-test="fee-per-hour-in-yen-equal-or-less-input"]').find('input')
    await feePerHourInYenEqualOrLessInput.setValue(feePerHourInYenEqualOrLess)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/consultant-list')
    expect(storeCommitMock).toHaveBeenCalledTimes(2)
    const consultantSearchParam = {
      career_param: {
        company_name: null,
        department_name: null,
        office: null,
        years_of_service: {
          equal_or_more: null,
          less_than: null
        } as YearsOfServiceParam,
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
        equal_or_more: feePerHourInYenEqualOrMore,
        equal_or_less: feePerHourInYenEqualOrLess
      } as FeePerHourInYenParam,
      sort_param: null,
      from: 0,
      size: getPageSize()
    } as ConsultantSearchParam
    expect(storeCommitMock).toHaveBeenNthCalledWith(2, SET_CONSULTANT_SEARCH_PARAM, consultantSearchParam)
  })

  it(`displays ${Message.EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_FEE_PER_HOUR_IN_YEN_MESSAGE} if illegal fees are passed`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const wrapper = mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    expect(storeCommitMock).toHaveBeenNthCalledWith(1, SET_CONSULTANT_SEARCH_PARAM, null)

    const feePerHourInYenEqualOrMore = MAX_FEE_PER_HOUR_IN_YEN
    const feePerHourInYenEqualOrMoreInput = wrapper.find('[data-test="fee-per-hour-in-yen-equal-or-more-input"]').find('input')
    await feePerHourInYenEqualOrMoreInput.setValue(feePerHourInYenEqualOrMore)

    const feePerHourInYenEqualOrLess = MAX_FEE_PER_HOUR_IN_YEN - 1
    const feePerHourInYenEqualOrLessInput = wrapper.find('[data-test="fee-per-hour-in-yen-equal-or-less-input"]').find('input')
    await feePerHourInYenEqualOrLessInput.setValue(feePerHourInYenEqualOrLess)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    expect(storeCommitMock).toHaveBeenCalledTimes(1)

    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    expect(alertMessage.text()).toContain(Message.EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_FEE_PER_HOUR_IN_YEN_MESSAGE)
  })

  it(`displays ${Message.EQUAL_OR_MORE_IS_LESS_THAN_OR_MORE_YEARS_OF_SERVICE_MESSAGE} if illegal years of service is passed`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const wrapper = mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    expect(storeCommitMock).toHaveBeenNthCalledWith(1, SET_CONSULTANT_SEARCH_PARAM, null)

    const equalOrMoreYearsOfService = '3'
    const equalOrMoreYearsOfServiceSelect = wrapper.find('[data-test="years-of-service-equal-or-more-select"]').find('select')
    await equalOrMoreYearsOfServiceSelect.setValue(equalOrMoreYearsOfService)

    const lessThanYearsOfService = '3'
    const lessThanYearsOfServiceSelect = wrapper.find('[data-test="years-of-service-less-than-select"]').find('select')
    await lessThanYearsOfServiceSelect.setValue(lessThanYearsOfService)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    expect(storeCommitMock).toHaveBeenCalledTimes(1)

    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    expect(alertMessage.text()).toContain(Message.EQUAL_OR_MORE_IS_LESS_THAN_OR_MORE_YEARS_OF_SERVICE_MESSAGE)
  })

  // 選択肢以外の値を用いると発生するケース。基本的にはありえないが念の為実施しておく。
  it(`displays ${Message.ILLEGAL_YEARS_OF_SERVICE_MESSAGE} if illegal years of service is passed case 1`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const wrapper = mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    expect(storeCommitMock).toHaveBeenNthCalledWith(1, SET_CONSULTANT_SEARCH_PARAM, null)

    const equalOrMoreYearsOfService = '1'
    const equalOrMoreYearsOfServiceSelect = wrapper.find('[data-test="years-of-service-equal-or-more-select"]').find('select')
    await equalOrMoreYearsOfServiceSelect.setValue(equalOrMoreYearsOfService)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    expect(storeCommitMock).toHaveBeenCalledTimes(1)

    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    expect(alertMessage.text()).toContain(Message.ILLEGAL_YEARS_OF_SERVICE_MESSAGE)
  })

  // 選択肢以外の値を用いると発生するケース。基本的にはありえないが念の為実施しておく。
  it(`displays ${Message.ILLEGAL_YEARS_OF_SERVICE_MESSAGE} if illegal years of service is passed case 2`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const wrapper = mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    expect(storeCommitMock).toHaveBeenNthCalledWith(1, SET_CONSULTANT_SEARCH_PARAM, null)

    const lessThanYearsOfService = '21'
    const lessThanYearsOfServiceSelect = wrapper.find('[data-test="years-of-service-less-than-select"]').find('select')
    await lessThanYearsOfServiceSelect.setValue(lessThanYearsOfService)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    expect(storeCommitMock).toHaveBeenCalledTimes(1)

    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    expect(alertMessage.text()).toContain(Message.ILLEGAL_YEARS_OF_SERVICE_MESSAGE)
  })
})
