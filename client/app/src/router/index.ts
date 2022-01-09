import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'
import LandingPage from '../views/LandingPage.vue'
import LoginPage from '../views/LoginPage.vue'
import ProfilePage from '../views/personalized/ProfilePage.vue'
import IdentityPage from '../views/personalized/IdentityPage.vue'
import RewardPage from '../views/personalized/RewardPage.vue'
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
    path: '/password-change',
    name: 'PasswordChangePage',
    // パスワード変更は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/PasswordChangePage.vue')
  },
  {
    path: '/new-password-creation-result',
    name: 'NewPasswordCreationResultPage',
    // パスワード変更は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/NewPasswordCreationResultPage.vue')
  },
  {
    path: '/password-change-confirmation',
    name: 'PasswordChangeConfirmationPage',
    // パスワード変更は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/PasswordChangeConfirmationPage.vue')
  },
  {
    path: '/apply-new-password-result',
    name: 'ApplyNewPasswordResultPage',
    // パスワード変更は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/ApplyNewPasswordResultPage.vue')
  },
  {
    path: '/terms-of-use',
    name: 'TermsOfUsePage',
    // 利用規約の同意は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/personalized/TermsOfUsePage.vue')
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
    path: '/reward',
    name: 'RewardPage',
    component: RewardPage
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
