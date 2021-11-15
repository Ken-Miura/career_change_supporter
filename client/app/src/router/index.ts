import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'
import Landing from '../views/Landing.vue'
import Login from '../views/Login.vue'
import Profile from '../views/personalized/Profile.vue'

const routes: Array<RouteRecordRaw> = [
  {
    path: '/index.html',
    name: 'RedirectToLanding',
    redirect: '/'
  },
  {
    path: '/',
    name: 'Landing',
    component: Landing
  },
  {
    path: '/login',
    name: 'Login',
    component: Login
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
    path: '/accounts',
    name: 'AccountCreated',
    // 新規登録は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/AccountCreated.vue')
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
