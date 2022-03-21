import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'
import LandingPage from '../views/LandingPage.vue'
import LoginPage from '../views/LoginPage.vue'
import AdminMenuPage from '../views/personalized/AdminMenuPage.vue'
import CreateIdentityRequestListPage from '../views/personalized/CreateIdentityRequestListPage.vue'
import CreateIdentityRequestDetailPage from '../views/personalized/CreateIdentityRequestDetailPage.vue'

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
    path: '/admin-menu',
    name: 'AdminMenuPage',
    component: AdminMenuPage
  },
  {
    path: '/create-identity-request-list',
    name: 'CreateIdentityRequestListPage',
    component: CreateIdentityRequestListPage
  },
  {
    path: '/create-identity-request-detail/:account_id',
    name: 'CreateIdentityRequestDetailPage',
    component: CreateIdentityRequestDetailPage
  }
]

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes
})

export default router
