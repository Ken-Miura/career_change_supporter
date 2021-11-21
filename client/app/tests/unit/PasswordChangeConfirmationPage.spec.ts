import PasswordChangeConfirmationPage from '@/views/PasswordChangeConfirmationPage.vue'
import { applyNewPassword } from '@/util/password/ApplyNewPassword'
import { mount, RouterLinkStub } from '@vue/test-utils'
import { ApplyNewPasswordResp } from '@/util/password/ApplyNewPasswordResp'
import flushPromises from 'flush-promises'
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

describe('PasswordChangeConfirmationPage.vue', () => {
  beforeEach(() => {
    applyNewPasswordMock.mockReset()
    queryObject = null
    routerPushMock.mockClear()
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
    const data = JSON.parse(`{ "name": "ApplyNewPasswordResultPage", "params": {"message": "${Message.NEW_PASSWORD_APPLIED_MESSAGE}"} }`)
    expect(routerPushMock).toHaveBeenCalledWith(data)
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
    const data = JSON.parse(`{ "name": "ApplyNewPasswordResultPage", "params": {"message": "${Message.INVALID_QUERY_PARAM}"} }`)
    expect(routerPushMock).toHaveBeenCalledWith(data)
  })

  //   it(`displays ${Message.INVALID_UUID_MESSAGE} when invalid uuid format is passed`, async () => {
  //     const apiErr = ApiError.create(Code.INVALID_UUID)
  //     applyNewPasswordMock.mockResolvedValue(ApiErrorResp.create(400, apiErr))
  //     queryObject = { 'new-password-id': /* 31桁 */ 'bc999c52f1cc4801bfd9216cdebc076' }
  //     const wrapper = mount(PasswordChangeConfirmationPage, {
  //       global: {
  //         stubs: {
  //           RouterLink: RouterLinkStub
  //         }
  //       }
  //     })
  //     await flushPromises()
  //     const mainTag = wrapper.find('main')
  //     const h3Tag = mainTag.find('h3')
  //     expect(h3Tag.text()).toMatch(`${Message.INVALID_UUID_MESSAGE}`)
  //   })

  //   it(`displays ${Message.ACCOUNT_ALREADY_EXISTS_MESSAGE} when account has already existed`, async () => {
  //     const apiErr = ApiError.create(Code.ACCOUNT_ALREADY_EXISTS)
  //     applyNewPasswordMock.mockResolvedValue(ApiErrorResp.create(400, apiErr))
  //     queryObject = { 'new-password-id': 'bc999c52f1cc4801bfd9216cdebc0763' }
  //     const wrapper = mount(PasswordChangeConfirmationPage, {
  //       global: {
  //         stubs: {
  //           RouterLink: RouterLinkStub
  //         }
  //       }
  //     })
  //     await flushPromises()
  //     const mainTag = wrapper.find('main')
  //     const h3Tag = mainTag.find('h3')
  //     expect(h3Tag.text()).toMatch(`${Message.ACCOUNT_ALREADY_EXISTS_MESSAGE}`)
  //   })

  //   it(`displays ${Message.NO_TEMP_ACCOUNT_FOUND_MESSAGE} when temp account id is not found`, async () => {
  //     const apiErr = ApiError.create(Code.NO_TEMP_ACCOUNT_FOUND)
  //     applyNewPasswordMock.mockResolvedValue(ApiErrorResp.create(400, apiErr))
  //     queryObject = { 'new-password-id': 'bc999c52f1cc4801bfd9216cdebc0763' }
  //     const wrapper = mount(PasswordChangeConfirmationPage, {
  //       global: {
  //         stubs: {
  //           RouterLink: RouterLinkStub
  //         }
  //       }
  //     })
  //     await flushPromises()
  //     const mainTag = wrapper.find('main')
  //     const h3Tag = mainTag.find('h3')
  //     expect(h3Tag.text()).toMatch(`${Message.NO_TEMP_ACCOUNT_FOUND_MESSAGE}`)
  //   })

//   it(`displays ${Message.TEMP_ACCOUNT_EXPIRED_MESSAGE} when temp account id has aleady expired`, async () => {
//     const apiErr = ApiError.create(Code.TEMP_ACCOUNT_EXPIRED)
//     applyNewPasswordMock.mockResolvedValue(ApiErrorResp.create(400, apiErr))
//     queryObject = { 'new-password-id': 'bc999c52f1cc4801bfd9216cdebc0763' }
//     const wrapper = mount(PasswordChangeConfirmationPage, {
//       global: {
//         stubs: {
//           RouterLink: RouterLinkStub
//         }
//       }
//     })
//     await flushPromises()
//     const mainTag = wrapper.find('main')
//     const h3Tag = mainTag.find('h3')
//     expect(h3Tag.text()).toMatch(`${Message.TEMP_ACCOUNT_EXPIRED_MESSAGE}`)
//   })
})
