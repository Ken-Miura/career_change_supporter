import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import CareerDetailPage from '@/views/personalized/CareerDetailPage.vue'
import TheHeader from '@/components/TheHeader.vue'
import { DeleteCareerResp } from '@/util/personalized/career-deletion-confirm/DeleteCareerResp'
import { Message } from '@/util/Message'
import { GetCareerResp } from '@/util/personalized/career-detail/GetCareerResp'
import { Career } from '@/util/personalized/Career'
import { Code } from '@/util/Error'
import { ApiError, ApiErrorResp } from '@/util/ApiError'

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

const career2 = {
  company_name: 'テスト２株式会社',
  department_name: '総務部',
  office: '三重事業所',
  career_start_date: {
    year: 1999,
    month: 4,
    day: 1
  },
  career_end_date: {
    year: 2003,
    month: 12,
    day: 31
  },
  contract_type: 'other',
  profession: '総務',
  annual_income_in_man_yen: 450,
  is_manager: true,
  position_name: '課長',
  is_new_graduate: true,
  note: '備考'
} as Career

describe('CareerDetailPage.vue', () => {
  beforeEach(() => {
    routeParam = '1'
    getCareerDoneMock.value = true
    getCareerFuncMock.mockReset()
    routerPushMock.mockClear()
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
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

  it('displays Career', async () => {
    const resp = GetCareerResp.create(career2)
    getCareerFuncMock.mockResolvedValue(resp)
    const wrapper = mount(CareerDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const careerDiv = wrapper.find('[data-test="career-set"]').text()
    expect(careerDiv).toContain('勤務先名称')
    expect(careerDiv).toContain(`${career2.company_name}`)
    expect(careerDiv).toContain('部署名')
    expect(careerDiv).toContain(`${career2.department_name}`)
    expect(careerDiv).toContain('勤務地')
    expect(careerDiv).toContain(`${career2.office}`)
    expect(careerDiv).toContain('入社日')
    expect(careerDiv).toContain(`${career2.career_start_date.year}年${career2.career_start_date.month}月${career2.career_start_date.day}日`)
    expect(careerDiv).toContain('退社日')
    const careerEndDate = career2.career_end_date
    if (!careerEndDate) {
      throw new Error('careerEndDate')
    }
    expect(careerDiv).toContain(`${careerEndDate.year}年${careerEndDate.month}月${careerEndDate.day}日`)
    expect(careerDiv).toContain('雇用形態')
    expect(careerDiv).toContain('その他') // contract_type => "その他"
    expect(careerDiv).toContain('職種')
    expect(careerDiv).toContain(`${career2.profession}`)
    expect(careerDiv).toContain('年収（単位：万円）')
    expect(careerDiv).toContain(`${career2.annual_income_in_man_yen}`)
    expect(careerDiv).toContain('管理職区分')
    expect(careerDiv).toContain('管理職') // is_manager
    expect(careerDiv).toContain('入社区分')
    expect(careerDiv).toContain('新卒入社') // is_new_graduate
    expect(careerDiv).toContain('備考')
    expect(careerDiv).toContain(`${career2.note}`)
  })

  it(`displays ${Message.NO_CAREER_TO_HANDLE_FOUND_MESSAGE} if ${Code.NO_CAREER_TO_HANDLE_FOUND} is returned`, async () => {
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
    const data = JSON.parse(`{"name": "CareerDeletionConfirmPage", "params": {"career_id": ${routeParam}}}`)
    expect(routerPushMock).toHaveBeenCalledWith(data)
  })
})
