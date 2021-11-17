import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'
import LandingPage from '../views/LandingPage.vue'
import LoginPage from '../views/LoginPage.vue'
import Profile from '../views/personalized/Profile.vue'

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
    name: 'NewAccount',
    // 新規登録は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/NewAccount.vue')
  },
  {
    path: '/temp-account-created',
    name: 'TempAccountCreated',
    props: true,
    // 新規登録は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/TempAccountCreated.vue')
  },
  {
    path: '/accounts',
    name: 'AccountCreated',
    // 新規登録は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/AccountCreated.vue')
  },
  {
    path: '/password-change',
    name: 'PasswordChange',
    // パスワード変更は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/PasswordChange.vue')
  },
  {
    path: '/new-password-created',
    name: 'NewPasswordCreated',
    props: true,
    // パスワード変更は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/NewPasswordCreated.vue')
  },
  {
    path: '/new-password',
    name: 'NewPassword',
    // パスワード変更は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/NewPassword.vue')
  },
  {
    path: '/new-password-applied',
    name: 'NewPasswordApplied',
    props: true,
    // パスワード変更は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/NewPasswordApplied.vue')
  },
  {
    path: '/profile',
    name: 'Profile',
    component: Profile
  },
  {
    path: '/terms-of-use',
    name: 'TermsOfUseAgreement',
    // 利用規約の同意は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/personalized/TermsOfUseAgreement.vue')
  }
]

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes
})

export default router
