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
import { UserSideAwaitingRating } from '@/util/personalized/awaiting-rating-list/UserSideAwaitingRating'
import { ConsultationDateTime } from '@/util/personalized/ConsultationDateTime'
import { ConsultantSideAwaitingRating } from '@/util/personalized/awaiting-rating-list/ConsultantSideAwaitingRating'

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

function createDummyUserSideAwaitingRating1 (): UserSideAwaitingRating {
  return {
    user_rating_id: 10,
    consultant_id: 53,
    meeting_date_time_in_jst: {
      year: 2023,
      month: 2,
      day: 26,
      hour: 7
    } as ConsultationDateTime
  } as UserSideAwaitingRating
}

function createDummyUserSideAwaitingRating2 (): UserSideAwaitingRating {
  return {
    user_rating_id: 11,
    consultant_id: 760,
    meeting_date_time_in_jst: {
      year: 2023,
      month: 2,
      day: 24,
      hour: 15
    } as ConsultationDateTime
  } as UserSideAwaitingRating
}

function createDummyConsultantSideAwaitingRating1 (): ConsultantSideAwaitingRating {
  return {
    consultant_rating_id: 234,
    user_account_id: 889,
    meeting_date_time_in_jst: {
      year: 2023,
      month: 2,
      day: 26,
      hour: 7
    } as ConsultationDateTime
  } as ConsultantSideAwaitingRating
}

function createDummyConsultantSideAwaitingRating2 (): ConsultantSideAwaitingRating {
  return {
    consultant_rating_id: 325,
    user_account_id: 1033,
    meeting_date_time_in_jst: {
      year: 2022,
      month: 12,
      day: 5,
      hour: 15
    } as ConsultationDateTime
  } as ConsultantSideAwaitingRating
}

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

  it('displays 1 user side awaiting rating and no consultant side awaiting ratings', async () => {
    const dummyUserSideAwaitingRating1 = createDummyUserSideAwaitingRating1()
    const resp = AwaitingRatingsResp.create({ user_side_awaiting_ratings: [dummyUserSideAwaitingRating1], consultant_side_awaiting_ratings: [] } as AwaitingRatings)
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

    const userSideAwaitingRating1 = wrapper.find(`[data-test="user-rating-id-${dummyUserSideAwaitingRating1.user_rating_id}"]`)
    const consultantIdLabel1 = userSideAwaitingRating1.find('[data-test="consultant-id-label"]')
    expect(consultantIdLabel1.text()).toContain(`コンサルタントID（${dummyUserSideAwaitingRating1.consultant_id}）`)
    const userSideConsultationDateTime1 = userSideAwaitingRating1.find('[data-test="user-side-consultation-date-time"]')
    expect(userSideConsultationDateTime1.text()).toContain(`相談日時：${dummyUserSideAwaitingRating1.meeting_date_time_in_jst.year}年${dummyUserSideAwaitingRating1.meeting_date_time_in_jst.month}月${dummyUserSideAwaitingRating1.meeting_date_time_in_jst.day}日${dummyUserSideAwaitingRating1.meeting_date_time_in_jst.hour}時`)
    const moveToRateConsultantPageBtn1 = userSideAwaitingRating1.find('[data-test="move-to-rate-consultant-page"]')
    expect(moveToRateConsultantPageBtn1.exists()).toBe(true)

    const consultantSideAwaitingRatingsLabel = wrapper.find('[data-test="consultant-side-awaiting-ratings-label"]')
    expect(consultantSideAwaitingRatingsLabel.text()).toContain('相談を受け付けたユーザー')
    const consultantSideAwaitingRatingsDescription = wrapper.find('[data-test="consultant-side-awaiting-ratings-description"]')
    expect(consultantSideAwaitingRatingsDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_CONSULTANT_SIDE_AWAITING_RATING}件分表示されます。${MAX_NUM_OF_CONSULTANT_SIDE_AWAITING_RATING}件を超えた分は表示されているユーザーの評価を終えると表示されます。`)
    const noConsultantSideAwaitingRatingsLabel = wrapper.find('[data-test="no-consultant-side-awaiting-ratings-label"]')
    expect(noConsultantSideAwaitingRatingsLabel.text()).toContain('未評価のユーザーはいません')
  })

  it('displays 2 user side awaiting ratings and no consultant side awaiting ratings', async () => {
    const dummyUserSideAwaitingRating1 = createDummyUserSideAwaitingRating1()
    const dummyUserSideAwaitingRating2 = createDummyUserSideAwaitingRating2()
    const resp = AwaitingRatingsResp.create({ user_side_awaiting_ratings: [dummyUserSideAwaitingRating1, dummyUserSideAwaitingRating2], consultant_side_awaiting_ratings: [] } as AwaitingRatings)
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

    const userSideAwaitingRating1 = wrapper.find(`[data-test="user-rating-id-${dummyUserSideAwaitingRating1.user_rating_id}"]`)
    const consultantIdLabel1 = userSideAwaitingRating1.find('[data-test="consultant-id-label"]')
    expect(consultantIdLabel1.text()).toContain(`コンサルタントID（${dummyUserSideAwaitingRating1.consultant_id}）`)
    const userSideConsultationDateTime1 = userSideAwaitingRating1.find('[data-test="user-side-consultation-date-time"]')
    expect(userSideConsultationDateTime1.text()).toContain(`相談日時：${dummyUserSideAwaitingRating1.meeting_date_time_in_jst.year}年${dummyUserSideAwaitingRating1.meeting_date_time_in_jst.month}月${dummyUserSideAwaitingRating1.meeting_date_time_in_jst.day}日${dummyUserSideAwaitingRating1.meeting_date_time_in_jst.hour}時`)
    const moveToRateConsultantPageBtn1 = userSideAwaitingRating1.find('[data-test="move-to-rate-consultant-page"]')
    expect(moveToRateConsultantPageBtn1.exists()).toBe(true)

    const userSideAwaitingRating2 = wrapper.find(`[data-test="user-rating-id-${dummyUserSideAwaitingRating2.user_rating_id}"]`)
    const consultantIdLabel2 = userSideAwaitingRating2.find('[data-test="consultant-id-label"]')
    expect(consultantIdLabel2.text()).toContain(`コンサルタントID（${dummyUserSideAwaitingRating2.consultant_id}）`)
    const userSideConsultationDateTime2 = userSideAwaitingRating2.find('[data-test="user-side-consultation-date-time"]')
    expect(userSideConsultationDateTime2.text()).toContain(`相談日時：${dummyUserSideAwaitingRating2.meeting_date_time_in_jst.year}年${dummyUserSideAwaitingRating2.meeting_date_time_in_jst.month}月${dummyUserSideAwaitingRating2.meeting_date_time_in_jst.day}日${dummyUserSideAwaitingRating2.meeting_date_time_in_jst.hour}時`)
    const moveToRateConsultantPageBtn2 = userSideAwaitingRating2.find('[data-test="move-to-rate-consultant-page"]')
    expect(moveToRateConsultantPageBtn2.exists()).toBe(true)

    const consultantSideAwaitingRatingsLabel = wrapper.find('[data-test="consultant-side-awaiting-ratings-label"]')
    expect(consultantSideAwaitingRatingsLabel.text()).toContain('相談を受け付けたユーザー')
    const consultantSideAwaitingRatingsDescription = wrapper.find('[data-test="consultant-side-awaiting-ratings-description"]')
    expect(consultantSideAwaitingRatingsDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_CONSULTANT_SIDE_AWAITING_RATING}件分表示されます。${MAX_NUM_OF_CONSULTANT_SIDE_AWAITING_RATING}件を超えた分は表示されているユーザーの評価を終えると表示されます。`)
    const noConsultantSideAwaitingRatingsLabel = wrapper.find('[data-test="no-consultant-side-awaiting-ratings-label"]')
    expect(noConsultantSideAwaitingRatingsLabel.text()).toContain('未評価のユーザーはいません')
  })

  it('displays no user side awaiting rating and 1 consultant side awaiting rating', async () => {
    const dummyConsultantSideAwaitingRating1 = createDummyConsultantSideAwaitingRating1()
    const resp = AwaitingRatingsResp.create({ user_side_awaiting_ratings: [], consultant_side_awaiting_ratings: [dummyConsultantSideAwaitingRating1] } as AwaitingRatings)
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

    const consultantSideAwaitingRating1 = wrapper.find(`[data-test="consultant-rating-id-${dummyConsultantSideAwaitingRating1.consultant_rating_id}"]`)
    const userAccountIdLabel1 = consultantSideAwaitingRating1.find('[data-test="user-account-id-label"]')
    expect(userAccountIdLabel1.text()).toContain(`ユーザーID（${dummyConsultantSideAwaitingRating1.user_account_id}）`)
    const consultantSideConsultationDateTime1 = consultantSideAwaitingRating1.find('[data-test="consultant-side-consultation-date-time"]')
    expect(consultantSideConsultationDateTime1.text()).toContain(`相談日時：${dummyConsultantSideAwaitingRating1.meeting_date_time_in_jst.year}年${dummyConsultantSideAwaitingRating1.meeting_date_time_in_jst.month}月${dummyConsultantSideAwaitingRating1.meeting_date_time_in_jst.day}日${dummyConsultantSideAwaitingRating1.meeting_date_time_in_jst.hour}時`)
    const moveToRateUserPageBtn1 = consultantSideAwaitingRating1.find('[data-test="move-to-rate-user-page"]')
    expect(moveToRateUserPageBtn1.exists()).toBe(true)
  })

  it('displays no user side awaiting ratings and 2 consultant side awaiting ratings', async () => {
    const dummyConsultantSideAwaitingRating1 = createDummyConsultantSideAwaitingRating1()
    const dummyConsultantSideAwaitingRating2 = createDummyConsultantSideAwaitingRating2()
    const resp = AwaitingRatingsResp.create({ user_side_awaiting_ratings: [], consultant_side_awaiting_ratings: [dummyConsultantSideAwaitingRating1, dummyConsultantSideAwaitingRating2] } as AwaitingRatings)
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

    const consultantSideAwaitingRating1 = wrapper.find(`[data-test="consultant-rating-id-${dummyConsultantSideAwaitingRating1.consultant_rating_id}"]`)
    const userAccountIdLabel1 = consultantSideAwaitingRating1.find('[data-test="user-account-id-label"]')
    expect(userAccountIdLabel1.text()).toContain(`ユーザーID（${dummyConsultantSideAwaitingRating1.user_account_id}）`)
    const consultantSideConsultationDateTime1 = consultantSideAwaitingRating1.find('[data-test="consultant-side-consultation-date-time"]')
    expect(consultantSideConsultationDateTime1.text()).toContain(`相談日時：${dummyConsultantSideAwaitingRating1.meeting_date_time_in_jst.year}年${dummyConsultantSideAwaitingRating1.meeting_date_time_in_jst.month}月${dummyConsultantSideAwaitingRating1.meeting_date_time_in_jst.day}日${dummyConsultantSideAwaitingRating1.meeting_date_time_in_jst.hour}時`)
    const moveToRateUserPageBtn1 = consultantSideAwaitingRating1.find('[data-test="move-to-rate-user-page"]')
    expect(moveToRateUserPageBtn1.exists()).toBe(true)

    const consultantSideAwaitingRating2 = wrapper.find(`[data-test="consultant-rating-id-${dummyConsultantSideAwaitingRating2.consultant_rating_id}"]`)
    const userAccountIdLabel2 = consultantSideAwaitingRating2.find('[data-test="user-account-id-label"]')
    expect(userAccountIdLabel2.text()).toContain(`ユーザーID（${dummyConsultantSideAwaitingRating2.user_account_id}）`)
    const consultantSideConsultationDateTime2 = consultantSideAwaitingRating2.find('[data-test="consultant-side-consultation-date-time"]')
    expect(consultantSideConsultationDateTime2.text()).toContain(`相談日時：${dummyConsultantSideAwaitingRating2.meeting_date_time_in_jst.year}年${dummyConsultantSideAwaitingRating2.meeting_date_time_in_jst.month}月${dummyConsultantSideAwaitingRating2.meeting_date_time_in_jst.day}日${dummyConsultantSideAwaitingRating2.meeting_date_time_in_jst.hour}時`)
    const moveToRateUserPageBtn2 = consultantSideAwaitingRating2.find('[data-test="move-to-rate-user-page"]')
    expect(moveToRateUserPageBtn2.exists()).toBe(true)
  })

  it('displays 1 user side awaiting rating and 1 consultant side awaiting rating', async () => {
    const dummyUserSideAwaitingRating1 = createDummyUserSideAwaitingRating1()
    const dummyConsultantSideAwaitingRating1 = createDummyConsultantSideAwaitingRating1()
    const resp = AwaitingRatingsResp.create({ user_side_awaiting_ratings: [dummyUserSideAwaitingRating1], consultant_side_awaiting_ratings: [dummyConsultantSideAwaitingRating1] } as AwaitingRatings)
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

    const userSideAwaitingRating1 = wrapper.find(`[data-test="user-rating-id-${dummyUserSideAwaitingRating1.user_rating_id}"]`)
    const consultantIdLabel1 = userSideAwaitingRating1.find('[data-test="consultant-id-label"]')
    expect(consultantIdLabel1.text()).toContain(`コンサルタントID（${dummyUserSideAwaitingRating1.consultant_id}）`)
    const userSideConsultationDateTime1 = userSideAwaitingRating1.find('[data-test="user-side-consultation-date-time"]')
    expect(userSideConsultationDateTime1.text()).toContain(`相談日時：${dummyUserSideAwaitingRating1.meeting_date_time_in_jst.year}年${dummyUserSideAwaitingRating1.meeting_date_time_in_jst.month}月${dummyUserSideAwaitingRating1.meeting_date_time_in_jst.day}日${dummyUserSideAwaitingRating1.meeting_date_time_in_jst.hour}時`)
    const moveToRateConsultantPageBtn1 = userSideAwaitingRating1.find('[data-test="move-to-rate-consultant-page"]')
    expect(moveToRateConsultantPageBtn1.exists()).toBe(true)

    const consultantSideAwaitingRatingsLabel = wrapper.find('[data-test="consultant-side-awaiting-ratings-label"]')
    expect(consultantSideAwaitingRatingsLabel.text()).toContain('相談を受け付けたユーザー')
    const consultantSideAwaitingRatingsDescription = wrapper.find('[data-test="consultant-side-awaiting-ratings-description"]')
    expect(consultantSideAwaitingRatingsDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_CONSULTANT_SIDE_AWAITING_RATING}件分表示されます。${MAX_NUM_OF_CONSULTANT_SIDE_AWAITING_RATING}件を超えた分は表示されているユーザーの評価を終えると表示されます。`)

    const consultantSideAwaitingRating1 = wrapper.find(`[data-test="consultant-rating-id-${dummyConsultantSideAwaitingRating1.consultant_rating_id}"]`)
    const userAccountIdLabel1 = consultantSideAwaitingRating1.find('[data-test="user-account-id-label"]')
    expect(userAccountIdLabel1.text()).toContain(`ユーザーID（${dummyConsultantSideAwaitingRating1.user_account_id}）`)
    const consultantSideConsultationDateTime1 = consultantSideAwaitingRating1.find('[data-test="consultant-side-consultation-date-time"]')
    expect(consultantSideConsultationDateTime1.text()).toContain(`相談日時：${dummyConsultantSideAwaitingRating1.meeting_date_time_in_jst.year}年${dummyConsultantSideAwaitingRating1.meeting_date_time_in_jst.month}月${dummyConsultantSideAwaitingRating1.meeting_date_time_in_jst.day}日${dummyConsultantSideAwaitingRating1.meeting_date_time_in_jst.hour}時`)
    const moveToRateUserPageBtn1 = consultantSideAwaitingRating1.find('[data-test="move-to-rate-user-page"]')
    expect(moveToRateUserPageBtn1.exists()).toBe(true)
  })

  it('displays 2 user side awaiting ratings and 2 consultant side awaiting ratings', async () => {
    const dummyUserSideAwaitingRating1 = createDummyUserSideAwaitingRating1()
    const dummyUserSideAwaitingRating2 = createDummyUserSideAwaitingRating2()
    const dummyConsultantSideAwaitingRating1 = createDummyConsultantSideAwaitingRating1()
    const dummyConsultantSideAwaitingRating2 = createDummyConsultantSideAwaitingRating2()
    const resp = AwaitingRatingsResp.create({ user_side_awaiting_ratings: [dummyUserSideAwaitingRating1, dummyUserSideAwaitingRating2], consultant_side_awaiting_ratings: [dummyConsultantSideAwaitingRating1, dummyConsultantSideAwaitingRating2] } as AwaitingRatings)
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

    const userSideAwaitingRating1 = wrapper.find(`[data-test="user-rating-id-${dummyUserSideAwaitingRating1.user_rating_id}"]`)
    const consultantIdLabel1 = userSideAwaitingRating1.find('[data-test="consultant-id-label"]')
    expect(consultantIdLabel1.text()).toContain(`コンサルタントID（${dummyUserSideAwaitingRating1.consultant_id}）`)
    const userSideConsultationDateTime1 = userSideAwaitingRating1.find('[data-test="user-side-consultation-date-time"]')
    expect(userSideConsultationDateTime1.text()).toContain(`相談日時：${dummyUserSideAwaitingRating1.meeting_date_time_in_jst.year}年${dummyUserSideAwaitingRating1.meeting_date_time_in_jst.month}月${dummyUserSideAwaitingRating1.meeting_date_time_in_jst.day}日${dummyUserSideAwaitingRating1.meeting_date_time_in_jst.hour}時`)
    const moveToRateConsultantPageBtn1 = userSideAwaitingRating1.find('[data-test="move-to-rate-consultant-page"]')
    expect(moveToRateConsultantPageBtn1.exists()).toBe(true)

    const userSideAwaitingRating2 = wrapper.find(`[data-test="user-rating-id-${dummyUserSideAwaitingRating2.user_rating_id}"]`)
    const consultantIdLabel2 = userSideAwaitingRating2.find('[data-test="consultant-id-label"]')
    expect(consultantIdLabel2.text()).toContain(`コンサルタントID（${dummyUserSideAwaitingRating2.consultant_id}）`)
    const userSideConsultationDateTime2 = userSideAwaitingRating2.find('[data-test="user-side-consultation-date-time"]')
    expect(userSideConsultationDateTime2.text()).toContain(`相談日時：${dummyUserSideAwaitingRating2.meeting_date_time_in_jst.year}年${dummyUserSideAwaitingRating2.meeting_date_time_in_jst.month}月${dummyUserSideAwaitingRating2.meeting_date_time_in_jst.day}日${dummyUserSideAwaitingRating2.meeting_date_time_in_jst.hour}時`)
    const moveToRateConsultantPageBtn2 = userSideAwaitingRating2.find('[data-test="move-to-rate-consultant-page"]')
    expect(moveToRateConsultantPageBtn2.exists()).toBe(true)

    const consultantSideAwaitingRatingsLabel = wrapper.find('[data-test="consultant-side-awaiting-ratings-label"]')
    expect(consultantSideAwaitingRatingsLabel.text()).toContain('相談を受け付けたユーザー')
    const consultantSideAwaitingRatingsDescription = wrapper.find('[data-test="consultant-side-awaiting-ratings-description"]')
    expect(consultantSideAwaitingRatingsDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_CONSULTANT_SIDE_AWAITING_RATING}件分表示されます。${MAX_NUM_OF_CONSULTANT_SIDE_AWAITING_RATING}件を超えた分は表示されているユーザーの評価を終えると表示されます。`)

    const consultantSideAwaitingRating1 = wrapper.find(`[data-test="consultant-rating-id-${dummyConsultantSideAwaitingRating1.consultant_rating_id}"]`)
    const userAccountIdLabel1 = consultantSideAwaitingRating1.find('[data-test="user-account-id-label"]')
    expect(userAccountIdLabel1.text()).toContain(`ユーザーID（${dummyConsultantSideAwaitingRating1.user_account_id}）`)
    const consultantSideConsultationDateTime1 = consultantSideAwaitingRating1.find('[data-test="consultant-side-consultation-date-time"]')
    expect(consultantSideConsultationDateTime1.text()).toContain(`相談日時：${dummyConsultantSideAwaitingRating1.meeting_date_time_in_jst.year}年${dummyConsultantSideAwaitingRating1.meeting_date_time_in_jst.month}月${dummyConsultantSideAwaitingRating1.meeting_date_time_in_jst.day}日${dummyConsultantSideAwaitingRating1.meeting_date_time_in_jst.hour}時`)
    const moveToRateUserPageBtn1 = consultantSideAwaitingRating1.find('[data-test="move-to-rate-user-page"]')
    expect(moveToRateUserPageBtn1.exists()).toBe(true)

    const consultantSideAwaitingRating2 = wrapper.find(`[data-test="consultant-rating-id-${dummyConsultantSideAwaitingRating2.consultant_rating_id}"]`)
    const userAccountIdLabel2 = consultantSideAwaitingRating2.find('[data-test="user-account-id-label"]')
    expect(userAccountIdLabel2.text()).toContain(`ユーザーID（${dummyConsultantSideAwaitingRating2.user_account_id}）`)
    const consultantSideConsultationDateTime2 = consultantSideAwaitingRating2.find('[data-test="consultant-side-consultation-date-time"]')
    expect(consultantSideConsultationDateTime2.text()).toContain(`相談日時：${dummyConsultantSideAwaitingRating2.meeting_date_time_in_jst.year}年${dummyConsultantSideAwaitingRating2.meeting_date_time_in_jst.month}月${dummyConsultantSideAwaitingRating2.meeting_date_time_in_jst.day}日${dummyConsultantSideAwaitingRating2.meeting_date_time_in_jst.hour}時`)
    const moveToRateUserPageBtn2 = consultantSideAwaitingRating2.find('[data-test="move-to-rate-user-page"]')
    expect(moveToRateUserPageBtn2.exists()).toBe(true)
  })
})
