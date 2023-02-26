import WaitingCircle from '@/components/WaitingCircle.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import TheHeader from '@/components/TheHeader.vue'
import { RouterLinkStub, mount, flushPromises } from '@vue/test-utils'
import AwaitingRatingListPage from '@/views/personalized/AwaitingRatingListPage.vue'
import { Message } from '@/util/Message'
import { ref } from 'vue'
import { AwaitingRatingsResp } from '@/util/personalized/awaiting-rating-list/AwaitingRatingsResp'
import { AwaitingRatings } from '@/util/personalized/awaiting-rating-list/AwaitingRatings'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { MAX_NUM_OF_CONSULTANT_SIDE_AWAITING_RATING, MAX_NUM_OF_USER_SIDE_AWAITING_RATING } from '@/util/personalized/awaiting-rating-list/MaxNumOfAwaitingRating'

const getAwaitingRatingsDoneMock = ref(true)
const getAwaitingRatingsFuncMock = jest.fn()
jest.mock('@/util/personalized/awaiting-rating-list/useGetAwaitingRatings', () => ({
  useGetAwaitingRatings: () => ({
    getAwaitingRatingsDone: getAwaitingRatingsDoneMock,
    getAwaitingRatingsFunc: getAwaitingRatingsFuncMock
  })
}))

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

describe('AwaitingRatingListPage.vue', () => {
  beforeEach(() => {
    getAwaitingRatingsDoneMock.value = true
    getAwaitingRatingsFuncMock.mockReset()
    routerPushMock.mockClear()
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    getAwaitingRatingsDoneMock.value = false
    const resp = AwaitingRatingsResp.create({ user_side_awaiting_ratings: [], consultant_side_awaiting_ratings: [] } as AwaitingRatings)
    getAwaitingRatingsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(AwaitingRatingListPage, {
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

  it('displays AlertMessage when error has happened', async () => {
    const errDetail = 'connection error'
    getAwaitingRatingsFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(AwaitingRatingListPage, {
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

  it(`moves to login if request returns ${Code.UNAUTHORIZED}`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    getAwaitingRatingsFuncMock.mockResolvedValue(apiErrResp)
    mount(AwaitingRatingListPage, {
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

  it(`moves to login if request returns ${Code.NOT_TERMS_OF_USE_AGREED_YET}`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    getAwaitingRatingsFuncMock.mockResolvedValue(apiErrResp)
    mount(AwaitingRatingListPage, {
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

  it('displays no user side awaiting ratings and consultant side awaiting ratings when both do not exist', async () => {
    const resp = AwaitingRatingsResp.create({ user_side_awaiting_ratings: [], consultant_side_awaiting_ratings: [] } as AwaitingRatings)
    getAwaitingRatingsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(AwaitingRatingListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const userSideAwaitingRatingsLabel = wrapper.find('[data-test="user-side-awaiting-ratings-label"]')
    expect(userSideAwaitingRatingsLabel.text()).toContain('相談を行ったコンサルタント')
    const userSideAwaitingRatingsDescription = wrapper.find('[data-test="user-side-awaiting-ratings-description"]')
    expect(userSideAwaitingRatingsDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_USER_SIDE_AWAITING_RATING}件分表示されます。${MAX_NUM_OF_USER_SIDE_AWAITING_RATING}件を超えた分は表示されているコンサルタントの評価を終えると表示されます。`)
    const noUserSideAwaitingRatingsLabel = wrapper.find('[data-test="no-user-side-awaiting-ratings-label"]')
    expect(noUserSideAwaitingRatingsLabel.text()).toContain('未評価のコンサルタントはいません')

    const consultantSideAwaitingRatingsLabel = wrapper.find('[data-test="consultant-side-awaiting-ratings-label"]')
    expect(consultantSideAwaitingRatingsLabel.text()).toContain('相談を受け付けたユーザー')
    const consultantSideAwaitingRatingsDescription = wrapper.find('[data-test="consultant-side-awaiting-ratings-description"]')
    expect(consultantSideAwaitingRatingsDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_CONSULTANT_SIDE_AWAITING_RATING}件分表示されます。${MAX_NUM_OF_CONSULTANT_SIDE_AWAITING_RATING}件を超えた分は表示されているユーザーの評価を終えると表示されます。`)
    const noConsultantSideAwaitingRatingsLabel = wrapper.find('[data-test="no-consultant-side-awaiting-ratings-label"]')
    expect(noConsultantSideAwaitingRatingsLabel.text()).toContain('未評価のユーザーはいません')
  })
})
