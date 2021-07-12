import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'
import Home from '../views/Home.vue'

const routes: Array<RouteRecordRaw> = [
  {
    path: '/advisor_app.html',
    name: 'advisor_app',
    redirect: '/home'
  },
  {
    path: '/home',
    name: 'Home',
    component: Home
  },
  {
    path: '/login',
    name: 'Login',
    component: () => import('../views/Login.vue')
  },
  {
    path: '/register',
    name: 'Register',
    component: () => import('../views/Register.vue')
  },
  {
    path: '/schedule',
    name: 'Schedule',
    component: () => import('../views/Schedule.vue')
  },
  {
    path: '/profile',
    name: 'Profile',
    component: () => import('../views/Profile.vue')
  },
  {
    path: '/registration-requests',
    name: 'RegistrationRequests',
    component: () => import('../views/RegistrationRequests.vue')
  },
  {
    path: '/edit-bank-info',
    name: 'EditBankInfo',
    component: () => import('../views/EditBankInfo.vue')
  },
  {
    path: '/edit-advice-fee',
    name: 'EditAdviceFee',
    component: () => import('../views/EditAdviceFee.vue')
  }
]

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes
})

export default router
