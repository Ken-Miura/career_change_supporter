import { ref } from 'vue'

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

  it('has WaitingCircle while calling ', async () => {
    console.log('test')
  })
})
