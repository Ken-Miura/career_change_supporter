import { UserAccountSearchParam } from '@/util/personalized/user-account-search/UserAccountSearchParam'
import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import TheHeader from '@/components/TheHeader.vue'
import UserAccountSearchPage from '@/views/personalized/UserAccountSearchPage.vue'
import { SET_USER_ACCOUNT_SEARCH_PARAM } from '@/store/mutationTypes'

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

let userAccountSearchParamMock = null as UserAccountSearchParam | null
const storeCommitMock = jest.fn()
jest.mock('vuex', () => ({
  useStore: () => ({
    commit: storeCommitMock,
    state: {
      userAccountSearchParam: userAccountSearchParamMock
    }
  })
}))

describe('UserAccountSearchPage.vue', () => {
  beforeEach(() => {
    userAccountSearchParamMock = null
    storeCommitMock.mockClear()
    routerPushMock.mockClear()
  })

  it('has header', async () => {
    const wrapper = mount(UserAccountSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const headers = wrapper.findAllComponents(TheHeader)
    expect(headers.length).toBe(1)
  })

  it('diaplays contents for UserAccountSearchPage', async () => {
    const wrapper = mount(UserAccountSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const headers = wrapper.findAllComponents(TheHeader)
    expect(headers.length).toBe(1)

    const label = wrapper.find('[data-test="label"]')
    expect(label.text()).toContain('アカウント検索')
    const description = wrapper.find('[data-test="description"]')
    expect(description.text()).toContain('アカウントID、またはメールアドレスを入力して検索を押して下さい。既に削除されたアカウントは、メールアドレスではなくアカウントIDで検索して下さい。削除されたアカウントは、そうでないアカウントと比較し、確認できる情報が限定されています。')
    const accountIdLabel = wrapper.find('[data-test="account-id-label"]')
    expect(accountIdLabel.text()).toContain('アカウントID')
    const emailAddressLabel = wrapper.find('[data-test="email-address-label"]')
    expect(emailAddressLabel.text()).toContain('メールアドレス')

    const button = wrapper.find('[data-test="button"]')
    expect(button.text()).toContain('検索')
    const buttonDisabledAttr = button.attributes('disabled')
    expect(buttonDisabledAttr).toBeDefined()
  })

  it('diaplays disabled button if both input is filled', async () => {
    const wrapper = mount(UserAccountSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const accountIdValue = wrapper.find('[data-test="account-id-value"]')
    await accountIdValue.setValue('1')
    const emailAddressValue = wrapper.find('[data-test="email-address-value"]')
    await emailAddressValue.setValue('test@test.com')
    await flushPromises()

    const button = wrapper.find('[data-test="button"]')
    expect(button.text()).toContain('検索')
    const buttonDisabledAttr = button.attributes('disabled')
    expect(buttonDisabledAttr).toBeDefined()
  })

  it('diaplays disabled button if both input is empty', async () => {
    const wrapper = mount(UserAccountSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const accountIdValue = wrapper.find('[data-test="account-id-value"]')
    await accountIdValue.setValue('')
    const emailAddressValue = wrapper.find('[data-test="email-address-value"]')
    await emailAddressValue.setValue('')
    await flushPromises()

    const button = wrapper.find('[data-test="button"]')
    expect(button.text()).toContain('検索')
    const buttonDisabledAttr = button.attributes('disabled')
    expect(buttonDisabledAttr).toBeDefined()
  })

  it('moves user-account-info if button is clicked with account id', async () => {
    const wrapper = mount(UserAccountSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const accountIdValue = wrapper.find('[data-test="account-id-value"]')
    await accountIdValue.setValue('1')
    const emailAddressValue = wrapper.find('[data-test="email-address-value"]')
    await emailAddressValue.setValue('')
    await flushPromises()

    const button = wrapper.find('[data-test="button"]')
    await button.trigger('submit')
    await flushPromises()

    const param = {
      accountId: 1,
      emailAddress: null
    } as UserAccountSearchParam
    expect(storeCommitMock).toHaveBeenCalledTimes(1)
    expect(storeCommitMock).toHaveBeenNthCalledWith(1, SET_USER_ACCOUNT_SEARCH_PARAM, param)

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/user-account-info')
  })

  it('moves user-account-info if button is clicked with email address', async () => {
    const wrapper = mount(UserAccountSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const accountIdValue = wrapper.find('[data-test="account-id-value"]')
    await accountIdValue.setValue('')
    const emailAddressValue = wrapper.find('[data-test="email-address-value"]')
    await emailAddressValue.setValue('test@test.com')
    await flushPromises()

    const button = wrapper.find('[data-test="button"]')
    await button.trigger('submit')
    await flushPromises()

    const param = {
      accountId: null,
      emailAddress: 'test@test.com'
    } as UserAccountSearchParam
    expect(storeCommitMock).toHaveBeenCalledTimes(1)
    expect(storeCommitMock).toHaveBeenNthCalledWith(1, SET_USER_ACCOUNT_SEARCH_PARAM, param)

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/user-account-info')
  })
})
