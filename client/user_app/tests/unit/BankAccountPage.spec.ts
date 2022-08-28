import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import BankAccountPage from '@/views/personalized/BankAccountPage.vue'
import TheHeader from '@/components/TheHeader.vue'
import { refresh } from '@/util/personalized/refresh/Refresh'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { Message } from '@/util/Message'
import { Code } from '@/util/Error'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { BankAccount } from '@/util/personalized/BankAccount'
import { PostBankAccountResp } from '@/util/personalized/bank-account/PostBankAccountResp'

jest.mock('@/util/personalized/refresh/Refresh')
const refreshMock = refresh as jest.MockedFunction<typeof refresh>

const postBankAccountDoneMock = ref(true)
const postBankAccountFuncMock = jest.fn()
jest.mock('@/util/personalized/bank-account/usePostBankAccount', () => ({
  usePostBankAccount: () => ({
    postBankAccountDone: postBankAccountDoneMock,
    postBankAccountFunc: postBankAccountFuncMock
  })
}))

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

let bankAccountMock = null as BankAccount | null
const storeCommitMock = jest.fn()
jest.mock('vuex', () => ({
  useStore: () => ({
    commit: storeCommitMock,
    state: {
      bankAccount: bankAccountMock
    }
  })
}))

describe('BankAccountPage.vue', () => {
  beforeEach(() => {
    bankAccountMock = null
    refreshMock.mockReset()
    postBankAccountDoneMock.value = true
    postBankAccountFuncMock.mockReset()
    routerPushMock.mockClear()
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    postBankAccountDoneMock.value = false
    const resp = PostBankAccountResp.create()
    postBankAccountFuncMock.mockResolvedValue(resp)
    const wrapper = mount(BankAccountPage, {
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

  it('displays AlertMessage when error has happened on refresh', async () => {
    const errDetail = 'connection error'
    refreshMock.mockRejectedValue(new Error(errDetail))
    const resp = PostBankAccountResp.create()
    postBankAccountFuncMock.mockResolvedValue(resp)
    const wrapper = mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.UNEXPECTED_ERR)
    expect(resultMessage).toContain(errDetail)
  })

  it(`moves to login if refresh returns ${Code.UNAUTHORIZED}`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    refreshMock.mockResolvedValue(apiErrResp)
    const resp = PostBankAccountResp.create()
    postBankAccountFuncMock.mockResolvedValue(resp)
    mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/login')
  })

  it(`moves to terms-of-use if refresh returns ${Code.NOT_TERMS_OF_USE_AGREED_YET}`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    refreshMock.mockResolvedValue(apiErrResp)
    const resp = PostBankAccountResp.create()
    postBankAccountFuncMock.mockResolvedValue(resp)
    mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/terms-of-use')
  })

  it('displays initial value and moves submit-bank-account-success if button is clicked', async () => {
    bankAccountMock = {
      bank_code: '0001',
      branch_code: '001',
      account_type: '普通',
      account_number: '1234567',
      account_holder_name: 'タナカ　タロウ'
    } as BankAccount
    refreshMock.mockResolvedValue(RefreshResp.create())
    const resp = PostBankAccountResp.create()
    postBankAccountFuncMock.mockResolvedValue(resp)
    const wrapper = mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const bankCodeDiv = wrapper.find('[data-test="bank-code-div"]')
    const bankCodeInput = bankCodeDiv.find('input')
    expect(bankCodeInput.element.value).toEqual(bankAccountMock.bank_code)
    const branchCodeDiv = wrapper.find('[data-test="branch-code-div"]')
    const branchCodeInput = branchCodeDiv.find('input')
    expect(branchCodeInput.element.value).toEqual(bankAccountMock.branch_code)
    const accountTypeDiv = wrapper.find('[data-test="account-type-div"]')
    const accountTypeLabel = accountTypeDiv.find('label')
    if (!accountTypeLabel.element.textContent) {
      throw new Error('!accountTypeLabel.element.textContent')
    }
    expect(accountTypeLabel.element.textContent).toEqual(bankAccountMock.account_type)
    const accountNumberDiv = wrapper.find('[data-test="account-number-div"]')
    const accountNumberInput = accountNumberDiv.find('input')
    expect(accountNumberInput.element.value).toEqual(bankAccountMock.account_number)
    const accountHolderNameDiv = wrapper.find('[data-test="account-holder-name-div"]')
    const accountHolderNameInput = accountHolderNameDiv.find('input')
    expect(accountHolderNameInput.element.value).toEqual(bankAccountMock.account_holder_name)

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/submit-bank-account-success')
  })

  it('moves submit-bank-account-success if value is set and button is clicked', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const resp = PostBankAccountResp.create()
    postBankAccountFuncMock.mockResolvedValue(resp)
    const wrapper = mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const bankCodeDiv = wrapper.find('[data-test="bank-code-div"]')
    const bankCodeInput = bankCodeDiv.find('input')
    await bankCodeInput.setValue('0001')
    const branchCodeDiv = wrapper.find('[data-test="branch-code-div"]')
    const branchCodeInput = branchCodeDiv.find('input')
    await branchCodeInput.setValue('001')
    const accountNumberDiv = wrapper.find('[data-test="account-number-div"]')
    const accountNumberInput = accountNumberDiv.find('input')
    await accountNumberInput.setValue('1234567')
    const accountHolderNameDiv = wrapper.find('[data-test="account-holder-name-div"]')
    const accountHolderNameInput = accountHolderNameDiv.find('input')
    await accountHolderNameInput.setValue('タナカ　タロウ')

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/submit-bank-account-success')
  })

  it(`displays ${Message.INVALID_BANK_CODE_FORMAT_MESSAGE} if ${Code.INVALID_BANK_CODE_FORMAT} is returned`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_BANK_CODE_FORMAT))
    postBankAccountFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const bankCodeDiv = wrapper.find('[data-test="bank-code-div"]')
    const bankCodeInput = bankCodeDiv.find('input')
    await bankCodeInput.setValue('001') // 不当な銀行コード
    const branchCodeDiv = wrapper.find('[data-test="branch-code-div"]')
    const branchCodeInput = branchCodeDiv.find('input')
    await branchCodeInput.setValue('001')
    const accountNumberDiv = wrapper.find('[data-test="account-number-div"]')
    const accountNumberInput = accountNumberDiv.find('input')
    await accountNumberInput.setValue('1234567')
    const accountHolderNameDiv = wrapper.find('[data-test="account-holder-name-div"]')
    const accountHolderNameInput = accountHolderNameDiv.find('input')
    await accountHolderNameInput.setValue('タナカ　タロウ')

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_BANK_CODE_FORMAT_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_BANK_CODE_FORMAT.toString())
  })

  it(`displays ${Message.INVALID_BRANCH_CODE_FORMAT_MESSAGE} if ${Code.INVALID_BRANCH_CODE_FORMAT} is returned`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_BRANCH_CODE_FORMAT))
    postBankAccountFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const bankCodeDiv = wrapper.find('[data-test="bank-code-div"]')
    const bankCodeInput = bankCodeDiv.find('input')
    await bankCodeInput.setValue('0001')
    const branchCodeDiv = wrapper.find('[data-test="branch-code-div"]')
    const branchCodeInput = branchCodeDiv.find('input')
    await branchCodeInput.setValue('0001') // 不当な支店コード
    const accountNumberDiv = wrapper.find('[data-test="account-number-div"]')
    const accountNumberInput = accountNumberDiv.find('input')
    await accountNumberInput.setValue('1234567')
    const accountHolderNameDiv = wrapper.find('[data-test="account-holder-name-div"]')
    const accountHolderNameInput = accountHolderNameDiv.find('input')
    await accountHolderNameInput.setValue('タナカ　タロウ')

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_BRANCH_CODE_FORMAT_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_BRANCH_CODE_FORMAT.toString())
  })

  // 預金種別はUIから入力できないので、送信するリクエストを改変する必要がある。そのため、通常操作では発生しないがテストコードは作成しておく。
  it(`displays ${Message.INVALID_ACCOUNT_TYPE_MESSAGE} if ${Code.INVALID_ACCOUNT_TYPE} is returned`, async () => {
    bankAccountMock = {
      bank_code: '0001',
      branch_code: '001',
      account_type: '当座',
      account_number: '1234567',
      account_holder_name: 'タナカ　タロウ'
    } as BankAccount
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_ACCOUNT_TYPE))
    postBankAccountFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_ACCOUNT_TYPE_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_ACCOUNT_TYPE.toString())
  })

  it(`displays ${Message.INVALID_ACCOUNT_NUMBER_FORMAT_MESSAGE} if ${Code.INVALID_ACCOUNT_NUMBER_FORMAT} is returned`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_ACCOUNT_NUMBER_FORMAT))
    postBankAccountFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const bankCodeDiv = wrapper.find('[data-test="bank-code-div"]')
    const bankCodeInput = bankCodeDiv.find('input')
    await bankCodeInput.setValue('0001')
    const branchCodeDiv = wrapper.find('[data-test="branch-code-div"]')
    const branchCodeInput = branchCodeDiv.find('input')
    await branchCodeInput.setValue('001')
    const accountNumberDiv = wrapper.find('[data-test="account-number-div"]')
    const accountNumberInput = accountNumberDiv.find('input')
    await accountNumberInput.setValue('123456') // 不当な口座番号
    const accountHolderNameDiv = wrapper.find('[data-test="account-holder-name-div"]')
    const accountHolderNameInput = accountHolderNameDiv.find('input')
    await accountHolderNameInput.setValue('タナカ　タロウ')

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_ACCOUNT_NUMBER_FORMAT_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_ACCOUNT_NUMBER_FORMAT.toString())
  })

  it(`displays ${Message.INVALID_ACCOUNT_HOLDER_NAME_LENGTH_MESSAGE} if ${Code.INVALID_ACCOUNT_HOLDER_NAME_LENGTH} is returned`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_ACCOUNT_HOLDER_NAME_LENGTH))
    postBankAccountFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const bankCodeDiv = wrapper.find('[data-test="bank-code-div"]')
    const bankCodeInput = bankCodeDiv.find('input')
    await bankCodeInput.setValue('0001')
    const branchCodeDiv = wrapper.find('[data-test="branch-code-div"]')
    const branchCodeInput = branchCodeDiv.find('input')
    await branchCodeInput.setValue('001')
    const accountNumberDiv = wrapper.find('[data-test="account-number-div"]')
    const accountNumberInput = accountNumberDiv.find('input')
    await accountNumberInput.setValue('1234567')
    const accountHolderNameDiv = wrapper.find('[data-test="account-holder-name-div"]')
    const accountHolderNameInput = accountHolderNameDiv.find('input')
    await accountHolderNameInput.setValue('イ')

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_ACCOUNT_HOLDER_NAME_LENGTH_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_ACCOUNT_HOLDER_NAME_LENGTH.toString())
  })

  it(`displays ${Message.ILLEGAL_CHAR_IN_ACCOUNT_HOLDER_NAME_MESSAGE} if ${Code.ILLEGAL_CHAR_IN_ACCOUNT_HOLDER_NAME} is returned`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CHAR_IN_ACCOUNT_HOLDER_NAME))
    postBankAccountFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const bankCodeDiv = wrapper.find('[data-test="bank-code-div"]')
    const bankCodeInput = bankCodeDiv.find('input')
    await bankCodeInput.setValue('0001')
    const branchCodeDiv = wrapper.find('[data-test="branch-code-div"]')
    const branchCodeInput = branchCodeDiv.find('input')
    await branchCodeInput.setValue('001')
    const accountNumberDiv = wrapper.find('[data-test="account-number-div"]')
    const accountNumberInput = accountNumberDiv.find('input')
    await accountNumberInput.setValue('1234567')
    const accountHolderNameDiv = wrapper.find('[data-test="account-holder-name-div"]')
    const accountHolderNameInput = accountHolderNameDiv.find('input')
    await accountHolderNameInput.setValue('イ')

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.ILLEGAL_CHAR_IN_ACCOUNT_HOLDER_NAME_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CHAR_IN_ACCOUNT_HOLDER_NAME.toString())
  })

  it(`displays ${Message.ACCOUNT_HOLDER_NAME_DOES_NOT_MATCH_FULL_NAME_MESSAGE} if ${Code.ACCOUNT_HOLDER_NAME_DOES_NOT_MATCH_FULL_NAME} is returned`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ACCOUNT_HOLDER_NAME_DOES_NOT_MATCH_FULL_NAME))
    postBankAccountFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const bankCodeDiv = wrapper.find('[data-test="bank-code-div"]')
    const bankCodeInput = bankCodeDiv.find('input')
    await bankCodeInput.setValue('0001')
    const branchCodeDiv = wrapper.find('[data-test="branch-code-div"]')
    const branchCodeInput = branchCodeDiv.find('input')
    await branchCodeInput.setValue('001')
    const accountNumberDiv = wrapper.find('[data-test="account-number-div"]')
    const accountNumberInput = accountNumberDiv.find('input')
    await accountNumberInput.setValue('1234567')
    const accountHolderNameDiv = wrapper.find('[data-test="account-holder-name-div"]')
    const accountHolderNameInput = accountHolderNameDiv.find('input')
    await accountHolderNameInput.setValue('タナカ　タロウ')

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.ACCOUNT_HOLDER_NAME_DOES_NOT_MATCH_FULL_NAME_MESSAGE)
    expect(resultMessage).toContain(Code.ACCOUNT_HOLDER_NAME_DOES_NOT_MATCH_FULL_NAME.toString())
  })

  it(`displays ${Message.INVALID_BANK_MESSAGE} if ${Code.INVALID_BANK} is returned`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_BANK))
    postBankAccountFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const bankCodeDiv = wrapper.find('[data-test="bank-code-div"]')
    const bankCodeInput = bankCodeDiv.find('input')
    await bankCodeInput.setValue('0004') // 存在しない銀行コード
    const branchCodeDiv = wrapper.find('[data-test="branch-code-div"]')
    const branchCodeInput = branchCodeDiv.find('input')
    await branchCodeInput.setValue('001')
    const accountNumberDiv = wrapper.find('[data-test="account-number-div"]')
    const accountNumberInput = accountNumberDiv.find('input')
    await accountNumberInput.setValue('1234567')
    const accountHolderNameDiv = wrapper.find('[data-test="account-holder-name-div"]')
    const accountHolderNameInput = accountHolderNameDiv.find('input')
    await accountHolderNameInput.setValue('タナカ　タロウ')

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_BANK_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_BANK.toString())
  })

  it(`displays ${Message.INVALID_BANK_BRANCH_MESSAGE} if ${Code.INVALID_BANK_BRANCH} is returned`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_BANK_BRANCH))
    postBankAccountFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const bankCodeDiv = wrapper.find('[data-test="bank-code-div"]')
    const bankCodeInput = bankCodeDiv.find('input')
    await bankCodeInput.setValue('0001')
    const branchCodeDiv = wrapper.find('[data-test="branch-code-div"]')
    const branchCodeInput = branchCodeDiv.find('input')
    await branchCodeInput.setValue('002') // 存在しない支店コード
    const accountNumberDiv = wrapper.find('[data-test="account-number-div"]')
    const accountNumberInput = accountNumberDiv.find('input')
    await accountNumberInput.setValue('1234567')
    const accountHolderNameDiv = wrapper.find('[data-test="account-holder-name-div"]')
    const accountHolderNameInput = accountHolderNameDiv.find('input')
    await accountHolderNameInput.setValue('タナカ　タロウ')

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_BANK_BRANCH_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_BANK_BRANCH.toString())
  })

  it(`displays ${Message.INVALID_BANK_ACCOUNT_NUMBER_MESSAGE} if ${Code.INVALID_BANK_ACCOUNT_NUMBER} is returned`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_BANK_ACCOUNT_NUMBER))
    postBankAccountFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const bankCodeDiv = wrapper.find('[data-test="bank-code-div"]')
    const bankCodeInput = bankCodeDiv.find('input')
    await bankCodeInput.setValue('0001')
    const branchCodeDiv = wrapper.find('[data-test="branch-code-div"]')
    const branchCodeInput = branchCodeDiv.find('input')
    await branchCodeInput.setValue('001')
    const accountNumberDiv = wrapper.find('[data-test="account-number-div"]')
    const accountNumberInput = accountNumberDiv.find('input')
    await accountNumberInput.setValue('12345678') // ゆうちょ以外は7桁
    const accountHolderNameDiv = wrapper.find('[data-test="account-holder-name-div"]')
    const accountHolderNameInput = accountHolderNameDiv.find('input')
    await accountHolderNameInput.setValue('タナカ　タロウ')

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_BANK_ACCOUNT_NUMBER_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_BANK_ACCOUNT_NUMBER.toString())
  })

  it(`displays ${Message.REACH_PAYMENT_PLATFORM_RATE_LIMIT_MESSAGE} if ${Code.REACH_PAYMENT_PLATFORM_RATE_LIMIT} is returned`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.REACH_PAYMENT_PLATFORM_RATE_LIMIT))
    postBankAccountFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const bankCodeDiv = wrapper.find('[data-test="bank-code-div"]')
    const bankCodeInput = bankCodeDiv.find('input')
    await bankCodeInput.setValue('0001')
    const branchCodeDiv = wrapper.find('[data-test="branch-code-div"]')
    const branchCodeInput = branchCodeDiv.find('input')
    await branchCodeInput.setValue('001')
    const accountNumberDiv = wrapper.find('[data-test="account-number-div"]')
    const accountNumberInput = accountNumberDiv.find('input')
    await accountNumberInput.setValue('1234567')
    const accountHolderNameDiv = wrapper.find('[data-test="account-holder-name-div"]')
    const accountHolderNameInput = accountHolderNameDiv.find('input')
    await accountHolderNameInput.setValue('タナカ　タロウ')

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.REACH_PAYMENT_PLATFORM_RATE_LIMIT_MESSAGE)
    expect(resultMessage).toContain(Code.REACH_PAYMENT_PLATFORM_RATE_LIMIT.toString())
  })

  it(`displays ${Message.NO_IDENTITY_REGISTERED_MESSAGE} if ${Code.NO_IDENTITY_REGISTERED}) is returned`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_IDENTITY_REGISTERED))
    postBankAccountFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const bankCodeDiv = wrapper.find('[data-test="bank-code-div"]')
    const bankCodeInput = bankCodeDiv.find('input')
    await bankCodeInput.setValue('0001')
    const branchCodeDiv = wrapper.find('[data-test="branch-code-div"]')
    const branchCodeInput = branchCodeDiv.find('input')
    await branchCodeInput.setValue('001')
    const accountNumberDiv = wrapper.find('[data-test="account-number-div"]')
    const accountNumberInput = accountNumberDiv.find('input')
    await accountNumberInput.setValue('1234567')
    const accountHolderNameDiv = wrapper.find('[data-test="account-holder-name-div"]')
    const accountHolderNameInput = accountHolderNameDiv.find('input')
    await accountHolderNameInput.setValue('タナカ　タロウ')

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.NO_IDENTITY_REGISTERED_MESSAGE)
    expect(resultMessage).toContain(Code.NO_IDENTITY_REGISTERED.toString())
  })

  it(`displays ${Message.NO_CAREERS_FOUND_MESSAGE} if ${Code.NO_CAREERS_FOUND}) is returned`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_CAREERS_FOUND))
    postBankAccountFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const bankCodeDiv = wrapper.find('[data-test="bank-code-div"]')
    const bankCodeInput = bankCodeDiv.find('input')
    await bankCodeInput.setValue('0001')
    const branchCodeDiv = wrapper.find('[data-test="branch-code-div"]')
    const branchCodeInput = branchCodeDiv.find('input')
    await branchCodeInput.setValue('001')
    const accountNumberDiv = wrapper.find('[data-test="account-number-div"]')
    const accountNumberInput = accountNumberDiv.find('input')
    await accountNumberInput.setValue('1234567')
    const accountHolderNameDiv = wrapper.find('[data-test="account-holder-name-div"]')
    const accountHolderNameInput = accountHolderNameDiv.find('input')
    await accountHolderNameInput.setValue('タナカ　タロウ')

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.NO_CAREERS_FOUND_MESSAGE)
    expect(resultMessage).toContain(Code.NO_CAREERS_FOUND.toString())
  })

  it(`displays ${Message.NO_FEE_PER_HOUR_IN_YEN_FOUND_MESSAGE} if ${Code.NO_FEE_PER_HOUR_IN_YEN_FOUND}) is returned`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_FEE_PER_HOUR_IN_YEN_FOUND))
    postBankAccountFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const bankCodeDiv = wrapper.find('[data-test="bank-code-div"]')
    const bankCodeInput = bankCodeDiv.find('input')
    await bankCodeInput.setValue('0001')
    const branchCodeDiv = wrapper.find('[data-test="branch-code-div"]')
    const branchCodeInput = branchCodeDiv.find('input')
    await branchCodeInput.setValue('001')
    const accountNumberDiv = wrapper.find('[data-test="account-number-div"]')
    const accountNumberInput = accountNumberDiv.find('input')
    await accountNumberInput.setValue('1234567')
    const accountHolderNameDiv = wrapper.find('[data-test="account-holder-name-div"]')
    const accountHolderNameInput = accountHolderNameDiv.find('input')
    await accountHolderNameInput.setValue('タナカ　タロウ')

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.NO_FEE_PER_HOUR_IN_YEN_FOUND_MESSAGE)
    expect(resultMessage).toContain(Code.NO_FEE_PER_HOUR_IN_YEN_FOUND.toString())
  })

  it('displays AlertMessage when error has happened on postBankAccount', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const errDetail = 'connection error'
    postBankAccountFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const bankCodeDiv = wrapper.find('[data-test="bank-code-div"]')
    const bankCodeInput = bankCodeDiv.find('input')
    await bankCodeInput.setValue('0001')
    const branchCodeDiv = wrapper.find('[data-test="branch-code-div"]')
    const branchCodeInput = branchCodeDiv.find('input')
    await branchCodeInput.setValue('001')
    const accountNumberDiv = wrapper.find('[data-test="account-number-div"]')
    const accountNumberInput = accountNumberDiv.find('input')
    await accountNumberInput.setValue('1234567')
    const accountHolderNameDiv = wrapper.find('[data-test="account-holder-name-div"]')
    const accountHolderNameInput = accountHolderNameDiv.find('input')
    await accountHolderNameInput.setValue('タナカ　タロウ')

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.UNEXPECTED_ERR)
    expect(resultMessage).toContain(errDetail)
  })

  it(`moves to login if postBankAccount returns ${Code.UNAUTHORIZED}`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    postBankAccountFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const bankCodeDiv = wrapper.find('[data-test="bank-code-div"]')
    const bankCodeInput = bankCodeDiv.find('input')
    await bankCodeInput.setValue('0001')
    const branchCodeDiv = wrapper.find('[data-test="branch-code-div"]')
    const branchCodeInput = branchCodeDiv.find('input')
    await branchCodeInput.setValue('001')
    const accountNumberDiv = wrapper.find('[data-test="account-number-div"]')
    const accountNumberInput = accountNumberDiv.find('input')
    await accountNumberInput.setValue('1234567')
    const accountHolderNameDiv = wrapper.find('[data-test="account-holder-name-div"]')
    const accountHolderNameInput = accountHolderNameDiv.find('input')
    await accountHolderNameInput.setValue('タナカ　タロウ')

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/login')
  })

  it(`moves to terms-of-use if postBankAccount returns ${Code.NOT_TERMS_OF_USE_AGREED_YET}`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    postBankAccountFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(BankAccountPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const bankCodeDiv = wrapper.find('[data-test="bank-code-div"]')
    const bankCodeInput = bankCodeDiv.find('input')
    await bankCodeInput.setValue('0001')
    const branchCodeDiv = wrapper.find('[data-test="branch-code-div"]')
    const branchCodeInput = branchCodeDiv.find('input')
    await branchCodeInput.setValue('001')
    const accountNumberDiv = wrapper.find('[data-test="account-number-div"]')
    const accountNumberInput = accountNumberDiv.find('input')
    await accountNumberInput.setValue('1234567')
    const accountHolderNameDiv = wrapper.find('[data-test="account-holder-name-div"]')
    const accountHolderNameInput = accountHolderNameDiv.find('input')
    await accountHolderNameInput.setValue('タナカ　タロウ')

    const button = wrapper.find('[data-test="submit-button"]')
    await button.trigger('submit')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/terms-of-use')
  })
})
