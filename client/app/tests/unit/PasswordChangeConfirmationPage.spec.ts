import AccountCreationPage from '@/views/AccountCreationPage.vue'
import { createAccount } from '@/util/account/CreateAccount'
import { mount, RouterLinkStub } from '@vue/test-utils'
import { CreateAccountResp } from '@/util/account/CreateAccountResp'
import flushPromises from 'flush-promises'
import { Message } from '@/util/Message'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'

jest.mock('@/util/account/CreateAccount')
const createAccountMock = createAccount as jest.MockedFunction<typeof createAccount>

// TODO: クエリパラメータを含むrouterをより良い方法でモック化できる場合、そちらに改善する
// eslint-disable-next-line
let queryObject: any
jest.mock('vue-router', () => ({
  useRouter: () => ({
    currentRoute: { value: { query: queryObject } }
  })
}))

describe('AccountCreationPage.vue', () => {
  beforeEach(() => {
    createAccountMock.mockReset()
    queryObject = null
  })

  it(`displays ${Message.ACCOUNT_CREATED} when account is created`, async () => {
    createAccountMock.mockResolvedValue(CreateAccountResp.create())
    queryObject = { 'temp-account-id': 'bc999c52f1cc4801bfd9216cdebc0763' }
    const wrapper = mount(AccountCreationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    const mainTag = wrapper.find('main')
    const h3Tag = mainTag.find('h3')
    expect(h3Tag.text()).toMatch(`${Message.ACCOUNT_CREATED}`)
  })

  it(`displays ${Message.INVALID_QUERY_PARAM} when query has no temp-account-id`, async () => {
    createAccountMock.mockResolvedValue(CreateAccountResp.create())
    queryObject = { 'temp-acc': 'bc999c52f1cc4801bfd9216cdebc0763' }
    const wrapper = mount(AccountCreationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    const mainTag = wrapper.find('main')
    const h3Tag = mainTag.find('h3')
    expect(h3Tag.text()).toMatch(`${Message.INVALID_QUERY_PARAM}`)
  })

  it(`displays ${Message.INVALID_UUID_MESSAGE} when invalid uuid format is passed`, async () => {
    const apiErr = ApiError.create(Code.INVALID_UUID)
    createAccountMock.mockResolvedValue(ApiErrorResp.create(400, apiErr))
    queryObject = { 'temp-account-id': /* 31桁 */ 'bc999c52f1cc4801bfd9216cdebc076' }
    const wrapper = mount(AccountCreationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    const mainTag = wrapper.find('main')
    const h3Tag = mainTag.find('h3')
    expect(h3Tag.text()).toMatch(`${Message.INVALID_UUID_MESSAGE}`)
  })

  it(`displays ${Message.ACCOUNT_ALREADY_EXISTS_MESSAGE} when account has already existed`, async () => {
    const apiErr = ApiError.create(Code.ACCOUNT_ALREADY_EXISTS)
    createAccountMock.mockResolvedValue(ApiErrorResp.create(400, apiErr))
    queryObject = { 'temp-account-id': 'bc999c52f1cc4801bfd9216cdebc0763' }
    const wrapper = mount(AccountCreationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    const mainTag = wrapper.find('main')
    const h3Tag = mainTag.find('h3')
    expect(h3Tag.text()).toMatch(`${Message.ACCOUNT_ALREADY_EXISTS_MESSAGE}`)
  })

  it(`displays ${Message.NO_TEMP_ACCOUNT_FOUND_MESSAGE} when temp account id is not found`, async () => {
    const apiErr = ApiError.create(Code.NO_TEMP_ACCOUNT_FOUND)
    createAccountMock.mockResolvedValue(ApiErrorResp.create(400, apiErr))
    queryObject = { 'temp-account-id': 'bc999c52f1cc4801bfd9216cdebc0763' }
    const wrapper = mount(AccountCreationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    const mainTag = wrapper.find('main')
    const h3Tag = mainTag.find('h3')
    expect(h3Tag.text()).toMatch(`${Message.NO_TEMP_ACCOUNT_FOUND_MESSAGE}`)
  })

  it(`displays ${Message.TEMP_ACCOUNT_EXPIRED_MESSAGE} when temp account id has aleady expired`, async () => {
    const apiErr = ApiError.create(Code.TEMP_ACCOUNT_EXPIRED)
    createAccountMock.mockResolvedValue(ApiErrorResp.create(400, apiErr))
    queryObject = { 'temp-account-id': 'bc999c52f1cc4801bfd9216cdebc0763' }
    const wrapper = mount(AccountCreationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    const mainTag = wrapper.find('main')
    const h3Tag = mainTag.find('h3')
    expect(h3Tag.text()).toMatch(`${Message.TEMP_ACCOUNT_EXPIRED_MESSAGE}`)
  })
})
