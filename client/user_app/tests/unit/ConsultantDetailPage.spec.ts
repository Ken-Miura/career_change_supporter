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
import { FIVE_YEARS_OR_MORE_LESS_THAN_TEN_YEARS, LESS_THAN_THREE_YEARS, THREE_YEARS_OR_MORE_LESS_THAN_FIVE_YEARS } from '@/util/personalized/consultant-detail/YearsOfService'
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

const consultant2 = {
  consultant_id: 2,
  fee_per_hour_in_yen: 10000,
  rating: '4.5',
  num_of_rated: 10,
  careers: [{
    counsultant_career_detail_id: 1,
    company_name: 'テスト（株）',
    department_name: '開発部',
    office: '那覇事業所',
    years_of_service: THREE_YEARS_OR_MORE_LESS_THAN_FIVE_YEARS,
    employed: false,
    contract_type: 'contract',
    profession: 'エンジニア',
    annual_income_in_man_yen: 400,
    is_manager: true,
    position_name: '係長',
    is_new_graduate: false,
    note: `
    備考テスト
    改行１
    改行２
    改行３
    `
  } as ConsultantCareerDetail]
} as ConsultantDetail

const consultant3 = {
  consultant_id: 3,
  fee_per_hour_in_yen: 3000,
  rating: null,
  num_of_rated: 0,
  careers: [
    {
      counsultant_career_detail_id: 1,
      company_name: 'テスト１（株）',
      department_name: null,
      office: null,
      years_of_service: FIVE_YEARS_OR_MORE_LESS_THAN_TEN_YEARS,
      employed: true,
      contract_type: 'regular',
      profession: null,
      annual_income_in_man_yen: null,
      is_manager: false,
      position_name: null,
      is_new_graduate: true,
      note: null
    } as ConsultantCareerDetail,
  {
    counsultant_career_detail_id: 2,
    company_name: 'テスト２（株）',
    department_name: null,
    office: null,
    years_of_service: FIVE_YEARS_OR_MORE_LESS_THAN_TEN_YEARS,
    employed: true,
    contract_type: 'regular',
    profession: null,
    annual_income_in_man_yen: null,
    is_manager: false,
    position_name: null,
    is_new_graduate: true,
    note: null
  } as ConsultantCareerDetail,
  {
    counsultant_career_detail_id: 3,
    company_name: 'テスト３（株）',
    department_name: null,
    office: null,
    years_of_service: FIVE_YEARS_OR_MORE_LESS_THAN_TEN_YEARS,
    employed: true,
    contract_type: 'regular',
    profession: null,
    annual_income_in_man_yen: null,
    is_manager: false,
    position_name: null,
    is_new_graduate: true,
    note: null
  } as ConsultantCareerDetail,
  {
    counsultant_career_detail_id: 4,
    company_name: 'テスト４（株）',
    department_name: null,
    office: null,
    years_of_service: FIVE_YEARS_OR_MORE_LESS_THAN_TEN_YEARS,
    employed: true,
    contract_type: 'regular',
    profession: null,
    annual_income_in_man_yen: null,
    is_manager: false,
    position_name: null,
    is_new_graduate: true,
    note: null
  } as ConsultantCareerDetail,
  {
    counsultant_career_detail_id: 5,
    company_name: 'テスト５（株）',
    department_name: null,
    office: null,
    years_of_service: FIVE_YEARS_OR_MORE_LESS_THAN_TEN_YEARS,
    employed: true,
    contract_type: 'regular',
    profession: null,
    annual_income_in_man_yen: null,
    is_manager: false,
    position_name: null,
    is_new_graduate: true,
    note: null
  } as ConsultantCareerDetail,
  {
    counsultant_career_detail_id: 6,
    company_name: 'テスト６（株）',
    department_name: null,
    office: null,
    years_of_service: FIVE_YEARS_OR_MORE_LESS_THAN_TEN_YEARS,
    employed: true,
    contract_type: 'regular',
    profession: null,
    annual_income_in_man_yen: null,
    is_manager: false,
    position_name: null,
    is_new_graduate: true,
    note: null
  } as ConsultantCareerDetail,
  {
    counsultant_career_detail_id: 7,
    company_name: 'テスト７（株）',
    department_name: null,
    office: null,
    years_of_service: FIVE_YEARS_OR_MORE_LESS_THAN_TEN_YEARS,
    employed: true,
    contract_type: 'regular',
    profession: null,
    annual_income_in_man_yen: null,
    is_manager: false,
    position_name: null,
    is_new_graduate: true,
    note: null
  } as ConsultantCareerDetail,
  {
    counsultant_career_detail_id: 8,
    company_name: 'テスト８（株）',
    department_name: null,
    office: null,
    years_of_service: FIVE_YEARS_OR_MORE_LESS_THAN_TEN_YEARS,
    employed: true,
    contract_type: 'regular',
    profession: null,
    annual_income_in_man_yen: null,
    is_manager: false,
    position_name: null,
    is_new_graduate: true,
    note: null
  } as ConsultantCareerDetail
  ]
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

  it(`displays ${Message.NON_POSITIVE_CONSULTANT_ID_MESSAGE} if getConsultantDetail returns ${Code.NON_POSITIVE_CONSULTANT_ID}`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NON_POSITIVE_CONSULTANT_ID))
    getConsultantDetailFuncMock.mockResolvedValue(apiErrResp)
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
    expect(resultMessage).toContain(Message.NON_POSITIVE_CONSULTANT_ID_MESSAGE)
    expect(resultMessage).toContain(Code.NON_POSITIVE_CONSULTANT_ID.toString())
  })

  it(`displays ${Message.CONSULTANT_IS_NOT_AVAILABLE_MESSAGE} if getConsultantDetail returns ${Code.CONSULTANT_IS_NOT_AVAILABLE}`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.CONSULTANT_IS_NOT_AVAILABLE))
    getConsultantDetailFuncMock.mockResolvedValue(apiErrResp)
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
    expect(resultMessage).toContain(Message.CONSULTANT_IS_NOT_AVAILABLE_MESSAGE)
    expect(resultMessage).toContain(Code.CONSULTANT_IS_NOT_AVAILABLE.toString())
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
    expect(ratingValue.text()).toContain('0/5')

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

  it('displays consultant detail case 2', async () => {
    routeParam = '2'
    const resp = GetConsultantDetailResp.create(consultant2)
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
    expect(consultantIdValue.text()).toContain(`${consultant2.consultant_id}`)

    const feePerHourInYenLabel = wrapper.find('[data-test="fee-per-hour-in-yen-label"]')
    expect(feePerHourInYenLabel.text()).toContain('相談一回（１時間）の相談料')
    const feePerHourInYenValue = wrapper.find('[data-test="fee-per-hour-in-yen-value"]')
    expect(feePerHourInYenValue.text()).toContain(`${consultant2.fee_per_hour_in_yen}円`)

    const ratingLabel = wrapper.find('[data-test="rating-label"]')
    expect(ratingLabel.text()).toContain(`評価（評価件数：${consultant2.num_of_rated} 件）`)
    const ratingValue = wrapper.find('[data-test="rating-value"]')
    expect(ratingValue.text()).toContain(`${consultant2.rating}/5`)

    const careerLabel = wrapper.find('[data-test="career-label"]')
    expect(careerLabel.text()).toContain('職務経歴')

    const career0 = wrapper.find('[data-test="career-detail-0"]')

    const career0DetailLabel = career0.find('[data-test="career-detail-label"]')
    expect(career0DetailLabel.text()).toContain('職務経歴1')

    const companyNameLabel = career0.find('[data-test="company-name-label"]')
    expect(companyNameLabel.text()).toContain('勤務先名称')
    const companyNameValue = career0.find('[data-test="company-name-value"]')
    expect(companyNameValue.text()).toContain(`${consultant2.careers[0].company_name}`)

    const departmentNameLabel = career0.find('[data-test="department-name-label"]')
    expect(departmentNameLabel.text()).toContain('部署名')
    const departmentNameValue = career0.find('[data-test="department-name-value"]')
    expect(departmentNameValue.text()).toContain(`${consultant2.careers[0].department_name}`)

    const officeLabel = career0.find('[data-test="office-label"]')
    expect(officeLabel.text()).toContain('勤務地')
    const officeValue = career0.find('[data-test="office-value"]')
    expect(officeValue.text()).toContain(`${consultant2.careers[0].office}`)

    const yearsOfServiceLabel = career0.find('[data-test="years-of-service-label"]')
    expect(yearsOfServiceLabel.text()).toContain('在籍年数')
    const yearsOfServiceValue = career0.find('[data-test="years-of-service-value"]')
    expect(yearsOfServiceValue.text()).toContain('3年以上5年未満')

    const employedLabel = career0.find('[data-test="employed-label"]')
    expect(employedLabel.text()).toContain('在籍の有無')
    const employedValue = career0.find('[data-test="employed-value"]')
    expect(employedValue.text()).toContain('退職済')

    const contractTypeLabel = career0.find('[data-test="contract-type-label"]')
    expect(contractTypeLabel.text()).toContain('雇用形態')
    const contractTypeValue = career0.find('[data-test="contract-type-value"]')
    expect(contractTypeValue.text()).toContain('契約社員')

    const professionLabel = career0.find('[data-test="profession-label"]')
    expect(professionLabel.text()).toContain('職種')
    const professionValue = career0.find('[data-test="profession-value"]')
    expect(professionValue.text()).toContain(`${consultant2.careers[0].profession}`)

    const annualIncomeInManYenLabel = career0.find('[data-test="annual-income-in-man-yen-label"]')
    expect(annualIncomeInManYenLabel.text()).toContain('年収')
    const annualIncomeInManYenValue = career0.find('[data-test="annual-income-in-man-yen-value"]')
    expect(annualIncomeInManYenValue.text()).toContain(`${consultant2.careers[0].annual_income_in_man_yen}万円`)

    const isManagerLabel = career0.find('[data-test="is-manager-label"]')
    expect(isManagerLabel.text()).toContain('管理職区分')
    const isManagerValue = career0.find('[data-test="is-manager-value"]')
    expect(isManagerValue.text()).toContain('管理職')

    const positionNameLabel = career0.find('[data-test="position-name-label"]')
    expect(positionNameLabel.text()).toContain('職位')
    const positionNameValue = career0.find('[data-test="position-name-value"]')
    expect(positionNameValue.text()).toContain(`${consultant2.careers[0].position_name}`)

    const isNewGraduateLabel = career0.find('[data-test="is-new-graduate-label"]')
    expect(isNewGraduateLabel.text()).toContain('入社区分')
    const isNewGraduateValue = career0.find('[data-test="is-new-graduate-value"]')
    expect(isNewGraduateValue.text()).toContain('中途入社')

    const noteLabel = career0.find('[data-test="note-label"]')
    expect(noteLabel.text()).toContain('備考')
    const noteValue = career0.find('[data-test="note-value"]')
    expect(noteValue.text()).toContain(`備考テスト
    改行１
    改行２
    改行３`)
  })

  it('displays consultant detail case 3', async () => {
    routeParam = '3'
    const resp = GetConsultantDetailResp.create(consultant3)
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
    expect(consultantIdValue.text()).toContain(`${consultant3.consultant_id}`)

    const feePerHourInYenLabel = wrapper.find('[data-test="fee-per-hour-in-yen-label"]')
    expect(feePerHourInYenLabel.text()).toContain('相談一回（１時間）の相談料')
    const feePerHourInYenValue = wrapper.find('[data-test="fee-per-hour-in-yen-value"]')
    expect(feePerHourInYenValue.text()).toContain(`${consultant3.fee_per_hour_in_yen}円`)

    const ratingLabel = wrapper.find('[data-test="rating-label"]')
    expect(ratingLabel.text()).toContain(`評価（評価件数：${consultant3.num_of_rated} 件）`)
    const ratingValue = wrapper.find('[data-test="rating-value"]')
    expect(ratingValue.text()).toContain('0/5')

    const careerLabel = wrapper.find('[data-test="career-label"]')
    expect(careerLabel.text()).toContain('職務経歴')

    for (let i = 0; i < consultant3.careers.length; i++) {
      const career = wrapper.find(`[data-test="career-detail-${i}"]`)

      const career0DetailLabel = career.find('[data-test="career-detail-label"]')
      expect(career0DetailLabel.text()).toContain(`職務経歴${i + 1}`)

      const companyNameLabel = career.find('[data-test="company-name-label"]')
      expect(companyNameLabel.text()).toContain('勤務先名称')
      const companyNameValue = career.find('[data-test="company-name-value"]')
      expect(companyNameValue.text()).toContain(`${consultant3.careers[i].company_name}`)

      const departmentNameLabel = career.find('[data-test="department-name-label"]')
      expect(departmentNameLabel.exists()).toBe(false)
      const departmentNameValue = career.find('[data-test="department-name-value"]')
      expect(departmentNameValue.exists()).toBe(false)

      const officeLabel = career.find('[data-test="office-label"]')
      expect(officeLabel.exists()).toBe(false)
      const officeValue = career.find('[data-test="office-value"]')
      expect(officeValue.exists()).toBe(false)

      const yearsOfServiceLabel = career.find('[data-test="years-of-service-label"]')
      expect(yearsOfServiceLabel.text()).toContain('在籍年数')
      const yearsOfServiceValue = career.find('[data-test="years-of-service-value"]')
      expect(yearsOfServiceValue.text()).toContain('5年以上10年未満')

      const employedLabel = career.find('[data-test="employed-label"]')
      expect(employedLabel.text()).toContain('在籍の有無')
      const employedValue = career.find('[data-test="employed-value"]')
      expect(employedValue.text()).toContain('在籍中')

      const contractTypeLabel = career.find('[data-test="contract-type-label"]')
      expect(contractTypeLabel.text()).toContain('雇用形態')
      const contractTypeValue = career.find('[data-test="contract-type-value"]')
      expect(contractTypeValue.text()).toContain('正社員')

      const professionLabel = career.find('[data-test="profession-label"]')
      expect(professionLabel.exists()).toBe(false)
      const professionValue = career.find('[data-test="profession-value"]')
      expect(professionValue.exists()).toBe(false)

      const annualIncomeInManYenLabel = career.find('[data-test="annual-income-in-man-yen-label"]')
      expect(annualIncomeInManYenLabel.exists()).toBe(false)
      const annualIncomeInManYenValue = career.find('[data-test="annual-income-in-man-yen-value"]')
      expect(annualIncomeInManYenValue.exists()).toBe(false)

      const isManagerLabel = career.find('[data-test="is-manager-label"]')
      expect(isManagerLabel.text()).toContain('管理職区分')
      const isManagerValue = career.find('[data-test="is-manager-value"]')
      expect(isManagerValue.text()).toContain('非管理職')

      const positionNameLabel = career.find('[data-test="position-name-label"]')
      expect(positionNameLabel.exists()).toBe(false)
      const positionNameValue = career.find('[data-test="position-name-value"]')
      expect(positionNameValue.exists()).toBe(false)

      const isNewGraduateLabel = career.find('[data-test="is-new-graduate-label"]')
      expect(isNewGraduateLabel.text()).toContain('入社区分')
      const isNewGraduateValue = career.find('[data-test="is-new-graduate-value"]')
      expect(isNewGraduateValue.text()).toContain('新卒入社')

      const noteLabel = career.find('[data-test="note-label"]')
      expect(noteLabel.exists()).toBe(false)
      const noteValue = career.find('[data-test="note-value"]')
      expect(noteValue.exists()).toBe(false)
    }
  })
})
