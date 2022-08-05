import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import ConsultantsSearchPage from '@/views/personalized/ConsultantsSearchPage.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { refresh } from '@/util/personalized/refresh/Refresh'
import TheHeader from '@/components/TheHeader.vue'
import { Message } from '@/util/Message'
import { getPageSize, PAGE_SIZE } from '@/util/PageSize'
import { ConsultantSearchParam } from '@/util/personalized/ConsultantSearchParam'

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

let consultantSearchParamMock = null as ConsultantSearchParam | null
const storeCommitMock = jest.fn()
jest.mock('vuex', () => ({
  useStore: () => ({
    commit: storeCommitMock,
    state: {
      consultantSearchParam: consultantSearchParamMock
    }
  })
}))

describe('ConsultantsSearchPage.vue', () => {
  beforeEach(() => {
    refreshMock.mockReset()
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(PAGE_SIZE)
    routerPushMock.mockClear()
    storeCommitMock.mockClear()
    consultantSearchParamMock = null
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
    expect(submitButton.exists)
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
    expect(companyNameLabel.exists)
    expect(companyNameLabel.text()).toContain('勤務先名称（例 xxx株式会社）')
    const companyNameInput = wrapper.find('[data-test="company-name-input"]').find('input')
    expect(companyNameInput.exists)

    const departmentNameLabel = wrapper.find('[data-test="department-name-label"]')
    expect(departmentNameLabel.exists)
    expect(departmentNameLabel.text()).toContain('部署名')
    const departmentNameInput = wrapper.find('[data-test="department-name-input"]').find('input')
    expect(departmentNameInput.exists)

    const officeLabel = wrapper.find('[data-test="office-label"]')
    expect(officeLabel.exists)
    expect(officeLabel.text()).toContain('勤務地（例 xxx事業所）')
    const officeInput = wrapper.find('[data-test="office-input"]').find('input')
    expect(officeInput.exists)

    const yearsOfServiceLabel = wrapper.find('[data-test="years-of-service-label"]')
    expect(yearsOfServiceLabel.exists)
    expect(yearsOfServiceLabel.text()).toContain('在籍年数')
    const yearsOfServiceSelect = wrapper.find('[data-test="years-of-service-select"]').find('select')
    expect(yearsOfServiceSelect.exists)

    const employedLabel = wrapper.find('[data-test="employed-label"]')
    expect(employedLabel.exists)
    expect(employedLabel.text()).toContain('在籍の有無')
    const employedSelect = wrapper.find('[data-test="employed-select"]').find('select')
    expect(employedSelect.exists)

    const contractTypeLabel = wrapper.find('[data-test="contract-type-label"]')
    expect(contractTypeLabel.exists)
    expect(contractTypeLabel.text()).toContain('雇用形態')
    const contractTypeSelect = wrapper.find('[data-test="contract-type-select"]').find('select')
    expect(contractTypeSelect.exists)

    const professionLabel = wrapper.find('[data-test="profession-label"]')
    expect(professionLabel.exists)
    expect(professionLabel.text()).toContain('職種（例 ITエンジニア）')
    const professionInput = wrapper.find('[data-test="profession-input"]').find('input')
    expect(professionInput.exists)

    const annualIncomeInManYenLabel = wrapper.find('[data-test="annual-income-in-man-yen-label"]')
    expect(annualIncomeInManYenLabel.exists)
    expect(annualIncomeInManYenLabel.text()).toContain('年収（単位：万円）')
    const annualIncomeInManYenEqualOrMoreInput = wrapper.find('[data-test="annual-income-in-man-yen-equal-or-more-input"]').find('input')
    expect(annualIncomeInManYenEqualOrMoreInput.exists)
    const annualIncomeInManYenEqualOrMoreLabel = wrapper.find('[data-test="annual-income-in-man-yen-equal-or-more-label"]')
    expect(annualIncomeInManYenEqualOrMoreLabel.exists)
    expect(annualIncomeInManYenEqualOrMoreLabel.text()).toContain('万円以上')
    const annualIncomeInManYenEqualOrLessInput = wrapper.find('[data-test="annual-income-in-man-yen-equal-or-less-input"]').find('input')
    expect(annualIncomeInManYenEqualOrLessInput.exists)
    const annualIncomeInManYenEqualOrLessLabel = wrapper.find('[data-test="annual-income-in-man-yen-equal-or-less-label"]')
    expect(annualIncomeInManYenEqualOrLessLabel.exists)
    expect(annualIncomeInManYenEqualOrLessLabel.text()).toContain('万円以下')

    const isManagerLabel = wrapper.find('[data-test="is-manager-label"]')
    expect(isManagerLabel.exists)
    expect(isManagerLabel.text()).toContain('管理職区分')
    const isManagerSelect = wrapper.find('[data-test="is-manager-select"]').find('select')
    expect(isManagerSelect.exists)

    const positionNameLabel = wrapper.find('[data-test="position-name-label"]')
    expect(positionNameLabel.exists)
    expect(positionNameLabel.text()).toContain('職位（例 係長）')
    const positionNameInput = wrapper.find('[data-test="position-name-input"]').find('input')
    expect(positionNameInput.exists)

    const isNewGraduateLabel = wrapper.find('[data-test="is-new-graduate-label"]')
    expect(isNewGraduateLabel.exists)
    expect(isNewGraduateLabel.text()).toContain('入社区分')
    const isNewGraduateSelect = wrapper.find('[data-test="is-new-graduate-select"]').find('select')
    expect(isNewGraduateSelect.exists)

    const noteLabel = wrapper.find('[data-test="note-label"]')
    expect(noteLabel.exists)
    expect(noteLabel.text()).toContain('備考')
    const noteInput = wrapper.find('[data-test="note-input"]').find('input')
    expect(noteInput.exists)

    const feePerHourInYenLabel = wrapper.find('[data-test="fee-per-hour-in-yen-label"]')
    expect(feePerHourInYenLabel.exists)
    expect(feePerHourInYenLabel.text()).toContain('相談一回（１時間）の相談料（単位：円）')
    const feePerHourInYenEqualOrMoreInput = wrapper.find('[data-test="fee-per-hour-in-yen-equal-or-more-input"]').find('input')
    expect(feePerHourInYenEqualOrMoreInput.exists)
    const feePerHourInYenEqualOrMoreLabel = wrapper.find('[data-test="fee-per-hour-in-yen-equal-or-more-label"]')
    expect(feePerHourInYenEqualOrMoreLabel.exists)
    expect(feePerHourInYenEqualOrMoreLabel.text()).toContain('円以上')
    const feePerHourInYenEqualOrLessInput = wrapper.find('[data-test="fee-per-hour-in-yen-equal-or-less-input"]').find('input')
    expect(feePerHourInYenEqualOrLessInput.exists)
    const feePerHourInYenEqualOrLessLabel = wrapper.find('[data-test="fee-per-hour-in-yen-equal-or-less-label"]')
    expect(feePerHourInYenEqualOrLessLabel.exists)
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
})
