import AccountCreated from '@/views/AccountCreated.vue'
import { createAccount } from '@/util/account/CreateAccount'
import { mount, RouterLinkStub } from '@vue/test-utils'
import { CreateAccountResp } from '@/util/account/CreateAccountResp'
import flushPromises from 'flush-promises'
import { Message } from '@/util/Message'

jest.mock('@/util/account/CreateAccount')
const createAccountMock = createAccount as jest.MockedFunction<typeof createAccount>

// TODO: クエリパラメータを含むrouterをより良い方法でモック化できる場合、そちらに改善する
let queryObject: any
jest.mock('vue-router', () => ({
  useRouter: () => ({
    currentRoute: { value: { query: queryObject } }
  })
}))

describe('AccountCreated.vue', () => {
  beforeEach(() => {
    createAccountMock.mockReset()
    queryObject = null
  })

  it(`displays ${Message.ACCOUNT_CREATED} when account is created`, async () => {
    createAccountMock.mockResolvedValue(CreateAccountResp.create())
    queryObject = { 'temp-account-id': 'bc999c52f1cc4801bfd9216cdebc0763' }
    const wrapper = mount(AccountCreated, {
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
    const wrapper = mount(AccountCreated, {
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
})
