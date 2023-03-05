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

const postUserRatingDoneMock = ref(true)
const postUserRatingFuncMock = jest.fn()
jest.mock('@/util/personalized/rate-user/usePostUserRating', () => ({
  usePostUserRating: () => ({
    postUserRatingDone: postUserRatingDoneMock,
    postUserRatingFunc: postUserRatingFuncMock
  })
}))

let routeParam = ''
let userId = ''
let year = ''
let month = ''
let day = ''
let hour = ''
const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRoute: () => ({
    params: {
      user_rating_id: routeParam
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
    routeParam = ''
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
