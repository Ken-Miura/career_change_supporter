import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { mount, flushPromises, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import DeleteAccountConfirmationPage from '@/views/personalized/DeleteAccountConfirmationPage.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { Message } from '@/util/Message'
import { Code } from '@/util/Error'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { DeleteAccountResp } from '@/util/personalized/delete-account-confirmation/DeleteAccountResp'

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

  it('displays delete account confirmation information', async () => {
    refreshFuncMock.mockResolvedValue(RefreshResp.create())
    const wrapper = mount(DeleteAccountConfirmationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(0)

    const label = wrapper.find('[data-test="label"]')
    expect(label.text()).toContain('アカウントの削除')
    const confirmationLabel = wrapper.find('[data-test="confirmation-label"]')
    expect(confirmationLabel.text()).toContain('確認事項')
    const confirmationDescription = wrapper.find('[data-test="confirmation-description"]')
    expect(confirmationDescription.text()).toContain('私は下記に記載の内容を理解した上でアカウントの削除を行います。')
    const firstConfirmation = wrapper.find('[data-test="first-confirmation"]')
    expect(firstConfirmation.text()).toContain('未入金の報酬を受け取れなくなる可能性があることを理解しています。')
    const secondConfirmation = wrapper.find('[data-test="second-confirmation"]')
    expect(secondConfirmation.text()).toContain('相談料の返金依頼ができなくなることを理解しています。')
    const deleteAccountButton = wrapper.find('[data-test="delete-account-button"]')
    expect(deleteAccountButton.text()).toContain('アカウントを削除する')
    const buttonDisabledAttr = deleteAccountButton.attributes('disabled')
    expect(buttonDisabledAttr).toBeDefined()
  })

  it(`displays ${Message.UNAUTHORIZED_ON_ACCOUNT_DELETE_OPERATION_MESSAGE} if ${Code.UNAUTHORIZED} is returned when refresh`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    refreshFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(DeleteAccountConfirmationPage, {
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
    expect(resultMessage).toContain(Message.UNAUTHORIZED_ON_ACCOUNT_DELETE_OPERATION_MESSAGE)
  })

  it(`displays ${Message.NOT_TERMS_OF_USE_AGREED_YET_ON_ACCOUNT_DELETE_OPERATION_MESSAGE} if ${Code.NOT_TERMS_OF_USE_AGREED_YET} is returned when refresh`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    refreshFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(DeleteAccountConfirmationPage, {
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
    expect(resultMessage).toContain(Message.NOT_TERMS_OF_USE_AGREED_YET_ON_ACCOUNT_DELETE_OPERATION_MESSAGE)
  })

  it('displays alert message if connection fails when refresh', async () => {
    const errDetail = 'connection error'
    refreshFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(DeleteAccountConfirmationPage, {
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

  it('moves delete-account-success if account delete is successful', async () => {
    refreshFuncMock.mockResolvedValue(RefreshResp.create())
    deleteAccountFuncMock.mockResolvedValue(DeleteAccountResp.create())
    const wrapper = mount(DeleteAccountConfirmationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const accountDeleteConfirmed = wrapper.find('[data-test="account-delete-confirmed"]')
    await accountDeleteConfirmed.setValue(true)
    await flushPromises()

    const deleteAccountButton = wrapper.find('[data-test="delete-account-button"]')
    await deleteAccountButton.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/delete-account-success')
  })

  it(`displays ${Message.UNAUTHORIZED_ON_ACCOUNT_DELETE_OPERATION_MESSAGE} if ${Code.UNAUTHORIZED} is returned when account delete`, async () => {
    refreshFuncMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    deleteAccountFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(DeleteAccountConfirmationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const accountDeleteConfirmed = wrapper.find('[data-test="account-delete-confirmed"]')
    await accountDeleteConfirmed.setValue(true)
    await flushPromises()

    const deleteAccountButton = wrapper.find('[data-test="delete-account-button"]')
    await deleteAccountButton.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.UNAUTHORIZED_ON_ACCOUNT_DELETE_OPERATION_MESSAGE)
  })

  it(`displays ${Message.NOT_TERMS_OF_USE_AGREED_YET_ON_ACCOUNT_DELETE_OPERATION_MESSAGE} if ${Code.NOT_TERMS_OF_USE_AGREED_YET} is returned when account delete`, async () => {
    refreshFuncMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    deleteAccountFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(DeleteAccountConfirmationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const accountDeleteConfirmed = wrapper.find('[data-test="account-delete-confirmed"]')
    await accountDeleteConfirmed.setValue(true)
    await flushPromises()

    const deleteAccountButton = wrapper.find('[data-test="delete-account-button"]')
    await deleteAccountButton.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.NOT_TERMS_OF_USE_AGREED_YET_ON_ACCOUNT_DELETE_OPERATION_MESSAGE)
  })

  it(`displays ${Message.ACCOUNT_DELETE_IS_NOT_CONFIRMED_MESSAGE} if ${Code.ACCOUNT_DELETE_IS_NOT_CONFIRMED} is returned when account delete`, async () => {
    refreshFuncMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ACCOUNT_DELETE_IS_NOT_CONFIRMED))
    deleteAccountFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(DeleteAccountConfirmationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const accountDeleteConfirmed = wrapper.find('[data-test="account-delete-confirmed"]')
    await accountDeleteConfirmed.setValue(true)
    await flushPromises()

    const deleteAccountButton = wrapper.find('[data-test="delete-account-button"]')
    await deleteAccountButton.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.ACCOUNT_DELETE_IS_NOT_CONFIRMED_MESSAGE)
    expect(resultMessage).toContain(Code.ACCOUNT_DELETE_IS_NOT_CONFIRMED.toString())
  })

  it(`displays ${Message.CONSULTATION_HAS_NOT_BEEN_FINISHED_MESSAGE} if ${Code.CONSULTATION_HAS_NOT_BEEN_FINISHED} is returned when account delete`, async () => {
    refreshFuncMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.CONSULTATION_HAS_NOT_BEEN_FINISHED))
    deleteAccountFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(DeleteAccountConfirmationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const accountDeleteConfirmed = wrapper.find('[data-test="account-delete-confirmed"]')
    await accountDeleteConfirmed.setValue(true)
    await flushPromises()

    const deleteAccountButton = wrapper.find('[data-test="delete-account-button"]')
    await deleteAccountButton.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.CONSULTATION_HAS_NOT_BEEN_FINISHED_MESSAGE)
    expect(resultMessage).toContain(Code.CONSULTATION_HAS_NOT_BEEN_FINISHED.toString())
  })

  it('displays alert message if connection fails when account delete', async () => {
    refreshFuncMock.mockResolvedValue(RefreshResp.create())
    const errDetail = 'connection error'
    deleteAccountFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(DeleteAccountConfirmationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const accountDeleteConfirmed = wrapper.find('[data-test="account-delete-confirmed"]')
    await accountDeleteConfirmed.setValue(true)
    await flushPromises()

    const deleteAccountButton = wrapper.find('[data-test="delete-account-button"]')
    await deleteAccountButton.trigger('click')
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
