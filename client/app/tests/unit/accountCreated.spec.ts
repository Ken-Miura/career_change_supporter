import AccountCreated from '@/views/AccountCreated.vue'
import { createAccount } from '@/util/account/CreateAccount'
import { mount, RouterLinkStub } from '@vue/test-utils'
import { CreateAccountResp } from '@/util/account/CreateAccountResp'
import flushPromises from 'flush-promises'
import { Message } from '@/util/Message'

jest.mock('@/util/account/CreateAccount')
const createAccountMock = createAccount as jest.MockedFunction<typeof createAccount>

jest.mock('vue-router', () => ({
  useRouter: () => ({
    currentRoute: { value: { query: { 'temp-account-id': 'bc999c52f1cc4801bfd9216cdebc0763' } } }
  })
}))

describe('AccountCreated.vue', () => {
  beforeEach(() => {
    createAccountMock.mockReset()
  })

  it(`displays ${Message.ACCOUNT_CREATED} when account is created`, async () => {
    createAccountMock.mockResolvedValue(CreateAccountResp.create())
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
})
