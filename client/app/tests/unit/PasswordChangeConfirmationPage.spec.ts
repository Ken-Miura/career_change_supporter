import PasswordChangeConfirmationPage from '@/views/PasswordChangeConfirmationPage.vue'
import { applyNewPassword } from '@/util/password/ApplyNewPassword'
import { mount, RouterLinkStub } from '@vue/test-utils'
import { ApplyNewPasswordResp } from '@/util/password/ApplyNewPasswordResp'
import { Message } from '@/util/Message'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'

jest.mock('@/util/password/ApplyNewPassword')
const applyNewPasswordMock = applyNewPassword as jest.MockedFunction<typeof applyNewPassword>

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

describe('PasswordChangeConfirmationPage.vue', () => {
  beforeEach(() => {
    applyNewPasswordMock.mockReset()
    queryObject = null
    routerPushMock.mockClear()
    storeCommitMock.mockClear()
  })

  it('has one button', () => {
    const wrapper = mount(PasswordChangeConfirmationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const buttons = wrapper.findAll('button')
    expect(buttons.length).toBe(1)
  })

  it(`moves to ApplyNewPasswordResultPage with ${Message.NEW_PASSWORD_APPLIED_MESSAGE} when new password is applied`, async () => {
    applyNewPasswordMock.mockResolvedValue(ApplyNewPasswordResp.create())
    queryObject = { 'new-password-id': 'bc999c52f1cc4801bfd9216cdebc0763' }
    const wrapper = mount(PasswordChangeConfirmationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const button = wrapper.find('button')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('apply-new-password-result')
    expect(storeCommitMock).toHaveBeenCalledTimes(1)
    expect(storeCommitMock).toHaveBeenCalledWith('setApplyNewPasswordResultMessage', `${Message.NEW_PASSWORD_APPLIED_MESSAGE}`)
  })

  it(`moves to ApplyNewPasswordResultPage with ${Message.INVALID_QUERY_PARAM} when query has no new-password-id`, async () => {
    applyNewPasswordMock.mockResolvedValue(ApplyNewPasswordResp.create())
    queryObject = { 'new-pwd': 'bc999c52f1cc4801bfd9216cdebc0763' }
    const wrapper = mount(PasswordChangeConfirmationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const button = wrapper.find('button')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('apply-new-password-result')
    expect(storeCommitMock).toHaveBeenCalledTimes(1)
    expect(storeCommitMock).toHaveBeenCalledWith('setApplyNewPasswordResultMessage', `${Message.INVALID_QUERY_PARAM}`)
  })

  it(`moves to ApplyNewPasswordResultPage with ${Message.INVALID_UUID_MESSAGE} when invalid uuid format is passed`, async () => {
    const apiErr = ApiError.create(Code.INVALID_UUID)
    applyNewPasswordMock.mockResolvedValue(ApiErrorResp.create(400, apiErr))
    queryObject = { 'new-password-id': /* 31桁 */ 'bc999c52f1cc4801bfd9216cdebc076' }
    const wrapper = mount(PasswordChangeConfirmationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const button = wrapper.find('button')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('apply-new-password-result')
    expect(storeCommitMock).toHaveBeenCalledTimes(1)
    expect(storeCommitMock).toHaveBeenCalledWith('setApplyNewPasswordResultMessage', `${Message.INVALID_UUID_MESSAGE} (${Code.INVALID_UUID})`)
  })

  it(`moves to ApplyNewPasswordResultPage with ${Message.NO_ACCOUNT_FOUND_MESSAGE} when account does not exist`, async () => {
    const apiErr = ApiError.create(Code.NO_ACCOUNT_FOUND)
    applyNewPasswordMock.mockResolvedValue(ApiErrorResp.create(400, apiErr))
    queryObject = { 'new-password-id': 'bc999c52f1cc4801bfd9216cdebc0763' }
    const wrapper = mount(PasswordChangeConfirmationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const button = wrapper.find('button')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('apply-new-password-result')
    expect(storeCommitMock).toHaveBeenCalledTimes(1)
    expect(storeCommitMock).toHaveBeenCalledWith('setApplyNewPasswordResultMessage', `${Message.NO_ACCOUNT_FOUND_MESSAGE} (${Code.NO_ACCOUNT_FOUND})`)
  })

  it(`moves to ApplyNewPasswordResultPage with ${Message.NO_NEW_PASSWORD_FOUND_MESSAGE} when new password is not found`, async () => {
    const apiErr = ApiError.create(Code.NO_NEW_PASSWORD_FOUND)
    applyNewPasswordMock.mockResolvedValue(ApiErrorResp.create(400, apiErr))
    queryObject = { 'new-password-id': 'bc999c52f1cc4801bfd9216cdebc0763' }
    const wrapper = mount(PasswordChangeConfirmationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const button = wrapper.find('button')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('apply-new-password-result')
    expect(storeCommitMock).toHaveBeenCalledTimes(1)
    expect(storeCommitMock).toHaveBeenCalledWith('setApplyNewPasswordResultMessage', `${Message.NO_NEW_PASSWORD_FOUND_MESSAGE} (${Code.NO_NEW_PASSWORD_FOUND})`)
  })

  it(`moves to ApplyNewPasswordResultPage with ${Message.NEW_PASSWORD_EXPIRED_MESSAGE} when new password has already expired`, async () => {
    const apiErr = ApiError.create(Code.NEW_PASSWORD_EXPIRED)
    applyNewPasswordMock.mockResolvedValue(ApiErrorResp.create(400, apiErr))
    queryObject = { 'new-password-id': 'bc999c52f1cc4801bfd9216cdebc0763' }
    const wrapper = mount(PasswordChangeConfirmationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const button = wrapper.find('button')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('apply-new-password-result')
    expect(storeCommitMock).toHaveBeenCalledTimes(1)
    expect(storeCommitMock).toHaveBeenCalledWith('setApplyNewPasswordResultMessage', `${Message.NEW_PASSWORD_EXPIRED_MESSAGE} (${Code.NEW_PASSWORD_EXPIRED})`)
  })
})
