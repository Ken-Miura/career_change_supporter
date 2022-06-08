import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import CareerDeletionConfirmPage from '@/views/personalized/CareerDeletionConfirmPage.vue'
import TheHeader from '@/components/TheHeader.vue'
import { DeleteCareerResp } from '@/util/personalized/career-deletion-confirm/DeleteCareerResp'
import { refresh } from '@/util/personalized/refresh/Refresh'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { Message } from '@/util/Message'
import { Code } from '@/util/Error'
import { ApiError, ApiErrorResp } from '@/util/ApiError'

jest.mock('@/util/personalized/refresh/Refresh')
const refreshMock = refresh as jest.MockedFunction<typeof refresh>

const deleteCareerDoneMock = ref(true)
const deleteCareerFuncMock = jest.fn()
jest.mock('@/util/personalized/career-deletion-confirm/useDeleteCareer', () => ({
  useDeleteCareer: () => ({
    deleteCareerDone: deleteCareerDoneMock,
    deleteCareerFunc: deleteCareerFuncMock
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

describe('CareerDeletionConfirmPage.vue', () => {
  beforeEach(() => {
    routeParam = '1'
    refreshMock.mockReset()
    deleteCareerDoneMock.value = true
    deleteCareerFuncMock.mockReset()
    routerPushMock.mockClear()
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    deleteCareerDoneMock.value = false
    const resp = DeleteCareerResp.create()
    deleteCareerFuncMock.mockResolvedValue(resp)
    const wrapper = mount(CareerDeletionConfirmPage, {
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

  it('does not display AlertMessage when error does not occur', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const resp = DeleteCareerResp.create()
    deleteCareerFuncMock.mockResolvedValue(resp)
    const wrapper = mount(CareerDeletionConfirmPage, {
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
    refreshMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(CareerDeletionConfirmPage, {
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

  it(`moves to login if ${Code.UNAUTHORIZED} is returned on opening page`, async () => {
    const resp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    refreshMock.mockResolvedValue(resp)
    deleteCareerFuncMock.mockResolvedValue(DeleteCareerResp.create())
    mount(CareerDeletionConfirmPage, {
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
    refreshMock.mockResolvedValue(resp)
    deleteCareerFuncMock.mockResolvedValue(DeleteCareerResp.create())
    mount(CareerDeletionConfirmPage, {
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

  it(`moves to login if ${Code.UNAUTHORIZED} is returned on clicking button`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const resp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    deleteCareerFuncMock.mockResolvedValue(resp)
    const wrapper = mount(CareerDeletionConfirmPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const button = wrapper.find('[data-test="delete-career-button"]')
    await button.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/login')
  })

  it(`moves to terms-of-use if ${Code.NOT_TERMS_OF_USE_AGREED_YET} is returned on clicking button`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const resp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    deleteCareerFuncMock.mockResolvedValue(resp)
    const wrapper = mount(CareerDeletionConfirmPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const button = wrapper.find('[data-test="delete-career-button"]')
    await button.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/terms-of-use')
  })

  it('moves to delete-career-success on clicking button', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    deleteCareerFuncMock.mockResolvedValue(DeleteCareerResp.create())
    const wrapper = mount(CareerDeletionConfirmPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const button = wrapper.find('[data-test="delete-career-button"]')
    await button.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/delete-career-success')
  })

  it(`displays ${Message.NO_CAREER_TO_HANDLE_FOUND_MESSAGE} if ${Code.NO_CAREER_TO_HANDLE_FOUND} is returned on clicking button`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const resp = ApiErrorResp.create(400, ApiError.create(Code.NO_CAREER_TO_HANDLE_FOUND))
    deleteCareerFuncMock.mockResolvedValue(resp)
    const wrapper = mount(CareerDeletionConfirmPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const button = wrapper.find('[data-test="delete-career-button"]')
    await button.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.NO_CAREER_TO_HANDLE_FOUND_MESSAGE)
    expect(resultMessage).toContain(Code.NO_CAREER_TO_HANDLE_FOUND.toString())
  })

  it('displays AlertMessage when error has happened on clicking page', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const errDetail = 'connection error'
    deleteCareerFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(CareerDeletionConfirmPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const button = wrapper.find('[data-test="delete-career-button"]')
    await button.trigger('click')
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
