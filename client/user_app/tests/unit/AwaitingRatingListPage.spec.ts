import WaitingCircle from '@/components/WaitingCircle.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import TheHeader from '@/components/TheHeader.vue'
import { RouterLinkStub, mount, flushPromises } from '@vue/test-utils'
import AwaitingRatingListPage from '@/views/personalized/AwaitingRatingListPage.vue'
import { Message } from '@/util/Message'
import { ref } from 'vue'
import { AwaitingRatingsResp } from '@/util/personalized/awaiting-rating-list/AwaitingRatingsResp'
import { AwaitingRatings } from '@/util/personalized/awaiting-rating-list/AwaitingRatings'

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
})
