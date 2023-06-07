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

const getIdentityOptionByUserAccountIdDoneMock = ref(true)
const getIdentityOptionByUserAccountIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/identity/useGetIdentityOptionByUserAccountId', () => ({
  useGetIdentityOptionByUserAccountId: () => ({
    getIdentityOptionByUserAccountIdDone: getIdentityOptionByUserAccountIdDoneMock,
    getIdentityOptionByUserAccountIdFunc: getIdentityOptionByUserAccountIdFuncMock
  })
}))

const getCareersByUserAccountIdDoneMock = ref(true)
const getCareersByUserAccountIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/career/useGetCareersByUserAccountId', () => ({
  useGetCareersByUserAccountId: () => ({
    getCareersByUserAccountIdDone: getCareersByUserAccountIdDoneMock,
    getCareersByUserAccountIdFunc: getCareersByUserAccountIdFuncMock
  })
}))

const getFeePerHourInYenByUserAccountIdDoneMock = ref(true)
const getFeePerHourInYenByUserAccountIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/fee-per-hour-in-yen/useGetFeePerHourInYenByUserAccountId', () => ({
  useGetFeePerHourInYenByUserAccountId: () => ({
    getFeePerHourInYenByUserAccountIdDone: getFeePerHourInYenByUserAccountIdDoneMock,
    getFeePerHourInYenByUserAccountIdFunc: getFeePerHourInYenByUserAccountIdFuncMock
  })
}))

const getTenantIdByUserAccountIdDoneMock = ref(true)
const getTenantIdByUserAccountIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/tenant/useGetTenantIdByUserAccountId', () => ({
  useGetTenantIdByUserAccountId: () => ({
    getTenantIdByUserAccountIdDone: getTenantIdByUserAccountIdDoneMock,
    getTenantIdByUserAccountIdFunc: getTenantIdByUserAccountIdFuncMock
  })
}))

const getConsultationReqsByUserAccountIdDoneMock = ref(true)
const getConsultationReqsByUserAccountIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/consultation-req/useGetConsultationReqsByUserAccountId', () => ({
  useGetConsultationReqsByUserAccountId: () => ({
    getConsultationReqsByUserAccountIdDone: getConsultationReqsByUserAccountIdDoneMock,
    getConsultationReqsByUserAccountIdFunc: getConsultationReqsByUserAccountIdFuncMock
  })
}))

const getConsultationReqsByConsultantIdDoneMock = ref(true)
const getConsultationReqsByConsultantIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/consultation-req/useGetConsultationReqsByConsultantId', () => ({
  useGetConsultationReqsByConsultantId: () => ({
    getConsultationReqsByConsultantIdDone: getConsultationReqsByConsultantIdDoneMock,
    getConsultationReqsByConsultantIdFunc: getConsultationReqsByConsultantIdFuncMock
  })
}))

const getConsultationsByUserAccountIdDoneMock = ref(true)
const getConsultationsByUserAccountIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/consultation/useGetConsultationsByUserAccountId', () => ({
  useGetConsultationsByUserAccountId: () => ({
    getConsultationsByUserAccountIdDone: getConsultationsByUserAccountIdDoneMock,
    getConsultationsByUserAccountIdFunc: getConsultationsByUserAccountIdFuncMock
  })
}))

const getConsultationsByConsultantIdDoneMock = ref(true)
const getConsultationsByConsultantIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/consultation/useGetConsultationsByConsultantId', () => ({
  useGetConsultationsByConsultantId: () => ({
    getConsultationsByConsultantIdDone: getConsultationsByConsultantIdDoneMock,
    getConsultationsByConsultantIdFunc: getConsultationsByConsultantIdFuncMock
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
    getIdentityOptionByUserAccountIdDoneMock.value = true
    getIdentityOptionByUserAccountIdFuncMock.mockReset()
    getCareersByUserAccountIdDoneMock.value = true
    getCareersByUserAccountIdFuncMock.mockReset()
    getFeePerHourInYenByUserAccountIdDoneMock.value = true
    getFeePerHourInYenByUserAccountIdFuncMock.mockReset()
    getTenantIdByUserAccountIdDoneMock.value = true
    getTenantIdByUserAccountIdFuncMock.mockReset()
    getConsultationReqsByUserAccountIdDoneMock.value = true
    getConsultationReqsByUserAccountIdFuncMock.mockReset()
    getConsultationReqsByConsultantIdDoneMock.value = true
    getConsultationReqsByConsultantIdFuncMock.mockReset()
    getConsultationsByUserAccountIdDoneMock.value = true
    getConsultationsByUserAccountIdFuncMock.mockReset()
    getConsultationsByConsultantIdDoneMock.value = true
    getConsultationsByConsultantIdFuncMock.mockReset()
  })

  it('tests', () => {
    console.log('test')
  })
})
