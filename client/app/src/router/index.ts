import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'
import LandingPage from '../views/LandingPage.vue'
import InformationPage from '../views/InformationPage.vue'
import LoginPage from '../views/LoginPage.vue'
import ProfilePage from '../views/personalized/ProfilePage.vue'
import IdentityPage from '../views/personalized/IdentityPage.vue'
import AddCareerPage from '../views/personalized/AddCareerPage.vue'
import EditCareerPage from '../views/personalized/EditCareerPage.vue'
import FeePerHourInYenPage from '../views/personalized/FeePerHourInYenPage.vue'
import DeleteAccountConfirmationPage from '../views/personalized/DeleteAccountConfirmationPage.vue'
import RewardPage from '../views/personalized/RewardPage.vue'
import BankAccountPage from '../views/personalized/BankAccountPage.vue'
import SchedulePage from '../views/personalized/SchedulePage.vue'
import AcceptConsultionPage from '../views/personalized/AcceptConsultionPage.vue'
import RequestConsultationPage from '../views/personalized/RequestConsultationPage.vue'

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
    path: '/information',
    name: 'InformationPage',
    component: InformationPage
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
    path: '/careers/:career_id',
    name: 'EditCareerPage',
    component: EditCareerPage
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
    path: '/schedule',
    name: 'SchedulePage',
    component: SchedulePage
  },
  {
    path: '/request-consultation',
    name: 'RequestConsultationPage',
    component: RequestConsultationPage
  },
  {
    path: '/accept-consultation',
    name: 'AcceptConsultionPage',
    component: AcceptConsultionPage
  }
]

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes
})

export default router
