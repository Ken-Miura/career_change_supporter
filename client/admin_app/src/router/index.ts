import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'
import LandingPage from '../views/LandingPage.vue'
import LoginPage from '../views/LoginPage.vue'
import AdminMenuPage from '../views/personalized/AdminMenuPage.vue'
import CreateIdentityRequestListPage from '../views/personalized/CreateIdentityRequestListPage.vue'
import CreateIdentityRequestDetailPage from '../views/personalized/CreateIdentityRequestDetailPage.vue'
import CreateIdentityRequestApprovalPage from '../views/personalized/CreateIdentityRequestApprovalPage.vue'
import CreateIdentityRequestRejectionDetailPage from '../views/personalized/CreateIdentityRequestRejectionDetailPage.vue'
import CreateIdentityRequestRejectionPage from '../views/personalized/CreateIdentityRequestRejectionPage.vue'
import UpdateIdentityRequestListPage from '../views/personalized/UpdateIdentityRequestListPage.vue'
import UpdateIdentityRequestDetailPage from '../views/personalized/UpdateIdentityRequestDetailPage.vue'
import UpdateIdentityRequestApprovalPage from '../views/personalized/UpdateIdentityRequestApprovalPage.vue'
import UpdateIdentityRequestRejectionDetailPage from '../views/personalized/UpdateIdentityRequestRejectionDetailPage.vue'
import UpdateIdentityRequestRejectionPage from '../views/personalized/UpdateIdentityRequestRejectionPage.vue'

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
  },
  {
    path: '/create-identity-request-approval',
    name: 'CreateIdentityRequestApprovalPage',
    component: CreateIdentityRequestApprovalPage
  },
  {
    path: '/create-identity-request-rejection-detail/:account_id',
    name: 'CreateIdentityRequestRejectionDetailPage',
    component: CreateIdentityRequestRejectionDetailPage
  },
  {
    path: '/create-identity-request-rejection',
    name: 'CreateIdentityRequestRejectionPage',
    component: CreateIdentityRequestRejectionPage
  },
  {
    path: '/update-identity-request-list',
    name: 'UpdateIdentityRequestListPage',
    component: UpdateIdentityRequestListPage
  },
  {
    path: '/update-identity-request-detail/:account_id',
    name: 'UpdateIdentityRequestDetailPage',
    component: UpdateIdentityRequestDetailPage
  },
  {
    path: '/update-identity-request-approval',
    name: 'UpdateIdentityRequestApprovalPage',
    component: UpdateIdentityRequestApprovalPage
  },
  {
    path: '/update-identity-request-rejection-detail/:account_id',
    name: 'UpdateIdentityRequestRejectionDetailPage',
    component: UpdateIdentityRequestRejectionDetailPage
  },
  {
    path: '/update-identity-request-rejection',
    name: 'UpdateIdentityRequestRejectionPage',
    component: UpdateIdentityRequestRejectionPage
  }
]

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes
})

export default router
