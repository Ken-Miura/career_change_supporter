import WaitingCircle from '@/components/WaitingCircle.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import TheHeader from '@/components/TheHeader.vue'
import { RouterLinkStub, mount, flushPromises } from '@vue/test-utils'
import RateConsultantPage from '@/views/personalized/RateConsultantPage.vue'
import { Message } from '@/util/Message'
import { ref } from 'vue'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { PostConsultantRatingResp } from '@/util/personalized/rate-consultant/PostConsultantRatingResp'
import { MAX_RATING, MIN_RATING } from '@/util/personalized/RatingConstants'

const postConsultantRatingDoneMock = ref(true)
const postConsultantRatingFuncMock = jest.fn()
jest.mock('@/util/personalized/rate-consultant/usePostConsultantRating', () => ({
  usePostConsultantRating: () => ({
    postConsultantRatingDone: postConsultantRatingDoneMock,
    postConsultantRatingFunc: postConsultantRatingFuncMock
  })
}))

let consultantRatingId = ''
let consultantId = ''
let year = ''
let month = ''
let day = ''
let hour = ''
const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRoute: () => ({
    params: {
      consultant_rating_id: consultantRatingId
    },
    query: {
      'consultant-id': consultantId,
      year,
      month,
      day,
      hour
    }
  }),
  useRouter: () => ({
    push: routerPushMock
  })
}))

describe('RateConsultantPage.vue', () => {
  beforeEach(() => {
    postConsultantRatingDoneMock.value = true
    postConsultantRatingFuncMock.mockReset()
    routerPushMock.mockClear()
    consultantRatingId = ''
    consultantId = ''
    year = ''
    month = ''
    day = ''
    hour = ''
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    postConsultantRatingDoneMock.value = false
    const resp = PostConsultantRatingResp.create()
    postConsultantRatingFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RateConsultantPage, {
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

  it('displays infomation for rating on opening', async () => {
    consultantRatingId = '511'
    consultantId = '701'
    year = '2023'
    month = '3'
    day = '3'
    hour = '21'
    const wrapper = mount(RateConsultantPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const consultantRatingLabel = wrapper.find('[data-test="consultant-rating-label"]')
    expect(consultantRatingLabel.text()).toContain('相談を受け付けたコンサルタントの評価')
    const consultantRatingDescription = wrapper.find('[data-test="consultant-rating-description"]')
    expect(consultantRatingDescription.text()).toContain(`相談を行ったコンサルタントを評価して下さい。${MIN_RATING}が最も低い（悪い）評価で、${MAX_RATING}が最も高い（良い）評価となります。`)

    const consultantIdLabel = wrapper.find('[data-test="consultant-id-label"]')
    expect(consultantIdLabel.text()).toContain('コンサルタントID')
    const consultantIdValue = wrapper.find('[data-test="consultant-id-value"]')
    expect(consultantIdValue.text()).toContain(`${consultantId}`)

    const consultationDateTimeLabel = wrapper.find('[data-test="consultation-date-time-label"]')
    expect(consultationDateTimeLabel.text()).toContain('相談実施日時')
    const consultationDateTimeValue = wrapper.find('[data-test="consultation-date-time-value"]')
    expect(consultationDateTimeValue.text()).toContain(`${year}年${month}月${day}日${hour}時`)

    const ratingLabel = wrapper.find('[data-test="rating-label"]')
    expect(ratingLabel.text()).toContain('評価')
    const ratingValue = wrapper.find('[data-test="rating-value"]')
    expect(ratingValue.text()).toContain('')

    const submitButton = wrapper.find('[data-test="submit-button"]')
    const submitButtonAttr = submitButton.attributes('disabled')
    expect(submitButtonAttr).toBeDefined()
  })

  it('enables submit button after selecting rate', async () => {
    consultantRatingId = '511'
    consultantId = '701'
    year = '2023'
    month = '3'
    day = '3'
    hour = '21'
    const wrapper = mount(RateConsultantPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const consultantRatingLabel = wrapper.find('[data-test="consultant-rating-label"]')
    expect(consultantRatingLabel.text()).toContain('相談を受け付けたコンサルタントの評価')
    const consultantRatingDescription = wrapper.find('[data-test="consultant-rating-description"]')
    expect(consultantRatingDescription.text()).toContain(`相談を行ったコンサルタントを評価して下さい。${MIN_RATING}が最も低い（悪い）評価で、${MAX_RATING}が最も高い（良い）評価となります。`)

    const consultantIdLabel = wrapper.find('[data-test="consultant-id-label"]')
    expect(consultantIdLabel.text()).toContain('コンサルタントID')
    const consultantIdValue = wrapper.find('[data-test="consultant-id-value"]')
    expect(consultantIdValue.text()).toContain(`${consultantId}`)

    const consultationDateTimeLabel = wrapper.find('[data-test="consultation-date-time-label"]')
    expect(consultationDateTimeLabel.text()).toContain('相談実施日時')
    const consultationDateTimeValue = wrapper.find('[data-test="consultation-date-time-value"]')
    expect(consultationDateTimeValue.text()).toContain(`${year}年${month}月${day}日${hour}時`)

    const ratingLabel = wrapper.find('[data-test="rating-label"]')
    expect(ratingLabel.text()).toContain('評価')
    const ratingValue = wrapper.find('[data-test="rating-value"]')
    expect(ratingValue.text()).toContain('')

    const submitButton = wrapper.find('[data-test="submit-button"]')
    const submitButtonAttr1 = submitButton.attributes('disabled')
    expect(submitButtonAttr1).toBeDefined()

    const rate = 3
    const rateSelect = ratingValue.find('select')
    await rateSelect.setValue(rate)

    const submitButtonAttr2 = submitButton.attributes('disabled')
    expect(submitButtonAttr2).not.toBeDefined()
  })

  it('moves /rate-success if 評価する is clicked', async () => {
    consultantRatingId = '511'
    consultantId = '701'
    year = '2023'
    month = '3'
    day = '3'
    hour = '21'
    const resp = PostConsultantRatingResp.create()
    postConsultantRatingFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RateConsultantPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const rate = 3
    const rateSelect = wrapper.find('[data-test="rating-value"]').find('select')
    await rateSelect.setValue(rate)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('click')

    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/rate-success')
  })

  it('displays AlertMessage when error has happened', async () => {
    consultantRatingId = '511'
    consultantId = '701'
    year = '2023'
    month = '3'
    day = '3'
    hour = '21'
    const errDetail = 'connection error'
    postConsultantRatingFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(RateConsultantPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const rate = 3
    const rateSelect = wrapper.find('[data-test="rating-value"]').find('select')
    await rateSelect.setValue(rate)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('click')

    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.UNEXPECTED_ERR)
    expect(resultMessage).toContain(errDetail)
  })

  it(`moves to login if request returns ${Code.UNAUTHORIZED}`, async () => {
    consultantRatingId = '511'
    consultantId = '701'
    year = '2023'
    month = '3'
    day = '3'
    hour = '21'
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    postConsultantRatingFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(RateConsultantPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const rate = 4
    const rateSelect = wrapper.find('[data-test="rating-value"]').find('select')
    await rateSelect.setValue(rate)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('click')

    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/login')
  })

  it(`moves to login if request returns ${Code.NOT_TERMS_OF_USE_AGREED_YET}`, async () => {
    consultantRatingId = '511'
    consultantId = '701'
    year = '2023'
    month = '3'
    day = '3'
    hour = '21'
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    postConsultantRatingFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(RateConsultantPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const rate = 5
    const rateSelect = wrapper.find('[data-test="rating-value"]').find('select')
    await rateSelect.setValue(rate)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('click')

    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/terms-of-use')
  })

  it(`displays ${Message.RATING_ID_IS_NOT_POSITIVE_MESSAGE} if ${Code.RATING_ID_IS_NOT_POSITIVE} is returned`, async () => {
    consultantRatingId = '-1'
    consultantId = '701'
    year = '2023'
    month = '3'
    day = '3'
    hour = '21'
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.RATING_ID_IS_NOT_POSITIVE))
    postConsultantRatingFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(RateConsultantPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const rate = 1
    const rateSelect = wrapper.find('[data-test="rating-value"]').find('select')
    await rateSelect.setValue(rate)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('click')

    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(`${Message.RATING_ID_IS_NOT_POSITIVE_MESSAGE} (${Code.RATING_ID_IS_NOT_POSITIVE})`)
  })

  // UI上、不正な値の入力は許可していないが、仕様のためテストを用意しておく
  it(`displays ${Message.INVALID_RATING_MESSAGE} if ${Code.INVALID_RATING} is returned`, async () => {
    consultantRatingId = '21'
    consultantId = '701'
    year = '2023'
    month = '3'
    day = '3'
    hour = '21'
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_RATING))
    postConsultantRatingFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(RateConsultantPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const rate = 2
    const rateSelect = wrapper.find('[data-test="rating-value"]').find('select')
    await rateSelect.setValue(rate)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('click')

    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(`${Message.INVALID_RATING_MESSAGE} (${Code.INVALID_RATING})`)
  })

  it(`displays ${Message.END_OF_CONSULTATION_DATE_TIME_HAS_NOT_PASSED_YET_MESSAGE} if ${Code.END_OF_CONSULTATION_DATE_TIME_HAS_NOT_PASSED_YET} is returned`, async () => {
    consultantRatingId = '21'
    consultantId = '701'
    year = '2023'
    month = '3'
    day = '3'
    hour = '21'
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.END_OF_CONSULTATION_DATE_TIME_HAS_NOT_PASSED_YET))
    postConsultantRatingFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(RateConsultantPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const rate = 4
    const rateSelect = wrapper.find('[data-test="rating-value"]').find('select')
    await rateSelect.setValue(rate)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('click')

    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(`${Message.END_OF_CONSULTATION_DATE_TIME_HAS_NOT_PASSED_YET_MESSAGE} (${Code.END_OF_CONSULTATION_DATE_TIME_HAS_NOT_PASSED_YET})`)
  })

  it(`displays ${Message.ACCOUNT_IS_NOT_AVAILABLE_MESSAGE} if ${Code.ACCOUNT_IS_NOT_AVAILABLE} is returned`, async () => {
    consultantRatingId = '21'
    consultantId = '701'
    year = '2023'
    month = '3'
    day = '3'
    hour = '21'
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ACCOUNT_IS_NOT_AVAILABLE))
    postConsultantRatingFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(RateConsultantPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const rate = 4
    const rateSelect = wrapper.find('[data-test="rating-value"]').find('select')
    await rateSelect.setValue(rate)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('click')

    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(`${Message.ACCOUNT_IS_NOT_AVAILABLE_MESSAGE} (${Code.ACCOUNT_IS_NOT_AVAILABLE})`)
  })

  it(`displays ${Message.NO_CONSULTANT_RATING_FOUND_MESSAGE} if ${Code.NO_CONSULTANT_RATING_FOUND} is returned`, async () => {
    consultantRatingId = '21'
    consultantId = '701'
    year = '2023'
    month = '3'
    day = '3'
    hour = '21'
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_CONSULTANT_RATING_FOUND))
    postConsultantRatingFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(RateConsultantPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const rate = 4
    const rateSelect = wrapper.find('[data-test="rating-value"]').find('select')
    await rateSelect.setValue(rate)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('click')

    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(`${Message.NO_CONSULTANT_RATING_FOUND_MESSAGE} (${Code.NO_CONSULTANT_RATING_FOUND})`)
  })

  it(`displays ${Message.CONSULTANT_HAS_ALREADY_BEEN_RATED_MESSAGE} if ${Code.CONSULTANT_HAS_ALREADY_BEEN_RATED} is returned`, async () => {
    consultantRatingId = '21'
    consultantId = '701'
    year = '2023'
    month = '3'
    day = '3'
    hour = '21'
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.CONSULTANT_HAS_ALREADY_BEEN_RATED))
    postConsultantRatingFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(RateConsultantPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const rate = 4
    const rateSelect = wrapper.find('[data-test="rating-value"]').find('select')
    await rateSelect.setValue(rate)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('click')

    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(`${Message.CONSULTANT_HAS_ALREADY_BEEN_RATED_MESSAGE} (${Code.CONSULTANT_HAS_ALREADY_BEEN_RATED})`)
  })

  it(`displays ${Message.REACH_PAYMENT_PLATFORM_RATE_LIMIT_MESSAGE} if ${Code.REACH_PAYMENT_PLATFORM_RATE_LIMIT} is returned`, async () => {
    consultantRatingId = '21'
    consultantId = '701'
    year = '2023'
    month = '3'
    day = '3'
    hour = '21'
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.REACH_PAYMENT_PLATFORM_RATE_LIMIT))
    postConsultantRatingFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(RateConsultantPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const rate = 4
    const rateSelect = wrapper.find('[data-test="rating-value"]').find('select')
    await rateSelect.setValue(rate)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('click')

    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(`${Message.REACH_PAYMENT_PLATFORM_RATE_LIMIT_MESSAGE} (${Code.REACH_PAYMENT_PLATFORM_RATE_LIMIT})`)
  })
})
