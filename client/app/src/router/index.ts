import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'
import Landing from '../views/Landing.vue'
import Login from '../views/Login.vue'

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
    // 新規作成は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/NewAccount.vue')
  },
  {
    path: '/temp-account-created',
    name: 'TempAccountCreated',
    props: true,
    // 新規作成は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/TempAccountCreated.vue')
  },
  {
    path: '/password-change',
    name: 'PasswordChange',
    // パスワード変更は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/PasswordChange.vue')
  },
  {
    path: '/accounts',
    name: 'AccountCreated',
    // 新規作成は頻繁に起こらないと思われるため、lazy loading
    component: () => import('../views/AccountCreated.vue')
  }
]

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes
})

export default router
