import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import IdentityPage from '@/views/personalized/IdentityPage.vue'
import { ref } from 'vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { refresh } from '@/util/personalized/refresh/Refresh'
import TheHeader from '@/components/TheHeader.vue'

const waitingPostIdentityDoneMock = ref(false)
const postIdentityFuncMock = jest.fn()
jest.mock('@/util/personalized/identity/usePostIdentity', () => ({
  usePostIdentity: () => ({
    waitingPostIdentityDone: waitingPostIdentityDoneMock,
    postIdentityFunc: postIdentityFuncMock
  })
}))

jest.mock('@/util/personalized/refresh/Refresh')
const refreshMock = refresh as jest.MockedFunction<typeof refresh>

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

const storeCommitMock = jest.fn()
jest.mock('vuex', () => ({
  useStore: () => ({
    commit: storeCommitMock
  })
}))

describe('IdentityPage.vue', () => {
  beforeEach(() => {
    waitingPostIdentityDoneMock.value = false
    postIdentityFuncMock.mockReset()
    refreshMock.mockReset()
    routerPushMock.mockClear()
    storeCommitMock.mockClear()
  })

  it('has one AlertMessage and one submit button', () => {
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const headers = wrapper.findAllComponents(TheHeader)
    expect(headers.length).toBe(1)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const submitButton = wrapper.find('[data-test="submit-button"]')
    expect(submitButton.exists)
  })

  it('has AlertMessage with a hidden attribute when created', () => {
    const wrapper = mount(IdentityPage, {
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

  it(`moves to login if ${Code.UNAUTHORIZED} is returned on opening IdentityPage`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    refreshMock.mockResolvedValue(apiErrResp)
    mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('login')
  })

  it(`moves to terms of use if ${Code.NOT_TERMS_OF_USE_AGREED_YET} is returned on opening IdentityPage`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    refreshMock.mockResolvedValue(apiErrResp)
    mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('terms-of-use')
  })
})
