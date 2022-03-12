import PasswordUpdatePage from '@/views/PasswordUpdatePage.vue'
import { updatePassword } from '@/util/password/UpdatePassword'
import { mount, RouterLinkStub } from '@vue/test-utils'
import { UpdatePasswordResp } from '@/util/password/UpdatePasswordResp'
import { Message } from '@/util/Message'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { SET_PASSWORD_UPDATE_RESULT_MESSAGE } from '@/store/mutationTypes'
import PasswordInput from '@/components/PasswordInput.vue'
import AlertMessage from '@/components/AlertMessage.vue'

jest.mock('@/util/password/UpdatePassword')
const updatePasswordMock = updatePassword as jest.MockedFunction<typeof updatePassword>

const routerPushMock = jest.fn()
// TODO: クエリパラメータを含むrouterをより良い方法でモック化できる場合、そちらに改善する
// eslint-disable-next-line
let queryObject: any
jest.mock('vue-router', () => ({
  useRouter: () => ({
    currentRoute: { value: { query: queryObject } },
    push: routerPushMock
  })
}))

const storeCommitMock = jest.fn()
jest.mock('vuex', () => ({
  useStore: () => ({
    commit: storeCommitMock
  })
}))

const PWD = 'abcdABCD1234'
const DIFFERENT_PWD = '1234abcdABCD'

describe('PasswordUpdatePage.vue', () => {
  beforeEach(() => {
    updatePasswordMock.mockReset()
    queryObject = null
    routerPushMock.mockClear()
    storeCommitMock.mockClear()
  })

  it('has two PasswordInputs and one AlertMessage', () => {
    const wrapper = mount(PasswordUpdatePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const passwords = wrapper.findAllComponents(PasswordInput)
    expect(passwords.length).toBe(2)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
  })

  it('has AlertMessage with a hidden attribute when created', () => {
    const wrapper = mount(PasswordUpdatePage, {
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

  it(`moves to PasswordUpdateResultPage with ${Message.PASSWORD_CHANGED_MESSAGE} when password is passed`, async () => {
    updatePasswordMock.mockResolvedValue(UpdatePasswordResp.create())
    queryObject = { 'pwd-change-req-id': 'bc999c52f1cc4801bfd9216cdebc0763' }
    const wrapper = mount(PasswordUpdatePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const pwds = wrapper.findAllComponents(PasswordInput)
    const pwdInput = pwds[0].find('input')
    await pwdInput.setValue(PWD)
    const pwdConfirmationInput = pwds[1].find('input')
    await pwdConfirmationInput.setValue(PWD)

    const button = wrapper.find('button')
    await button.trigger('submit')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('password-update-result')
    expect(storeCommitMock).toHaveBeenCalledTimes(1)
    expect(storeCommitMock).toHaveBeenCalledWith(SET_PASSWORD_UPDATE_RESULT_MESSAGE, `${Message.PASSWORD_CHANGED_MESSAGE}`)
  })

  it(`displays alert message ${Message.INVALID_QUERY_PARAM} when query has no pwd-change-req-id`, async () => {
    updatePasswordMock.mockResolvedValue(UpdatePasswordResp.create())
    queryObject = { 'pwd-change': 'bc999c52f1cc4801bfd9216cdebc0763' }
    const wrapper = mount(PasswordUpdatePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const pwds = wrapper.findAllComponents(PasswordInput)
    const pwdInput = pwds[0].find('input')
    await pwdInput.setValue(PWD)
    const pwdConfirmationInput = pwds[1].find('input')
    await pwdConfirmationInput.setValue(PWD)

    const button = wrapper.find('button')
    await button.trigger('submit')

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    expect(storeCommitMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_QUERY_PARAM)
  })

  it(`displays alert message ${Message.PASSWORD_CONFIRMATION_FAILED} when password and password confirmatin are different`, async () => {
    updatePasswordMock.mockResolvedValue(UpdatePasswordResp.create())
    queryObject = { 'pwd-change-req-id': 'bc999c52f1cc4801bfd9216cdebc0763' }
    const wrapper = mount(PasswordUpdatePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const pwds = wrapper.findAllComponents(PasswordInput)
    const pwdInput = pwds[0].find('input')
    await pwdInput.setValue(PWD)
    const pwdConfirmationInput = pwds[1].find('input')
    await pwdConfirmationInput.setValue(DIFFERENT_PWD)

    const button = wrapper.find('button')
    await button.trigger('submit')

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    expect(storeCommitMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.PASSWORD_CONFIRMATION_FAILED)
  })

  it(`moves PasswordUpdateResultPage with ${Message.INVALID_UUID_FORMAT_MESSAGE} when invalid uuid format is passed`, async () => {
    const apiErr = ApiError.create(Code.INVALID_UUID_FORMAT)
    updatePasswordMock.mockResolvedValue(ApiErrorResp.create(400, apiErr))
    queryObject = { 'pwd-change-req-id': /* 31桁 */ 'bc999c52f1cc4801bfd9216cdebc076' }
    const wrapper = mount(PasswordUpdatePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const pwds = wrapper.findAllComponents(PasswordInput)
    const pwdInput = pwds[0].find('input')
    await pwdInput.setValue(PWD)
    const pwdConfirmationInput = pwds[1].find('input')
    await pwdConfirmationInput.setValue(PWD)

    const button = wrapper.find('button')
    await button.trigger('submit')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('password-update-result')
    expect(storeCommitMock).toHaveBeenCalledTimes(1)
    expect(storeCommitMock).toHaveBeenCalledWith(SET_PASSWORD_UPDATE_RESULT_MESSAGE, `${Message.INVALID_UUID_FORMAT_MESSAGE} (${Code.INVALID_UUID_FORMAT})`)
  })

  it(`moves to PasswordUpdateResultPage with ${Message.NO_ACCOUNT_FOUND_MESSAGE} when account does not exist`, async () => {
    const apiErr = ApiError.create(Code.NO_ACCOUNT_FOUND)
    updatePasswordMock.mockResolvedValue(ApiErrorResp.create(400, apiErr))
    queryObject = { 'pwd-change-req-id': 'bc999c52f1cc4801bfd9216cdebc0763' }
    const wrapper = mount(PasswordUpdatePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const pwds = wrapper.findAllComponents(PasswordInput)
    const pwdInput = pwds[0].find('input')
    await pwdInput.setValue(PWD)
    const pwdConfirmationInput = pwds[1].find('input')
    await pwdConfirmationInput.setValue(PWD)

    const button = wrapper.find('button')
    await button.trigger('submit')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('password-update-result')
    expect(storeCommitMock).toHaveBeenCalledTimes(1)
    expect(storeCommitMock).toHaveBeenCalledWith(SET_PASSWORD_UPDATE_RESULT_MESSAGE, `${Message.NO_ACCOUNT_FOUND_MESSAGE} (${Code.NO_ACCOUNT_FOUND})`)
  })

  it(`moves to PasswordUpdateResultPage with ${Message.NO_PWD_CHANGE_REQ_FOUND_MESSAGE} when password change request is not found`, async () => {
    const apiErr = ApiError.create(Code.NO_PWD_CHANGE_REQ_FOUND)
    updatePasswordMock.mockResolvedValue(ApiErrorResp.create(400, apiErr))
    queryObject = { 'pwd-change-req-id': 'bc999c52f1cc4801bfd9216cdebc0763' }
    const wrapper = mount(PasswordUpdatePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const pwds = wrapper.findAllComponents(PasswordInput)
    const pwdInput = pwds[0].find('input')
    await pwdInput.setValue(PWD)
    const pwdConfirmationInput = pwds[1].find('input')
    await pwdConfirmationInput.setValue(PWD)

    const button = wrapper.find('button')
    await button.trigger('submit')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('password-update-result')
    expect(storeCommitMock).toHaveBeenCalledTimes(1)
    expect(storeCommitMock).toHaveBeenCalledWith(SET_PASSWORD_UPDATE_RESULT_MESSAGE, `${Message.NO_PWD_CHANGE_REQ_FOUND_MESSAGE} (${Code.NO_PWD_CHANGE_REQ_FOUND})`)
  })

  it(`moves to ApplyNewPasswordResultPage with ${Message.PWD_CHANGE_REQ_EXPIRED_MESSAGE} when new password has already expired`, async () => {
    const apiErr = ApiError.create(Code.PWD_CHANGE_REQ_EXPIRED)
    updatePasswordMock.mockResolvedValue(ApiErrorResp.create(400, apiErr))
    queryObject = { 'pwd-change-req-id': 'bc999c52f1cc4801bfd9216cdebc0763' }
    const wrapper = mount(PasswordUpdatePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const pwds = wrapper.findAllComponents(PasswordInput)
    const pwdInput = pwds[0].find('input')
    await pwdInput.setValue(PWD)
    const pwdConfirmationInput = pwds[1].find('input')
    await pwdConfirmationInput.setValue(PWD)

    const button = wrapper.find('button')
    await button.trigger('submit')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('password-update-result')
    expect(storeCommitMock).toHaveBeenCalledTimes(1)
    expect(storeCommitMock).toHaveBeenCalledWith(SET_PASSWORD_UPDATE_RESULT_MESSAGE, `${Message.PWD_CHANGE_REQ_EXPIRED_MESSAGE} (${Code.PWD_CHANGE_REQ_EXPIRED})`)
  })
})
