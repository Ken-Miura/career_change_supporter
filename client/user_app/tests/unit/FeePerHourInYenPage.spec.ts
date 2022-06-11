import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import FeePerHourInYenPage from '@/views/personalized/FeePerHourInYenPage.vue'
import TheHeader from '@/components/TheHeader.vue'
import { refresh } from '@/util/personalized/refresh/Refresh'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { Message } from '@/util/Message'
import { PostFeePerHourInYenResp } from '@/util/personalized/fee-per-hour-in-yen/PostFeePerHourInYenResp'
import { Code } from '@/util/Error'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { MAX_FEE_PER_HOUR_IN_YEN, MIN_FEE_PER_HOUR_IN_YEN } from '@/util/Fee'

jest.mock('@/util/personalized/refresh/Refresh')
const refreshMock = refresh as jest.MockedFunction<typeof refresh>

const postFeePerHourInYenDoneMock = ref(true)
const postFeePerHourInYenFuncMock = jest.fn()
jest.mock('@/util/personalized/fee-per-hour-in-yen/usePostFeePerHourInYen', () => ({
  usePostFeePerHourInYen: () => ({
    postFeePerHourInYenDone: postFeePerHourInYenDoneMock,
    postFeePerHourInYenFunc: postFeePerHourInYenFuncMock
  })
}))

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

let feePerHourInYenock = null as number | null
const storeCommitMock = jest.fn()
jest.mock('vuex', () => ({
  useStore: () => ({
    commit: storeCommitMock,
    state: {
      feePerHourInYen: feePerHourInYenock
    }
  })
}))

describe('FeePerHourInYenPage.vue', () => {
  beforeEach(() => {
    feePerHourInYenock = null
    refreshMock.mockReset()
    postFeePerHourInYenDoneMock.value = true
    postFeePerHourInYenFuncMock.mockReset()
    routerPushMock.mockClear()
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    postFeePerHourInYenDoneMock.value = false
    const resp = PostFeePerHourInYenResp.create()
    postFeePerHourInYenFuncMock.mockResolvedValue(resp)
    const wrapper = mount(FeePerHourInYenPage, {
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

  it('displays AlertMessage when error has happened on refresh', async () => {
    const errDetail = 'connection error'
    refreshMock.mockRejectedValue(new Error(errDetail))
    const resp = PostFeePerHourInYenResp.create()
    postFeePerHourInYenFuncMock.mockResolvedValue(resp)
    const wrapper = mount(FeePerHourInYenPage, {
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

  it(`moves to login if refresh returns ${Code.UNAUTHORIZED}`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    refreshMock.mockResolvedValue(apiErrResp)
    const resp = PostFeePerHourInYenResp.create()
    postFeePerHourInYenFuncMock.mockResolvedValue(resp)
    mount(FeePerHourInYenPage, {
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

  it(`moves to terms-of-use if refresh returns ${Code.NOT_TERMS_OF_USE_AGREED_YET}`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    refreshMock.mockResolvedValue(apiErrResp)
    const resp = PostFeePerHourInYenResp.create()
    postFeePerHourInYenFuncMock.mockResolvedValue(resp)
    mount(FeePerHourInYenPage, {
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

  it('displays initial value and moves submit-fee-per-hour-in-yen-success if button is clicked', async () => {
    feePerHourInYenock = 5000
    refreshMock.mockResolvedValue(RefreshResp.create())
    const resp = PostFeePerHourInYenResp.create()
    postFeePerHourInYenFuncMock.mockResolvedValue(resp)
    const wrapper = mount(FeePerHourInYenPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const feeInputDiv = wrapper.find('[data-test="fee-input-div"]')
    const feeInput = feeInputDiv.find('input')
    expect(parseInt(feeInput.element.value)).toEqual(feePerHourInYenock)
    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/submit-fee-per-hour-in-yen-success')
  })

  it(`moves to submit-fee-per-hour-in-yen-success if button is clicked (${MIN_FEE_PER_HOUR_IN_YEN})`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const resp = PostFeePerHourInYenResp.create()
    postFeePerHourInYenFuncMock.mockResolvedValue(resp)
    const wrapper = mount(FeePerHourInYenPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const feeInputDiv = wrapper.find('[data-test="fee-input-div"]')
    const feeInput = feeInputDiv.find('input')
    await feeInput.setValue(MIN_FEE_PER_HOUR_IN_YEN.toString())
    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/submit-fee-per-hour-in-yen-success')
  })

  it(`moves to submit-fee-per-hour-in-yen-success if button is clicked (${MAX_FEE_PER_HOUR_IN_YEN})`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const resp = PostFeePerHourInYenResp.create()
    postFeePerHourInYenFuncMock.mockResolvedValue(resp)
    const wrapper = mount(FeePerHourInYenPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const feeInputDiv = wrapper.find('[data-test="fee-input-div"]')
    const feeInput = feeInputDiv.find('input')
    await feeInput.setValue(MAX_FEE_PER_HOUR_IN_YEN.toString())
    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/submit-fee-per-hour-in-yen-success')
  })

  it(`displays ${Message.ILLEGAL_FEE_PER_HOUR_IN_YEN_MESSAGE} (fee less than ${MIN_FEE_PER_HOUR_IN_YEN})`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_FEE_PER_HOUR_IN_YEN))
    postFeePerHourInYenFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(FeePerHourInYenPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const feeInputDiv = wrapper.find('[data-test="fee-input-div"]')
    const feeInput = feeInputDiv.find('input')
    await feeInput.setValue((MIN_FEE_PER_HOUR_IN_YEN - 1).toString())
    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.ILLEGAL_FEE_PER_HOUR_IN_YEN_MESSAGE)
    // UI上のロジックでエラーとするため、サーバから返されるエラーコードは表示されない
    // expect(resultMessage).toContain(Code.ILLEGAL_FEE_PER_HOUR_IN_YEN.toString())
  })

  it(`displays ${Message.ILLEGAL_FEE_PER_HOUR_IN_YEN_MESSAGE} fee more than ${MAX_FEE_PER_HOUR_IN_YEN})`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_FEE_PER_HOUR_IN_YEN))
    postFeePerHourInYenFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(FeePerHourInYenPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const feeInputDiv = wrapper.find('[data-test="fee-input-div"]')
    const feeInput = feeInputDiv.find('input')
    await feeInput.setValue((MAX_FEE_PER_HOUR_IN_YEN + 1).toString())
    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.ILLEGAL_FEE_PER_HOUR_IN_YEN_MESSAGE)
    // UI上のロジックでエラーとするため、サーバから返されるエラーコードは表示されない
    // expect(resultMessage).toContain(Code.ILLEGAL_FEE_PER_HOUR_IN_YEN.toString())
  })

  it(`displays ${Message.NO_IDENTITY_REGISTERED_MESSAGE} if ${Code.NO_IDENTITY_REGISTERED}) is returned`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_IDENTITY_REGISTERED))
    postFeePerHourInYenFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(FeePerHourInYenPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const feeInputDiv = wrapper.find('[data-test="fee-input-div"]')
    const feeInput = feeInputDiv.find('input')
    await feeInput.setValue(MAX_FEE_PER_HOUR_IN_YEN.toString())
    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.NO_IDENTITY_REGISTERED_MESSAGE)
    expect(resultMessage).toContain(Code.NO_IDENTITY_REGISTERED.toString())
  })

  it('displays AlertMessage when error has happened on postFeePerHourInYen', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const errDetail = 'connection error'
    postFeePerHourInYenFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(FeePerHourInYenPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const feeInputDiv = wrapper.find('[data-test="fee-input-div"]')
    const feeInput = feeInputDiv.find('input')
    await feeInput.setValue(MIN_FEE_PER_HOUR_IN_YEN.toString())
    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.UNEXPECTED_ERR)
    expect(resultMessage).toContain(errDetail)
  })

  it(`moves to login if postFeePerHourInYen returns ${Code.UNAUTHORIZED}`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    postFeePerHourInYenFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(FeePerHourInYenPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const feeInputDiv = wrapper.find('[data-test="fee-input-div"]')
    const feeInput = feeInputDiv.find('input')
    await feeInput.setValue(MIN_FEE_PER_HOUR_IN_YEN.toString())
    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/login')
  })

  it(`moves to terms-of-use if postFeePerHourInYen returns ${Code.NOT_TERMS_OF_USE_AGREED_YET}`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    postFeePerHourInYenFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(FeePerHourInYenPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const feeInputDiv = wrapper.find('[data-test="fee-input-div"]')
    const feeInput = feeInputDiv.find('input')
    await feeInput.setValue(MIN_FEE_PER_HOUR_IN_YEN.toString())
    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/terms-of-use')
  })
})
