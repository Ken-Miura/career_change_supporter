import WaitingCircle from '@/components/WaitingCircle.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import TheHeader from '@/components/TheHeader.vue'
import { RouterLinkStub, mount, flushPromises } from '@vue/test-utils'
import RateUserPage from '@/views/personalized/RateUserPage.vue'
import { Message } from '@/util/Message'
import { ref } from 'vue'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { PostUserRatingResp } from '@/util/personalized/rate-user/PostUserRatingResp'
import { MAX_RATING, MIN_RATING } from '@/util/personalized/RatingConstants'

const postUserRatingDoneMock = ref(true)
const postUserRatingFuncMock = jest.fn()
jest.mock('@/util/personalized/rate-user/usePostUserRating', () => ({
  usePostUserRating: () => ({
    postUserRatingDone: postUserRatingDoneMock,
    postUserRatingFunc: postUserRatingFuncMock
  })
}))

let userRatingId = ''
let userId = ''
let year = ''
let month = ''
let day = ''
let hour = ''
const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRoute: () => ({
    params: {
      user_rating_id: userRatingId
    },
    query: {
      'user-id': userId,
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

describe('RateUserPage.vue', () => {
  beforeEach(() => {
    postUserRatingDoneMock.value = true
    postUserRatingFuncMock.mockReset()
    routerPushMock.mockClear()
    userRatingId = ''
    userId = ''
    year = ''
    month = ''
    day = ''
    hour = ''
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    postUserRatingDoneMock.value = false
    const resp = PostUserRatingResp.create()
    postUserRatingFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RateUserPage, {
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
    userRatingId = '511'
    userId = '701'
    year = '2023'
    month = '3'
    day = '3'
    hour = '21'
    const wrapper = mount(RateUserPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const userRatingLabel = wrapper.find('[data-test="user-rating-label"]')
    expect(userRatingLabel.text()).toContain('相談を受け付けたユーザーの評価')
    const userRatingDescription = wrapper.find('[data-test="user-rating-description"]')
    expect(userRatingDescription.text()).toContain(`相談を行ったユーザーを評価して下さい。${MIN_RATING}が最も低い（悪い）評価で、${MAX_RATING}が最も高い（良い）評価となります。`)

    const userIdLabel = wrapper.find('[data-test="user-id-label"]')
    expect(userIdLabel.text()).toContain('ユーザーID')
    const userIdValue = wrapper.find('[data-test="user-id-value"]')
    expect(userIdValue.text()).toContain(`${userId}`)

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
    userRatingId = '511'
    userId = '701'
    year = '2023'
    month = '3'
    day = '3'
    hour = '21'
    const wrapper = mount(RateUserPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const userRatingLabel = wrapper.find('[data-test="user-rating-label"]')
    expect(userRatingLabel.text()).toContain('相談を受け付けたユーザーの評価')
    const userRatingDescription = wrapper.find('[data-test="user-rating-description"]')
    expect(userRatingDescription.text()).toContain(`相談を行ったユーザーを評価して下さい。${MIN_RATING}が最も低い（悪い）評価で、${MAX_RATING}が最も高い（良い）評価となります。`)

    const userIdLabel = wrapper.find('[data-test="user-id-label"]')
    expect(userIdLabel.text()).toContain('ユーザーID')
    const userIdValue = wrapper.find('[data-test="user-id-value"]')
    expect(userIdValue.text()).toContain(`${userId}`)

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
    userRatingId = '511'
    userId = '701'
    year = '2023'
    month = '3'
    day = '3'
    hour = '21'
    const resp = PostUserRatingResp.create()
    postUserRatingFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RateUserPage, {
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

  // it('displays AlertMessage when error has happened', async () => {
  //   const errDetail = 'connection error'
  //   postUserRatingFuncMock.mockRejectedValue(new Error(errDetail))
  //   const wrapper = mount(RateUserPage, {
  //     global: {
  //       stubs: {
  //         RouterLink: RouterLinkStub
  //       }
  //     }
  //   })
  //   await flushPromises()

  //   const alertMessages = wrapper.findAllComponents(AlertMessage)
  //   expect(alertMessages.length).toBe(1)
  //   const alertMessage = alertMessages[0]
  //   expect(alertMessage).not.toContain('hidden')
  //   const resultMessage = alertMessage.text()
  //   expect(resultMessage).toContain(Message.UNEXPECTED_ERR)
  //   expect(resultMessage).toContain(errDetail)
  // })

  // it(`moves to login if request returns ${Code.UNAUTHORIZED}`, async () => {
  //   const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
  //   postUserRatingFuncMock.mockResolvedValue(apiErrResp)
  //   mount(RateUserPage, {
  //     global: {
  //       stubs: {
  //         RouterLink: RouterLinkStub
  //       }
  //     }
  //   })
  //   await flushPromises()

  //   expect(routerPushMock).toHaveBeenCalledTimes(1)
  //   expect(routerPushMock).toHaveBeenCalledWith('/login')
  // })

  // it(`moves to login if request returns ${Code.NOT_TERMS_OF_USE_AGREED_YET}`, async () => {
  //   const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
  //   postUserRatingFuncMock.mockResolvedValue(apiErrResp)
  //   mount(RateUserPage, {
  //     global: {
  //       stubs: {
  //         RouterLink: RouterLinkStub
  //       }
  //     }
  //   })
  //   await flushPromises()

  //   expect(routerPushMock).toHaveBeenCalledTimes(1)
  //   expect(routerPushMock).toHaveBeenCalledWith('/terms-of-use')
  // })
})
