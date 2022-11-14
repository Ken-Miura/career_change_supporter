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
const disabledMock = ref(true)
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

describe('RequestConsultationPage.vue', () => {
  beforeEach(() => {
    routeParam = '1'
    routerPushMock.mockClear()
    getFeePerHourInYenForApplicationDoneMock.value = true
    getFeePerHourInYenForApplicationFuncMock.mockReset()
    requestConsultationDoneMock.value = true
    startRequestConsultationMock.mockReset()
    finishRequestConsultationMock.mockReset()
    disabledMock.value = true
    disableBtnMock.mockReset()
    enableBtnMock.mockReset()
    payJpMock = null
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
  })
})
