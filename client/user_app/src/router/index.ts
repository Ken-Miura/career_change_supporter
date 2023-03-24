import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'
import LandingPage from '../views/LandingPage.vue'
import NewsPage from '../views/NewsPage.vue'
import LoginPage from '../views/LoginPage.vue'
import ProfilePage from '../views/personalized/ProfilePage.vue'
import IdentityPage from '../views/personalized/IdentityPage.vue'
import SubmitIdentitySuccessPage from '../views/personalized/SubmitIdentitySuccessPage.vue'
import AddCareerPage from '../views/personalized/AddCareerPage.vue'
import CareerDetailPage from '../views/personalized/CareerDetailPage.vue'
import CareerDeletionConfirmPage from '../views/personalized/CareerDeletionConfirmPage.vue'
import DeleteCareerSuccessPage from '../views/personalized/DeleteCareerSuccessPage.vue'
import FeePerHourInYenPage from '../views/personalized/FeePerHourInYenPage.vue'
import SubmitFeePerHourInYenSuccessPage from '../views/personalized/SubmitFeePerHourInYenSuccessPage.vue'
import DeleteAccountConfirmationPage from '../views/personalized/DeleteAccountConfirmationPage.vue'
import RewardPage from '../views/personalized/RewardPage.vue'
import BankAccountPage from '../views/personalized/BankAccountPage.vue'
import SubmitBankAccountSuccessPage from '../views/personalized/SubmitBankAccountSuccessPage.vue'
import SchedulePage from '../views/personalized/SchedulePage.vue'
import ConsultationRequestListPage from '../views/personalized/ConsultationRequestListPage.vue'
import ConsultantsSearchPage from '../views/personalized/ConsultantsSearchPage.vue'
import SubmitCareerSuccessPage from '../views/personalized/SubmitCareerSuccessPage.vue'
import OssLicensePage from '../views/OssLicensePage.vue'
import PrivacyPolicyPage from '../views/PrivacyPolicyPage.vue'
import TransactionLawPage from '../views/TransactionLawPage.vue'
import ConsultantListPage from '../views/personalized/ConsultantListPage.vue'
import ConsultantDetailPage from '../views/personalized/ConsultantDetailPage.vue'
import RequestConsultationPage from '../views/personalized/RequestConsultationPage.vue'
import UnratedItemListPage from '../views/personalized/UnratedItemListPage.vue'
import AudioTestPage from '../views/personalized/AudioTestPage.vue'
import RequestConsultationSuccessPage from '../views/personalized/RequestConsultationSuccessPage.vue'
import ConsultationRequestDetailPage from '../views/personalized/ConsultationRequestDetailPage.vue'
import ConsultationRequestRejectionPage from '../views/personalized/ConsultationRequestRejectionPage.vue'
import ConsultationRequestAcceptancePage from '../views/personalized/ConsultationRequestAcceptancePage.vue'
import UserSideConsultationPage from '../views/personalized/UserSideConsultationPage.vue'
import ConsultantSideConsultationPage from '../views/personalized/ConsultantSideConsultationPage.vue'
import RateUserPage from '../views/personalized/RateUserPage.vue'
import RateConsultantPage from '../views/personalized/RateConsultantPage.vue'
import RateSuccessPage from '../views/personalized/RateSuccessPage.vue'
import MfaSettingPage from '../views/personalized/MfaSettingPage.vue'
import EnableMfaConfirmationPage from '../views/personalized/EnableMfaConfirmationPage.vue'

const routes: Array<RouteRecordRaw> = [
  {
    path: '/index.html',
    name: 'RedirectToLandingPage',
    redirect: '/'
  },
  {
    path: '/',
    name: 'LandingPage',
    component: LandingPage
  },
  {
    path: '/login',
    name: 'LoginPage',
    component: LoginPage
  },
  {
    path: '/new-account',
    name: 'NewAccountPage',
    // 新規登録は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/NewAccountPage.vue')
  },
  {
    path: '/temp-account-creation-result',
    name: 'TempAccountCreationResultPage',
    // 新規登録は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/TempAccountCreationResultPage.vue')
  },
  {
    path: '/account-creation',
    name: 'AccountCreationPage',
    // 新規登録は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/AccountCreationPage.vue')
  },
  {
    path: '/password-change-req',
    name: 'PasswordChangeReqPage',
    // パスワード変更は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/PasswordChangeReqPage.vue')
  },
  {
    path: '/password-change-req-result',
    name: 'PasswordChangeReqResultPage',
    // パスワード変更は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/PasswordChangeReqResultPage.vue')
  },
  {
    path: '/password-update',
    name: 'PasswordUpdatePage',
    // パスワード変更は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/PasswordUpdatePage.vue')
  },
  {
    path: '/password-update-result',
    name: 'PasswordUpdateResultPage',
    // パスワード変更は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/PasswordUpdateResultPage.vue')
  },
  {
    path: '/terms-of-use',
    name: 'TermsOfUsePage',
    // 利用規約の同意は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/personalized/TermsOfUsePage.vue')
  },
  {
    path: '/news',
    name: 'NewsPage',
    component: NewsPage
  },
  {
    path: '/published-terms-of-use',
    name: 'PublishedTermsOfUsePage',
    // 利用規約の確認は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/PublishedTermsOfUsePage.vue')
  },
  {
    path: '/profile',
    name: 'ProfilePage',
    component: ProfilePage
  },
  {
    path: '/identity',
    name: 'IdentityPage',
    component: IdentityPage
  },
  {
    path: '/submit-identity-success',
    name: 'SubmitIdentitySuccessPage',
    component: SubmitIdentitySuccessPage
  },
  {
    path: '/careers/:career_id',
    name: 'CareerDetailPage',
    component: CareerDetailPage
  },
  {
    path: '/career-deletion-confirm/:career_id',
    name: 'CareerDeletionConfirmPage',
    component: CareerDeletionConfirmPage
  },
  {
    path: '/delete-career-success',
    name: 'DeleteCareerSuccessPage',
    component: DeleteCareerSuccessPage
  },
  {
    path: '/careers',
    name: 'AddCareerPage',
    component: AddCareerPage
  },
  {
    path: '/fee-per-hour-in-yen',
    name: 'FeePerHourInYenPage',
    component: FeePerHourInYenPage
  },
  {
    path: '/mfa-setting',
    name: 'MfaSettingPage',
    component: MfaSettingPage
  },
  {
    path: '/enable-mfa-confirmation',
    name: 'EnableMfaConfirmationPage',
    component: EnableMfaConfirmationPage
  },
  {
    path: '/submit-fee-per-hour-in-yen-success',
    name: 'SubmitFeePerHourInYenSuccessPage',
    component: SubmitFeePerHourInYenSuccessPage
  },
  {
    path: '/delete-account-confirmation',
    name: 'DeleteAccountConfirmationPage',
    component: DeleteAccountConfirmationPage
  },
  {
    path: '/reward',
    name: 'RewardPage',
    component: RewardPage
  },
  {
    path: '/bank-account',
    name: 'BankAccountPage',
    component: BankAccountPage
  },
  {
    path: '/submit-bank-account-success',
    name: 'SubmitBankAccountSuccessPage',
    component: SubmitBankAccountSuccessPage
  },
  {
    path: '/schedule',
    name: 'SchedulePage',
    component: SchedulePage
  },
  {
    path: '/user-side-consultation/:consultation_id',
    children: [
      {
        path: 'consultant/:consultant_id',
        name: 'UserSideConsultationPage',
        component: UserSideConsultationPage
      }
    ]
  },
  {
    path: '/consultant-side-consultation/:consultation_id',
    children: [
      {
        path: 'user/:user_account_id',
        name: 'ConsultantSideConsultationPage',
        component: ConsultantSideConsultationPage
      }
    ]
  },
  {
    path: '/consultants-search',
    name: 'ConsultantsSearchPage',
    component: ConsultantsSearchPage
  },
  {
    path: '/consultation-request-list',
    name: 'ConsultationRequestListPage',
    component: ConsultationRequestListPage
  },
  {
    path: '/consultation-request-detail/:consultation_req_id',
    name: 'ConsultationRequestDetailPage',
    component: ConsultationRequestDetailPage
  },
  {
    path: '/consultation-request-rejection',
    name: 'ConsultationRequestRejectionPage',
    component: ConsultationRequestRejectionPage
  },
  {
    path: '/consultation-request-acceptance',
    name: 'ConsultationRequestAcceptancePage',
    component: ConsultationRequestAcceptancePage
  },
  {
    path: '/submit-career-success',
    name: 'SubmitCareerSuccessPage',
    component: SubmitCareerSuccessPage
  },
  {
    path: '/oss-license',
    name: 'OssLicensePage',
    component: OssLicensePage
  },
  {
    path: '/privacy-policy',
    name: 'PrivacyPolicyPage',
    component: PrivacyPolicyPage
  },
  {
    path: '/transaction-law',
    name: 'TransactionLawPage',
    component: TransactionLawPage
  },
  {
    path: '/consultant-list',
    name: 'ConsultantListPage',
    component: ConsultantListPage
  },
  {
    path: '/consultant-detail/:consultant_id',
    name: 'ConsultantDetailPage',
    component: ConsultantDetailPage
  },
  {
    path: '/request-consultation/:consultant_id',
    name: 'RequestConsultationPage',
    component: RequestConsultationPage
  },
  {
    path: '/request-consultation-success',
    name: 'RequestConsultationSuccessPage',
    component: RequestConsultationSuccessPage
  },
  {
    path: '/audio-test',
    name: 'AudioTestPage',
    component: AudioTestPage
  },
  {
    path: '/unrated-item-list',
    name: 'UnratedItemListPage',
    component: UnratedItemListPage
  },
  {
    path: '/rate-consultant/:consultant_rating_id',
    name: 'RateConsultantPage',
    component: RateConsultantPage
  },
  {
    path: '/rate-user/:user_rating_id',
    name: 'RateUserPage',
    component: RateUserPage
  },
  {
    path: '/rate-success',
    name: 'RateSuccessPage',
    component: RateSuccessPage
  }
]

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes
})

export default router
