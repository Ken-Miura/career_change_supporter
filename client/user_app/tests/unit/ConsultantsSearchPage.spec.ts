import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import ConsultantsSearchPage from '@/views/personalized/ConsultantsSearchPage.vue'
import { nextTick, reactive } from 'vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { refresh } from '@/util/personalized/refresh/Refresh'
import TheHeader from '@/components/TheHeader.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { Message } from '@/util/Message'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { PostIdentityResp } from '@/util/personalized/identity/PostIdentityResp'
import { getPageSize, PAGE_SIZE } from '@/util/PageSize'
import { ConsultantSearchParam } from '@/util/personalized/ConsultantSearchParam'

jest.mock('@/util/personalized/refresh/Refresh')
const refreshMock = refresh as jest.MockedFunction<typeof refresh>

jest.mock('@/util/PageSize')
const getPageSizeMock = getPageSize as jest.MockedFunction<typeof getPageSize>

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

let consultantSearchParamMock = null as ConsultantSearchParam | null
const storeCommitMock = jest.fn()
jest.mock('vuex', () => ({
  useStore: () => ({
    commit: storeCommitMock,
    state: {
      consultantSearchParam: consultantSearchParamMock
    }
  })
}))

describe('ConsultantsSearchPage.vue', () => {
  beforeEach(() => {
    refreshMock.mockReset()
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(PAGE_SIZE)
    routerPushMock.mockClear()
    storeCommitMock.mockClear()
    consultantSearchParamMock = null
  })

  it('has one TheHeader, one submit button and one AlertMessage', () => {
    const wrapper = mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const headers = wrapper.findAllComponents(TheHeader)
    expect(headers.length).toBe(1)
    const submitButton = wrapper.find('[data-test="submit-button"]')
    expect(submitButton.exists)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
  })

  it('has labels and inputs for search param', () => {
    const wrapper = mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const companyNameLabel = wrapper.find('[data-test="company-name-label"]')
    expect(companyNameLabel.exists)
    expect(companyNameLabel.text()).toContain('勤務先名称（例 xxx株式会社）')
    const companyNameInput = wrapper.find('[data-test="company-name-input"]').find('input')
    expect(companyNameInput.exists)
    // TODO: 項目の追加
  })

  it('has AlertMessage with a hidden attribute when created', () => {
    const wrapper = mount(ConsultantsSearchPage, {
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

  it(`moves to login if ${Code.UNAUTHORIZED} is returned on opening ConsultantsSearchPage`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    refreshMock.mockResolvedValue(apiErrResp)
    mount(ConsultantsSearchPage, {
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

  it(`moves to terms of use if ${Code.NOT_TERMS_OF_USE_AGREED_YET} is returned on opening ConsultantsSearchPage`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    refreshMock.mockResolvedValue(apiErrResp)
    mount(ConsultantsSearchPage, {
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

  it(`displays alert message ${Message.UNEXPECTED_ERR} when connection error happened on opening ConsultantsSearchPage`, async () => {
    const errDetail = 'connection error'
    refreshMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(ConsultantsSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

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
})
