import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { nextTick, ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import CreateCareerRequestRejectionDetailPage from '@/views/personalized/CreateCareerRequestRejectionDetailPage.vue'
import { PostCreateCareerRequestRejectionResp } from '@/util/personalized/create-career-request-rejection-detail/PostCreateCareerRequestRejectionResp'
import AlertMessage from '@/components/AlertMessage.vue'
import { createReasonList } from '@/util/personalized/create-career-request-rejection-detail/ReasonList'
import { Code } from '@/util/Error'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Message } from '@/util/Message'

const routerPushMock = jest.fn()
let routeParam = ''
jest.mock('vue-router', () => ({
  useRoute: () => ({
    params: {
      create_career_req_id: routeParam
    }
  }),
  useRouter: () => ({
    push: routerPushMock
  })
}))

const waitingRequestDoneMock = ref(false)
const postCreateCareerRequestRejectionFuncMock = jest.fn()
jest.mock('@/util/personalized/create-career-request-rejection-detail/usePostCreateCareerRequestRejection', () => ({
  usePostCreateCareerRequestRejection: () => ({
    waitingRequestDone: waitingRequestDoneMock,
    postCreateCareerRequestRejectionFunc: postCreateCareerRequestRejectionFuncMock
  })
}))

describe('CreateCareerRequestRejectionDetailPage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    routeParam = ''
    waitingRequestDoneMock.value = false
    postCreateCareerRequestRejectionFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    routeParam = '1'
    const resp = PostCreateCareerRequestRejectionResp.create()
    postCreateCareerRequestRejectionFuncMock.mockResolvedValue(resp)
    waitingRequestDoneMock.value = true
    const wrapper = mount(CreateCareerRequestRejectionDetailPage, {
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

  it('has AlertMessage with a hidden attribute when created', () => {
    const wrapper = mount(CreateCareerRequestRejectionDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).toContain('hidden')
  })

  it('has title, description and label for selection', () => {
    const wrapper = mount(CreateCareerRequestRejectionDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const title = wrapper.find('[data-test="title"]')
    expect(title.text()).toContain('職務経歴確認依頼拒否理由')
    const description = wrapper.find('[data-test="description"]')
    expect(description.text()).toContain('拒否理由を選択して依頼を拒否して下さい。適切な拒否理由がない場合、管理者にご連絡下さい。')
    const label = wrapper.find('[data-test="label"]')
    expect(label.text()).toContain('拒否理由')
  })

  it('moves to CreateCareerRequestRejectionPage if request is successful', async () => {
    routeParam = '1'
    const resp = PostCreateCareerRequestRejectionResp.create()
    postCreateCareerRequestRejectionFuncMock.mockResolvedValue(resp)
    const wrapper = mount(CreateCareerRequestRejectionDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')

    expect(postCreateCareerRequestRejectionFuncMock).toHaveBeenCalledTimes(1)
    const list = createReasonList()
    expect(postCreateCareerRequestRejectionFuncMock).toHaveBeenCalledWith(parseInt(routeParam), list[0])
    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/create-career-request-rejection')
  })

  it(`moves to login if ${Code.UNAUTHORIZED} is returned`, async () => {
    routeParam = '1'
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    postCreateCareerRequestRejectionFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(CreateCareerRequestRejectionDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')

    expect(postCreateCareerRequestRejectionFuncMock).toHaveBeenCalledTimes(1)
    const list = createReasonList()
    expect(postCreateCareerRequestRejectionFuncMock).toHaveBeenCalledWith(parseInt(routeParam), list[0])
    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/login')
  })

  it('displays AlertMessage when error has happened', async () => {
    routeParam = '1'
    const errDetail = 'connection error'
    postCreateCareerRequestRejectionFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(CreateCareerRequestRejectionDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await nextTick()

    expect(postCreateCareerRequestRejectionFuncMock).toHaveBeenCalledTimes(1)
    const list = createReasonList()
    expect(postCreateCareerRequestRejectionFuncMock).toHaveBeenCalledWith(parseInt(routeParam), list[0])
    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.UNEXPECTED_ERR)
    expect(resultMessage).toContain(errDetail)
  })

  it(`displays ${Message.INVALID_FORMAT_REASON_MESSAGE} if ${Code.INVALID_FORMAT_REASON} is returned`, async () => {
    routeParam = '1'
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_FORMAT_REASON))
    postCreateCareerRequestRejectionFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(CreateCareerRequestRejectionDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await nextTick()

    expect(postCreateCareerRequestRejectionFuncMock).toHaveBeenCalledTimes(1)
    const list = createReasonList()
    expect(postCreateCareerRequestRejectionFuncMock).toHaveBeenCalledWith(parseInt(routeParam), list[0])
    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(`${Message.INVALID_FORMAT_REASON_MESSAGE} (${Code.INVALID_FORMAT_REASON})`)
  })
})
