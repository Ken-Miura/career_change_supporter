import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { mount, flushPromises, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import DeleteAccountConfirmationPage from '@/views/personalized/DeleteAccountConfirmationPage.vue'

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

const refreshDoneMock = ref(true)
const refreshFuncMock = jest.fn()
jest.mock('@/util/personalized/refresh/useRefresh', () => ({
  useRefresh: () => ({
    refreshDone: refreshDoneMock,
    refreshFunc: refreshFuncMock
  })
}))

const deleteAccountDoneMock = ref(true)
const deleteAccountFuncMock = jest.fn()
jest.mock('@/util/personalized/delete-account-confirmation/useDeleteAccount', () => ({
  useDeleteAccount: () => ({
    deleteAccountDone: deleteAccountDoneMock,
    deleteAccountFunc: deleteAccountFuncMock
  })
}))

describe('DeleteAccountConfirmationPage.spec.vue', () => {
  beforeEach(() => {
    refreshDoneMock.value = true
    refreshFuncMock.mockReset()
    deleteAccountDoneMock.value = true
    deleteAccountFuncMock.mockReset()
    routerPushMock.mockClear()
  })

  it('has WaitingCircle while calling refresh', async () => {
    refreshDoneMock.value = false
    refreshFuncMock.mockResolvedValue(RefreshResp.create())
    const wrapper = mount(DeleteAccountConfirmationPage, {
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

  it('has WaitingCircle while calling deleteAccount', async () => {
    deleteAccountDoneMock.value = false
    refreshFuncMock.mockResolvedValue(RefreshResp.create())
    const wrapper = mount(DeleteAccountConfirmationPage, {
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
})
