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
import { Code } from '@/util/Error'
import { ApiError, ApiErrorResp } from '@/util/ApiError'

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

  it(`moves to login if refresh returns ${Code.UNAUTHORIZED}`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    getConsultantDetailFuncMock.mockResolvedValue(apiErrResp)
    mount(ConsultantDetailPage, {
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
    getConsultantDetailFuncMock.mockResolvedValue(apiErrResp)
    mount(ConsultantDetailPage, {
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

  it('moves to RequestConsultationPage with consultant id', async () => {
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

    const button = wrapper.find('[data-test="move-to-request-consultantion-page-button"]')
    await button.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    const data = { name: 'RequestConsultationPage', params: { consultant_id: routeParam } }
    expect(routerPushMock).toHaveBeenCalledWith(data)
  })

  it('displays consultant detail case 1', async () => {
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

    const consultantDetailLabel = wrapper.find('[data-test="consultant-detail-label"]')
    expect(consultantDetailLabel.text()).toContain('コンサルタント詳細')

    const consultantIdLabel = wrapper.find('[data-test="consultant-id-label"]')
    expect(consultantIdLabel.text()).toContain('コンサルタントID')
    const consultantIdValue = wrapper.find('[data-test="consultant-id-value"]')
    expect(consultantIdValue.text()).toContain(`${consultant1.consultant_id}`)

    const feePerHourInYenLabel = wrapper.find('[data-test="fee-per-hour-in-yen-label"]')
    expect(feePerHourInYenLabel.text()).toContain('相談一回（１時間）の相談料')
    const feePerHourInYenValue = wrapper.find('[data-test="fee-per-hour-in-yen-value"]')
    expect(feePerHourInYenValue.text()).toContain(`${consultant1.fee_per_hour_in_yen}円`)

    const ratingLabel = wrapper.find('[data-test="rating-label"]')
    expect(ratingLabel.text()).toContain(`評価（評価件数：${consultant1.num_of_rated} 件）`)
    const ratingValue = wrapper.find('[data-test="rating-value"]')
    expect(ratingValue.text()).toContain('0')

    const careerLabel = wrapper.find('[data-test="career-label"]')
    expect(careerLabel.text()).toContain('職務経歴')

    const career0 = wrapper.find('[data-test="career-detail-0"]')

    const career0DetailLabel = career0.find('[data-test="career-detail-label"]')
    expect(career0DetailLabel.text()).toContain('職務経歴1')

    const companyNameLabel = career0.find('[data-test="company-name-label"]')
    expect(companyNameLabel.text()).toContain('勤務先名称')
    const companyNameValue = career0.find('[data-test="company-name-value"]')
    expect(companyNameValue.text()).toContain(`${consultant1.careers[0].company_name}`)

    const departmentNameLabel = career0.find('[data-test="department-name-label"]')
    expect(departmentNameLabel.exists()).toBe(false)
    const departmentNameValue = career0.find('[data-test="department-name-value"]')
    expect(departmentNameValue.exists()).toBe(false)

    const officeLabel = career0.find('[data-test="office-label"]')
    expect(officeLabel.exists()).toBe(false)
    const officeValue = career0.find('[data-test="office-value"]')
    expect(officeValue.exists()).toBe(false)

    const yearsOfServiceLabel = career0.find('[data-test="years-of-service-label"]')
    expect(yearsOfServiceLabel.text()).toContain('在籍年数')
    const yearsOfServiceValue = career0.find('[data-test="years-of-service-value"]')
    expect(yearsOfServiceValue.text()).toContain('3年未満')

    const employedLabel = career0.find('[data-test="employed-label"]')
    expect(employedLabel.text()).toContain('在籍の有無')
    const employedValue = career0.find('[data-test="employed-value"]')
    expect(employedValue.text()).toContain('在籍中')

    const contractTypeLabel = career0.find('[data-test="contract-type-label"]')
    expect(contractTypeLabel.text()).toContain('雇用形態')
    const contractTypeValue = career0.find('[data-test="contract-type-value"]')
    expect(contractTypeValue.text()).toContain('正社員')

    const professionLabel = career0.find('[data-test="profession-label"]')
    expect(professionLabel.exists()).toBe(false)
    const professionValue = career0.find('[data-test="profession-value"]')
    expect(professionValue.exists()).toBe(false)

    const annualIncomeInManYenLabel = career0.find('[data-test="annual-income-in-man-yen-label"]')
    expect(annualIncomeInManYenLabel.exists()).toBe(false)
    const annualIncomeInManYenValue = career0.find('[data-test="annual-income-in-man-yen-value"]')
    expect(annualIncomeInManYenValue.exists()).toBe(false)

    const isManagerLabel = career0.find('[data-test="is-manager-label"]')
    expect(isManagerLabel.text()).toContain('管理職区分')
    const isManagerValue = career0.find('[data-test="is-manager-value"]')
    expect(isManagerValue.text()).toContain('非管理職')

    const positionNameLabel = career0.find('[data-test="position-name-label"]')
    expect(positionNameLabel.exists()).toBe(false)
    const positionNameValue = career0.find('[data-test="position-name-value"]')
    expect(positionNameValue.exists()).toBe(false)

    const isNewGraduateLabel = career0.find('[data-test="is-new-graduate-label"]')
    expect(isNewGraduateLabel.text()).toContain('入社区分')
    const isNewGraduateValue = career0.find('[data-test="is-new-graduate-value"]')
    expect(isNewGraduateValue.text()).toContain('新卒入社')

    const noteLabel = career0.find('[data-test="note-label"]')
    expect(noteLabel.exists()).toBe(false)
    const noteValue = career0.find('[data-test="note-value"]')
    expect(noteValue.exists()).toBe(false)
  })
})
