import { RouterLinkStub, mount } from '@vue/test-utils'
import flushPromises from 'flush-promises'
import RequestConsultationPage from '@/views/personalized/RequestConsultationPage.vue'
import { ref } from 'vue'
import { GetFeePerHourInYenForApplicationResp } from '@/util/personalized/request-consultation/GetFeePerHourInYenForApplicationResp'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { Message } from '@/util/Message'
import { Code } from '@/util/Error'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { getMinDurationBeforeConsultationInDays, getMaxDurationBeforeConsultationInDays } from '@/util/personalized/request-consultation/DurationBeforeConsultation'
import { createYearList } from '@/util/personalized/request-consultation/YearList'
import { createMonthList } from '@/util/personalized/request-consultation/MonthList'
import { postRequestConsultation } from '@/util/personalized/request-consultation/PostRequestConsultation'
import { postFinishRequestConsultation } from '@/util/personalized/request-consultation/PostFinishRequestConsultation'
import { PostRequestConsultationResp } from '@/util/personalized/request-consultation/PostRequestConsultationResp'

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

const getFeePerHourInYenForApplicationDoneMock = ref(true)
const getFeePerHourInYenForApplicationFuncMock = jest.fn()
jest.mock('@/util/personalized/request-consultation/useGetFeePerHourInYenForApplication', () => ({
  useGetFeePerHourInYenForApplication: () => ({
    getFeePerHourInYenForApplicationDone: getFeePerHourInYenForApplicationDoneMock,
    getFeePerHourInYenForApplicationFunc: getFeePerHourInYenForApplicationFuncMock
  })
}))

const requestConsultationDoneMock = ref(true)
const startRequestConsultationMock = jest.fn()
const finishRequestConsultationMock = jest.fn()
const disabledMock = ref(false)
const disableBtnMock = jest.fn()
const enableBtnMock = jest.fn()
jest.mock('@/util/personalized/request-consultation/useRequestConsultationDone', () => ({
  useRequestConsultationDone: () => ({
    requestConsultationDone: requestConsultationDoneMock,
    startRequestConsultation: startRequestConsultationMock,
    finishRequestConsultation: finishRequestConsultationMock,
    disabled: disabledMock,
    disableBtn: disableBtnMock,
    enableBtn: enableBtnMock
  })
}))

// PAY.JPから型定義が提供されていないため、anyでの扱いを許容する
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let payJpMock = null as any | null
const storeCommitMock = jest.fn()
jest.mock('vuex', () => ({
  useStore: () => ({
    commit: storeCommitMock,
    state: {
      payJp: payJpMock
    }
  })
}))

// ルートフォントサイズ（環境に依存する値）は、テストでは固定値として扱う
const fontSize = 16
jest.mock('@/util/personalized/request-consultation/FontSizeConverter', () => ({
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  convertRemToPx: (rem: number): number => {
    return rem * parseFloat(fontSize.toString() + 'px')
  }
}))

let createTokenErr = false
const createTokenErrMessage = 'createToken Error'
const dummyChargeId = 'ch_fa990a4c10672a93053a774730b0a'
// PAY.JPから型定義が提供されていないため、anyでの扱いを許容する
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const createPayJpMockObject = {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  elements: (): any => {
    return {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      create: (type: string, options?: object): any => {
        expect(type).toBe('card')
        expect(options).toStrictEqual({
          style: {
            base: {
              color: 'black',
              fontSize: (fontSize * 1.5) + 'px'
            },
            invalid: {
              color: 'red'
            }
          }
        })
        return {
          mount: (domElement: string) => {
            expect(domElement).toBe('#payjp-card-area')
          }
        }
      }
    }
  },
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  createToken: async (cardElement: any): Promise<any> => {
    expect(cardElement).not.toBeNull()
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    let obj: any
    if (createTokenErr) {
      obj = {
        error: {
          message: createTokenErrMessage
        }
      }
    } else {
      obj = {
        id: 'tok_76e202b409f3da51a0706605ac81'
      }
    }
    return new Promise((resolve) => {
      resolve(obj)
    })
  },
  openThreeDSecureDialog: async (chargeId: string) => {
    expect(chargeId).toBe(dummyChargeId)
  }
}
jest.mock('@/util/PayJp', () => ({
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  createPayJp: async (): Promise<any> => {
    return new Promise((resolve) => {
      resolve(createPayJpMockObject)
    })
  }
}))

let currentYear = 2022
let currentMonth = 11
let currentDay = 1
let currentHour = 7
let currentMinute = 0
let currentSecond = 0
jest.mock('@/util/personalized/request-consultation/CurrentDateTime', () => ({
  getCurrentYear: (): number => {
    return currentYear
  },
  getCurrentMonth: (): number => {
    return currentMonth
  },
  getCurrentDate: (): Date => {
    const zeroIndexedCurrentMonth = currentMonth - 1
    const date = new Date(currentYear, zeroIndexedCurrentMonth, currentDay, currentHour, currentMinute, currentSecond)
    return date
  }
}))

jest.mock('@/util/personalized/request-consultation/PostRequestConsultation')
const postRequestConsultationMock = postRequestConsultation as jest.MockedFunction<typeof postRequestConsultation>

jest.mock('@/util/personalized/request-consultation/PostFinishRequestConsultation')
const postFinishRequestConsultationMock = postFinishRequestConsultation as jest.MockedFunction<typeof postFinishRequestConsultation>

describe('RequestConsultationPage.vue', () => {
  beforeEach(() => {
    routeParam = '1'
    routerPushMock.mockClear()
    getFeePerHourInYenForApplicationDoneMock.value = true
    getFeePerHourInYenForApplicationFuncMock.mockReset()
    requestConsultationDoneMock.value = true
    startRequestConsultationMock.mockReset()
    finishRequestConsultationMock.mockReset()
    disabledMock.value = false
    disableBtnMock.mockReset()
    enableBtnMock.mockReset()
    payJpMock = null
    createTokenErr = false
    currentYear = 2022
    currentMonth = 11
    currentDay = 1
    currentHour = 7
    currentMinute = 0
    currentSecond = 0
    postRequestConsultationMock.mockReset()
    postFinishRequestConsultationMock.mockReset()
  })

  it('has WaitingCircle and TheHeader while waiting response of fee per hour in yen', async () => {
    getFeePerHourInYenForApplicationDoneMock.value = false
    const resp = GetFeePerHourInYenForApplicationResp.create(5000)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
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
    expect(waitingCircles.length).toBe(1)
    // ユーザーに待ち時間を表すためにWaitingCircleが出ていることが確認できれば十分のため、
    // mainが出ていないことまで確認しない。
  })

  it('does not display AlertMessage when error does not occur', async () => {
    const resp = GetFeePerHourInYenForApplicationResp.create(5000)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(0)
  })

  it('displays AlertMessage when error has happened', async () => {
    const errDetail = 'connection error'
    getFeePerHourInYenForApplicationFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const alertMessage = wrapper.find('[data-test="outer-alert-message"]').findComponent(AlertMessage)
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.UNEXPECTED_ERR)
    expect(resultMessage).toContain(errDetail)
  })

  it(`moves to login if ${Code.UNAUTHORIZED} is returned on opening page`, async () => {
    const resp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    mount(RequestConsultationPage, {
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

  it(`moves to terms-of-use if ${Code.NOT_TERMS_OF_USE_AGREED_YET} is returned on opening page`, async () => {
    const resp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    mount(RequestConsultationPage, {
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

  it('displays information for application (payJp object is not stored)', async () => {
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const description = wrapper.find('[data-test="description"]')
    const descpritionMessage = `相談開始日時に関して、第一希望、第二希望、第三希望を入力して下さい。申し込み可能な相談開始日時は、申し込み日時から${getMinDurationBeforeConsultationInDays() * 24}時間（${getMinDurationBeforeConsultationInDays()}日）以降、${getMaxDurationBeforeConsultationInDays() * 24}時間（${getMaxDurationBeforeConsultationInDays()}日）以前までとなります。`
    expect(description.text()).toContain(descpritionMessage)

    const firstCandidateLabel = wrapper.find('[data-test="first-candidate-lablel"]')
    expect(firstCandidateLabel.text()).toContain('相談開始日時（第一希望）')
    const firstCandidateYearLabel = wrapper.find('[data-test="first-candidate-year-lablel"]')
    expect(firstCandidateYearLabel.text()).toContain('年')
    const firstCandidateMonthLabel = wrapper.find('[data-test="first-candidate-month-lablel"]')
    expect(firstCandidateMonthLabel.text()).toContain('月')
    const firstCandidateDayLabel = wrapper.find('[data-test="first-candidate-day-lablel"]')
    expect(firstCandidateDayLabel.text()).toContain('日')
    const firstCandidateHourLabel = wrapper.find('[data-test="first-candidate-hour-lablel"]')
    expect(firstCandidateHourLabel.text()).toContain('時')

    const secondCandidateLabel = wrapper.find('[data-test="second-candidate-lablel"]')
    expect(secondCandidateLabel.text()).toContain('相談開始日時（第二希望）')
    const secondCandidateYearLabel = wrapper.find('[data-test="second-candidate-year-lablel"]')
    expect(secondCandidateYearLabel.text()).toContain('年')
    const secondCandidateMonthLabel = wrapper.find('[data-test="second-candidate-month-lablel"]')
    expect(secondCandidateMonthLabel.text()).toContain('月')
    const secondCandidateDayLabel = wrapper.find('[data-test="second-candidate-day-lablel"]')
    expect(secondCandidateDayLabel.text()).toContain('日')
    const secondCandidateHourLabel = wrapper.find('[data-test="second-candidate-hour-lablel"]')
    expect(secondCandidateHourLabel.text()).toContain('時')

    const thirdCandidateLabel = wrapper.find('[data-test="third-candidate-lablel"]')
    expect(thirdCandidateLabel.text()).toContain('相談開始日時（第三希望）')
    const thirdCandidateYearLabel = wrapper.find('[data-test="third-candidate-year-lablel"]')
    expect(thirdCandidateYearLabel.text()).toContain('年')
    const thirdCandidateMonthLabel = wrapper.find('[data-test="third-candidate-month-lablel"]')
    expect(thirdCandidateMonthLabel.text()).toContain('月')
    const thirdCandidateDayLabel = wrapper.find('[data-test="third-candidate-day-lablel"]')
    expect(thirdCandidateDayLabel.text()).toContain('日')
    const thirdCandidateHourLabel = wrapper.find('[data-test="third-candidate-hour-lablel"]')
    expect(thirdCandidateHourLabel.text()).toContain('時')

    const consultationDetail = wrapper.find('[data-test="consultation-detail"]')
    expect(consultationDetail.text()).toContain('相談申し込み詳細')

    const consultantId = wrapper.find('[data-test="consultant-id"]')
    expect(consultantId.text()).toContain('コンサルタントID')
    const consultantIdValue = wrapper.find('[data-test="consultant-id-value"]')
    expect(consultantIdValue.text()).toContain(`${routeParam}`)

    const feePerHourInYen = wrapper.find('[data-test="fee-per-hour-in-yen"]')
    expect(feePerHourInYen.text()).toContain('相談一回（１時間）の相談料')
    const feePerHourInYenValue = wrapper.find('[data-test="fee-per-hour-in-yen-value"]')
    expect(feePerHourInYenValue.text()).toContain(`${fee}円`)

    const cardLabel = wrapper.find('[data-test="card-label"]')
    expect(cardLabel.text()).toContain('クレジットカード')
    const cardArea = wrapper.find('[data-test="card-area"]')
    expect(cardArea.exists()).toBe(true)

    const notice = wrapper.find('[data-test="notice"]')
    expect(notice.text()).toContain('相談申し込み後にキャンセルや相談開始日時変更は出来ませんので、申し込み内容についてよくご確認の上、相談をお申し込み下さい。')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)

    const innerAlert = wrapper.find('[data-test="inner-alert-message"]')
    // ページが表示されたタイミングではエラーはない
    expect(innerAlert.exists()).toBe(false)
  })

  it('displays information for application (payJp object has been already stored)', async () => {
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const description = wrapper.find('[data-test="description"]')
    const descpritionMessage = `相談開始日時に関して、第一希望、第二希望、第三希望を入力して下さい。申し込み可能な相談開始日時は、申し込み日時から${getMinDurationBeforeConsultationInDays() * 24}時間（${getMinDurationBeforeConsultationInDays()}日）以降、${getMaxDurationBeforeConsultationInDays() * 24}時間（${getMaxDurationBeforeConsultationInDays()}日）以前までとなります。`
    expect(description.text()).toContain(descpritionMessage)

    const firstCandidateLabel = wrapper.find('[data-test="first-candidate-lablel"]')
    expect(firstCandidateLabel.text()).toContain('相談開始日時（第一希望）')
    const firstCandidateYearLabel = wrapper.find('[data-test="first-candidate-year-lablel"]')
    expect(firstCandidateYearLabel.text()).toContain('年')
    const firstCandidateMonthLabel = wrapper.find('[data-test="first-candidate-month-lablel"]')
    expect(firstCandidateMonthLabel.text()).toContain('月')
    const firstCandidateDayLabel = wrapper.find('[data-test="first-candidate-day-lablel"]')
    expect(firstCandidateDayLabel.text()).toContain('日')
    const firstCandidateHourLabel = wrapper.find('[data-test="first-candidate-hour-lablel"]')
    expect(firstCandidateHourLabel.text()).toContain('時')

    const secondCandidateLabel = wrapper.find('[data-test="second-candidate-lablel"]')
    expect(secondCandidateLabel.text()).toContain('相談開始日時（第二希望）')
    const secondCandidateYearLabel = wrapper.find('[data-test="second-candidate-year-lablel"]')
    expect(secondCandidateYearLabel.text()).toContain('年')
    const secondCandidateMonthLabel = wrapper.find('[data-test="second-candidate-month-lablel"]')
    expect(secondCandidateMonthLabel.text()).toContain('月')
    const secondCandidateDayLabel = wrapper.find('[data-test="second-candidate-day-lablel"]')
    expect(secondCandidateDayLabel.text()).toContain('日')
    const secondCandidateHourLabel = wrapper.find('[data-test="second-candidate-hour-lablel"]')
    expect(secondCandidateHourLabel.text()).toContain('時')

    const thirdCandidateLabel = wrapper.find('[data-test="third-candidate-lablel"]')
    expect(thirdCandidateLabel.text()).toContain('相談開始日時（第三希望）')
    const thirdCandidateYearLabel = wrapper.find('[data-test="third-candidate-year-lablel"]')
    expect(thirdCandidateYearLabel.text()).toContain('年')
    const thirdCandidateMonthLabel = wrapper.find('[data-test="third-candidate-month-lablel"]')
    expect(thirdCandidateMonthLabel.text()).toContain('月')
    const thirdCandidateDayLabel = wrapper.find('[data-test="third-candidate-day-lablel"]')
    expect(thirdCandidateDayLabel.text()).toContain('日')
    const thirdCandidateHourLabel = wrapper.find('[data-test="third-candidate-hour-lablel"]')
    expect(thirdCandidateHourLabel.text()).toContain('時')

    const consultationDetail = wrapper.find('[data-test="consultation-detail"]')
    expect(consultationDetail.text()).toContain('相談申し込み詳細')

    const consultantId = wrapper.find('[data-test="consultant-id"]')
    expect(consultantId.text()).toContain('コンサルタントID')
    const consultantIdValue = wrapper.find('[data-test="consultant-id-value"]')
    expect(consultantIdValue.text()).toContain(`${routeParam}`)

    const feePerHourInYen = wrapper.find('[data-test="fee-per-hour-in-yen"]')
    expect(feePerHourInYen.text()).toContain('相談一回（１時間）の相談料')
    const feePerHourInYenValue = wrapper.find('[data-test="fee-per-hour-in-yen-value"]')
    expect(feePerHourInYenValue.text()).toContain(`${fee}円`)

    const cardLabel = wrapper.find('[data-test="card-label"]')
    expect(cardLabel.text()).toContain('クレジットカード')
    const cardArea = wrapper.find('[data-test="card-area"]')
    expect(cardArea.exists()).toBe(true)

    const notice = wrapper.find('[data-test="notice"]')
    expect(notice.text()).toContain('相談申し込み後にキャンセルや相談開始日時変更は出来ませんので、申し込み内容についてよくご確認の上、相談をお申し込み下さい。')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)

    const innerAlert = wrapper.find('[data-test="inner-alert-message"]')
    // ページが表示されたタイミングではエラーはない
    expect(innerAlert.exists()).toBe(false)
  })

  it('uses createYearList (case 1)', () => {
    const yearList = createYearList(11, 2022)
    expect(yearList).toStrictEqual(['', '2022'])
  })

  it('uses createYearList (case 2)', () => {
    const yearList = createYearList(12, 2022)
    expect(yearList).toStrictEqual(['', '2022', '2023'])
  })

  it('uses createMonthList (case 1)', () => {
    const monthList = createMonthList(11)
    expect(monthList).toStrictEqual(['', '11', '12'])
  })

  it('uses createMonthList (case 2)', () => {
    const monthList = createMonthList(12)
    expect(monthList).toStrictEqual(['', '12', '1'])
  })

  it(`displays ${Message.NOT_ALL_CANDIDATES_ARE_INPUT_MESSAGE} when necessary input is lack (case 1)`, async () => {
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const innerAlert = wrapper.find('[data-test="inner-alert-message"]')
    expect(innerAlert.exists()).toBe(true)
    expect(innerAlert.text()).toContain(Message.NOT_ALL_CANDIDATES_ARE_INPUT_MESSAGE)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays ${Message.NOT_ALL_CANDIDATES_ARE_INPUT_MESSAGE} when necessary input is lack (case 2)`, async () => {
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('21')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('4')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('4')
    // 必須入力項目の内、一つだけ空のままとする
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const innerAlert = wrapper.find('[data-test="inner-alert-message"]')
    expect(innerAlert.exists()).toBe(true)
    expect(innerAlert.text()).toContain(Message.NOT_ALL_CANDIDATES_ARE_INPUT_MESSAGE)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays ${Message.DUPLICATE_DATE_TIME_CANDIDATES_MESSAGE} when same candidates exist (case 1)`, async () => {
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('4')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('4')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('7')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const innerAlert = wrapper.find('[data-test="inner-alert-message"]')
    expect(innerAlert.exists()).toBe(true)
    expect(innerAlert.text()).toContain(Message.DUPLICATE_DATE_TIME_CANDIDATES_MESSAGE)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays ${Message.DUPLICATE_DATE_TIME_CANDIDATES_MESSAGE} when same candidates exist (case 2)`, async () => {
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('21')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('21')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('4')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const innerAlert = wrapper.find('[data-test="inner-alert-message"]')
    expect(innerAlert.exists()).toBe(true)
    expect(innerAlert.text()).toContain(Message.DUPLICATE_DATE_TIME_CANDIDATES_MESSAGE)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays ${Message.DUPLICATE_DATE_TIME_CANDIDATES_MESSAGE} when same candidates exist (case 3)`, async () => {
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('21')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('4')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('4')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('7')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const innerAlert = wrapper.find('[data-test="inner-alert-message"]')
    expect(innerAlert.exists()).toBe(true)
    expect(innerAlert.text()).toContain(Message.DUPLICATE_DATE_TIME_CANDIDATES_MESSAGE)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays ${Message.DUPLICATE_DATE_TIME_CANDIDATES_MESSAGE} when same candidates exist (case 4)`, async () => {
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('21')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('4')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('21')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('7')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const innerAlert = wrapper.find('[data-test="inner-alert-message"]')
    expect(innerAlert.exists()).toBe(true)
    expect(innerAlert.text()).toContain(Message.DUPLICATE_DATE_TIME_CANDIDATES_MESSAGE)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays ${Message.INVALID_CONSULTATION_DATE_TIME_MESSAGE} when candidate has out of valid range value (case 1)`, async () => {
    currentYear = 2022
    currentMonth = 11
    currentDay = 1
    currentHour = 7
    currentMinute = 0
    currentSecond = 1

    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('21')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('21')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const innerAlert = wrapper.find('[data-test="inner-alert-message"]')
    expect(innerAlert.exists()).toBe(true)
    expect(innerAlert.text()).toContain(Message.INVALID_CONSULTATION_DATE_TIME_MESSAGE)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays ${Message.INVALID_CONSULTATION_DATE_TIME_MESSAGE} when candidate has out of valid range value (case 2)`, async () => {
    currentYear = 2022
    currentMonth = 11
    currentDay = 1
    currentHour = 22
    currentMinute = 59
    currentSecond = 59

    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('22')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('23')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('22')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('7')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const innerAlert = wrapper.find('[data-test="inner-alert-message"]')
    expect(innerAlert.exists()).toBe(true)
    expect(innerAlert.text()).toContain(Message.INVALID_CONSULTATION_DATE_TIME_MESSAGE)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays ${Message.INVALID_CONSULTATION_DATE_TIME_MESSAGE} when candidate has out of valid range value (case 3)`, async () => {
    currentYear = 2022
    currentMonth = 11
    currentDay = 1
    currentHour = 22
    currentMinute = 59
    currentSecond = 59

    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('22')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('22')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const innerAlert = wrapper.find('[data-test="inner-alert-message"]')
    expect(innerAlert.exists()).toBe(true)
    expect(innerAlert.text()).toContain(Message.INVALID_CONSULTATION_DATE_TIME_MESSAGE)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays ${Message.INVALID_CONSULTATION_DATE_TIME_MESSAGE} when candidate has out of valid range value (case 4)`, async () => {
    currentYear = 2022
    currentMonth = 11
    currentDay = 1
    currentHour = 7
    currentMinute = 0
    currentSecond = 1

    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('3')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('2')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('7')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const innerAlert = wrapper.find('[data-test="inner-alert-message"]')
    expect(innerAlert.exists()).toBe(true)
    expect(innerAlert.text()).toContain(Message.INVALID_CONSULTATION_DATE_TIME_MESSAGE)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it('displays error message when payjp.createToken has error', async () => {
    createTokenErr = true
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('21')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('21')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const innerAlert = wrapper.find('[data-test="inner-alert-message"]')
    expect(innerAlert.exists()).toBe(true)
    expect(innerAlert.text()).toContain(createTokenErrMessage)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(0)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays ${Message.UNAUTHORIZED_ON_CARD_OPERATION_MESSAGE} when ${Code.UNAUTHORIZED} is returned on postRequestConsultation`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    postRequestConsultationMock.mockResolvedValue(apiErrResp)
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('21')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('21')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const outerAlert = wrapper.find('[data-test="outer-alert-message"]')
    expect(outerAlert.exists()).toBe(true)
    expect(outerAlert.text()).toContain(`${Message.UNAUTHORIZED_ON_CARD_OPERATION_MESSAGE} (${Code.UNAUTHORIZED})`)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays ${Message.NOT_TERMS_OF_USE_AGREED_YET_ON_CARD_OPERATION_MESSAGE} when ${Code.NOT_TERMS_OF_USE_AGREED_YET} is returned on postRequestConsultation`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    postRequestConsultationMock.mockResolvedValue(apiErrResp)
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('21')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('21')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const outerAlert = wrapper.find('[data-test="outer-alert-message"]')
    expect(outerAlert.exists()).toBe(true)
    expect(outerAlert.text()).toContain(`${Message.NOT_TERMS_OF_USE_AGREED_YET_ON_CARD_OPERATION_MESSAGE} (${Code.NOT_TERMS_OF_USE_AGREED_YET})`)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays ${Message.REACH_PAYMENT_PLATFORM_RATE_LIMIT_MESSAGE} when ${Code.REACH_PAYMENT_PLATFORM_RATE_LIMIT} is returned on postRequestConsultation`, async () => {
    const apiErrResp = ApiErrorResp.create(429, ApiError.create(Code.REACH_PAYMENT_PLATFORM_RATE_LIMIT))
    postRequestConsultationMock.mockResolvedValue(apiErrResp)
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('21')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('21')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const outerAlert = wrapper.find('[data-test="outer-alert-message"]')
    expect(outerAlert.exists()).toBe(true)
    expect(outerAlert.text()).toContain(`${Message.REACH_PAYMENT_PLATFORM_RATE_LIMIT_MESSAGE} (${Code.REACH_PAYMENT_PLATFORM_RATE_LIMIT})`)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays ${Message.FEE_PER_HOUR_IN_YEN_WAS_UPDATED_MESSAGE} when ${Code.FEE_PER_HOUR_IN_YEN_WAS_UPDATED} is returned on postRequestConsultation`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.FEE_PER_HOUR_IN_YEN_WAS_UPDATED))
    postRequestConsultationMock.mockResolvedValue(apiErrResp)
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('21')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('21')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const outerAlert = wrapper.find('[data-test="outer-alert-message"]')
    expect(outerAlert.exists()).toBe(true)
    expect(outerAlert.text()).toContain(`${Message.FEE_PER_HOUR_IN_YEN_WAS_UPDATED_MESSAGE} (${Code.FEE_PER_HOUR_IN_YEN_WAS_UPDATED})`)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  // 通常操作で発生し得ないが、ユーザーが無理やりリクエストを偽造することで可能
  // そのため、仕様を示す意味でテストを記載
  it(`displays ${Message.NON_POSITIVE_CONSULTANT_ID_MESSAGE} when ${Code.NON_POSITIVE_CONSULTANT_ID} is returned on postRequestConsultation`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NON_POSITIVE_CONSULTANT_ID))
    postRequestConsultationMock.mockResolvedValue(apiErrResp)
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('21')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('21')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const outerAlert = wrapper.find('[data-test="outer-alert-message"]')
    expect(outerAlert.exists()).toBe(true)
    expect(outerAlert.text()).toContain(`${Message.NON_POSITIVE_CONSULTANT_ID_MESSAGE} (${Code.NON_POSITIVE_CONSULTANT_ID})`)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  // 通常操作で発生し得ないが、ユーザーが無理やりリクエストを偽造することで可能
  // そのため、仕様を示す意味でテストを記載
  it(`displays ${Message.DUPLICATE_DATE_TIME_CANDIDATES_MESSAGE} when ${Code.DUPLICATE_DATE_TIME_CANDIDATES} is returned on postRequestConsultation`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.DUPLICATE_DATE_TIME_CANDIDATES))
    postRequestConsultationMock.mockResolvedValue(apiErrResp)
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('21')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('21')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const outerAlert = wrapper.find('[data-test="outer-alert-message"]')
    expect(outerAlert.exists()).toBe(true)
    expect(outerAlert.text()).toContain(`${Message.DUPLICATE_DATE_TIME_CANDIDATES_MESSAGE} (${Code.DUPLICATE_DATE_TIME_CANDIDATES})`)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  // 通常操作で発生し得ないが、ユーザーが無理やりリクエストを偽造することで可能
  // そのため、仕様を示す意味でテストを記載
  it(`displays ${Message.ILLEGAL_CONSULTATION_DATE_TIME_MESSAGE} when ${Code.ILLEGAL_CONSULTATION_DATE_TIME} is returned on postRequestConsultation`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CONSULTATION_DATE_TIME))
    postRequestConsultationMock.mockResolvedValue(apiErrResp)
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('21')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('21')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const outerAlert = wrapper.find('[data-test="outer-alert-message"]')
    expect(outerAlert.exists()).toBe(true)
    expect(outerAlert.text()).toContain(`${Message.ILLEGAL_CONSULTATION_DATE_TIME_MESSAGE} (${Code.ILLEGAL_CONSULTATION_DATE_TIME})`)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  // 通常操作で発生し得ないが、ユーザーが無理やりリクエストを偽造することで可能
  // そのため、仕様を示す意味でテストを記載
  it(`displays ${Message.ILLEGAL_CONSULTATION_HOUR_MESSAGE} when ${Code.ILLEGAL_CONSULTATION_HOUR} is returned on postRequestConsultation`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CONSULTATION_HOUR))
    postRequestConsultationMock.mockResolvedValue(apiErrResp)
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('21')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('21')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const outerAlert = wrapper.find('[data-test="outer-alert-message"]')
    expect(outerAlert.exists()).toBe(true)
    expect(outerAlert.text()).toContain(`${Message.ILLEGAL_CONSULTATION_HOUR_MESSAGE} (${Code.ILLEGAL_CONSULTATION_HOUR})`)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  // 通常操作で発生し得ないが、ユーザーが無理やりリクエストを偽造することで可能
  // そのため、仕様を示す意味でテストを記載
  it(`displays ${Message.INVALID_CONSULTATION_DATE_TIME_MESSAGE} when ${Code.INVALID_CONSULTATION_DATE_TIME} is returned on postRequestConsultation`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_CONSULTATION_DATE_TIME))
    postRequestConsultationMock.mockResolvedValue(apiErrResp)
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('21')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('21')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const outerAlert = wrapper.find('[data-test="outer-alert-message"]')
    expect(outerAlert.exists()).toBe(true)
    expect(outerAlert.text()).toContain(`${Message.INVALID_CONSULTATION_DATE_TIME_MESSAGE} (${Code.INVALID_CONSULTATION_DATE_TIME})`)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  // 通常操作で発生し得ないが、ユーザーが無理やりリクエストを偽造することで可能
  // そのため、仕様を示す意味でテストを記載
  it(`displays ${Message.NO_IDENTITY_REGISTERED_MESSAGE} when ${Code.NO_IDENTITY_REGISTERED} is returned on postRequestConsultation`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_IDENTITY_REGISTERED))
    postRequestConsultationMock.mockResolvedValue(apiErrResp)
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('21')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('21')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const outerAlert = wrapper.find('[data-test="outer-alert-message"]')
    expect(outerAlert.exists()).toBe(true)
    expect(outerAlert.text()).toContain(`${Message.NO_IDENTITY_REGISTERED_MESSAGE} (${Code.NO_IDENTITY_REGISTERED})`)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  // 通常操作で発生し得ないが、ユーザーが無理やりリクエストを偽造することで可能
  // そのため、仕様を示す意味でテストを記載
  it(`displays ${Message.CONSULTANT_IS_NOT_AVAILABLE_MESSAGE} when ${Code.CONSULTANT_IS_NOT_AVAILABLE} is returned on postRequestConsultation`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.CONSULTANT_IS_NOT_AVAILABLE))
    postRequestConsultationMock.mockResolvedValue(apiErrResp)
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('21')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('21')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const outerAlert = wrapper.find('[data-test="outer-alert-message"]')
    expect(outerAlert.exists()).toBe(true)
    expect(outerAlert.text()).toContain(`${Message.CONSULTANT_IS_NOT_AVAILABLE_MESSAGE} (${Code.CONSULTANT_IS_NOT_AVAILABLE})`)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays ${Message.EXCEED_MAX_ANNUAL_REWARDS_MESSAGE} when ${Code.EXCEED_MAX_ANNUAL_REWARDS} is returned on postRequestConsultation`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.EXCEED_MAX_ANNUAL_REWARDS))
    postRequestConsultationMock.mockResolvedValue(apiErrResp)
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('21')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('21')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const outerAlert = wrapper.find('[data-test="outer-alert-message"]')
    expect(outerAlert.exists()).toBe(true)
    expect(outerAlert.text()).toContain(`${Message.EXCEED_MAX_ANNUAL_REWARDS_MESSAGE} (${Code.EXCEED_MAX_ANNUAL_REWARDS})`)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays ${Message.UNAUTHORIZED_ON_CARD_OPERATION_MESSAGE} when ${Code.UNAUTHORIZED} is returned on postFinishRequestConsultation`, async () => {
    const rcResp = PostRequestConsultationResp.create(dummyChargeId)
    postRequestConsultationMock.mockResolvedValue(rcResp)
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    postFinishRequestConsultationMock.mockResolvedValue(apiErrResp)
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('21')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('21')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const outerAlert = wrapper.find('[data-test="outer-alert-message"]')
    expect(outerAlert.exists()).toBe(true)
    expect(outerAlert.text()).toContain(`${Message.UNAUTHORIZED_ON_CARD_OPERATION_MESSAGE} (${Code.UNAUTHORIZED})`)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(postRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays ${Message.NOT_TERMS_OF_USE_AGREED_YET_ON_CARD_OPERATION_MESSAGE} when ${Code.NOT_TERMS_OF_USE_AGREED_YET} is returned on postFinishRequestConsultation`, async () => {
    const rcResp = PostRequestConsultationResp.create(dummyChargeId)
    postRequestConsultationMock.mockResolvedValue(rcResp)
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    postFinishRequestConsultationMock.mockResolvedValue(apiErrResp)
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('21')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('21')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const outerAlert = wrapper.find('[data-test="outer-alert-message"]')
    expect(outerAlert.exists()).toBe(true)
    expect(outerAlert.text()).toContain(`${Message.NOT_TERMS_OF_USE_AGREED_YET_ON_CARD_OPERATION_MESSAGE} (${Code.NOT_TERMS_OF_USE_AGREED_YET})`)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(postRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays ${Message.NO_IDENTITY_REGISTERED_MESSAGE} when ${Code.NO_IDENTITY_REGISTERED} is returned on postFinishRequestConsultation`, async () => {
    const rcResp = PostRequestConsultationResp.create(dummyChargeId)
    postRequestConsultationMock.mockResolvedValue(rcResp)
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_IDENTITY_REGISTERED))
    postFinishRequestConsultationMock.mockResolvedValue(apiErrResp)
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('21')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('21')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const outerAlert = wrapper.find('[data-test="outer-alert-message"]')
    expect(outerAlert.exists()).toBe(true)
    expect(outerAlert.text()).toContain(`${Message.NO_IDENTITY_REGISTERED_MESSAGE} (${Code.NO_IDENTITY_REGISTERED})`)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(postRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays ${Message.CONSULTANT_IS_NOT_AVAILABLE_MESSAGE} when ${Code.CONSULTANT_IS_NOT_AVAILABLE} is returned on postFinishRequestConsultation`, async () => {
    const rcResp = PostRequestConsultationResp.create(dummyChargeId)
    postRequestConsultationMock.mockResolvedValue(rcResp)
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.CONSULTANT_IS_NOT_AVAILABLE))
    postFinishRequestConsultationMock.mockResolvedValue(apiErrResp)
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('21')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('21')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const outerAlert = wrapper.find('[data-test="outer-alert-message"]')
    expect(outerAlert.exists()).toBe(true)
    expect(outerAlert.text()).toContain(`${Message.CONSULTANT_IS_NOT_AVAILABLE_MESSAGE} (${Code.CONSULTANT_IS_NOT_AVAILABLE})`)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(postRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })

  it(`displays ${Message.THREE_D_SECURE_ERROR_MESSAGE} when ${Code.THREE_D_SECURE_ERROR} is returned on postFinishRequestConsultation`, async () => {
    const rcResp = PostRequestConsultationResp.create(dummyChargeId)
    postRequestConsultationMock.mockResolvedValue(rcResp)
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.THREE_D_SECURE_ERROR))
    postFinishRequestConsultationMock.mockResolvedValue(apiErrResp)
    payJpMock = createPayJpMockObject
    const fee = 5000
    const resp = GetFeePerHourInYenForApplicationResp.create(fee)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidateYear = wrapper.find('[data-test="first-candidate-year"]')
    await firstCandidateYear.setValue('2022')
    const firstCandidateMonth = wrapper.find('[data-test="first-candidate-month"]')
    await firstCandidateMonth.setValue('11')
    const firstCandidateDay = wrapper.find('[data-test="first-candidate-day"]')
    await firstCandidateDay.setValue('4')
    const firstCandidateHour = wrapper.find('[data-test="first-candidate-hour"]')
    await firstCandidateHour.setValue('7')

    const secondCandidateYear = wrapper.find('[data-test="second-candidate-year"]')
    await secondCandidateYear.setValue('2022')
    const secondCandidateMonth = wrapper.find('[data-test="second-candidate-month"]')
    await secondCandidateMonth.setValue('11')
    const secondCandidateDay = wrapper.find('[data-test="second-candidate-day"]')
    await secondCandidateDay.setValue('21')
    const secondCandidateHour = wrapper.find('[data-test="second-candidate-hour"]')
    await secondCandidateHour.setValue('7')

    const thirdCandidateYear = wrapper.find('[data-test="third-candidate-year"]')
    await thirdCandidateYear.setValue('2022')
    const thirdCandidateMonth = wrapper.find('[data-test="third-candidate-month"]')
    await thirdCandidateMonth.setValue('11')
    const thirdCandidateDay = wrapper.find('[data-test="third-candidate-day"]')
    await thirdCandidateDay.setValue('21')
    const thirdCandidateHour = wrapper.find('[data-test="third-candidate-hour"]')
    await thirdCandidateHour.setValue('23')

    const btn = wrapper.find('[data-test="apply-for-consultation-btn"]')
    expect(btn.exists()).toBe(true)
    await btn.trigger('click')
    await flushPromises()

    const outerAlert = wrapper.find('[data-test="outer-alert-message"]')
    expect(outerAlert.exists()).toBe(true)
    expect(outerAlert.text()).toContain(`${Message.THREE_D_SECURE_ERROR_MESSAGE} (${Code.THREE_D_SECURE_ERROR})`)

    expect(disableBtnMock).toHaveBeenCalledTimes(1)
    expect(enableBtnMock).toHaveBeenCalledTimes(1)
    expect(startRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(postRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(finishRequestConsultationMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
  })
})
