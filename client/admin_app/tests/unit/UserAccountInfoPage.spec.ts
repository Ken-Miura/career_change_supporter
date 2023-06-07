import { ref } from 'vue'

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

const postUserAccountRetrievalDoneMock = ref(true)
const postUserAccountRetrievalByUserAccountIdFuncMock = jest.fn()
const postUserAccountRetrievalByEmailAddressFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/usePostUserAccountRetrieval', () => ({
  usePostUserAccountRetrieval: () => ({
    postUserAccountRetrievalDone: postUserAccountRetrievalDoneMock,
    postUserAccountRetrievalByUserAccountIdFunc: postUserAccountRetrievalByUserAccountIdFuncMock,
    postUserAccountRetrievalByEmailAddressFunc: postUserAccountRetrievalByEmailAddressFuncMock
  })
}))

const getAgreementsByUserAccountIdDoneMock = ref(true)
const getAgreementsByUserAccountIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/terms-of-use/useGetAgreementsByUserAccountId', () => ({
  useGetAgreementsByUserAccountId: () => ({
    getAgreementsByUserAccountIdDone: getAgreementsByUserAccountIdDoneMock,
    getAgreementsByUserAccountIdFunc: getAgreementsByUserAccountIdFuncMock
  })
}))

describe('UserAccountInfoPage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    postUserAccountRetrievalDoneMock.value = true
    postUserAccountRetrievalByUserAccountIdFuncMock.mockReset()
    postUserAccountRetrievalByEmailAddressFuncMock.mockReset()
    getAgreementsByUserAccountIdDoneMock.value = true
    getAgreementsByUserAccountIdFuncMock.mockReset()
  })

  it('tests', () => {
    console.log('test')
  })
})
