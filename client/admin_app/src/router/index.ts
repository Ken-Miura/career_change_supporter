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
import CreateCareerRequestListPage from '../views/personalized/CreateCareerRequestListPage.vue'
import CreateCareerRequestDetailPage from '../views/personalized/CreateCareerRequestDetailPage.vue'
import CreateCareerRequestApprovalPage from '../views/personalized/CreateCareerRequestApprovalPage.vue'
import CreateCareerRequestRejectionDetailPage from '../views/personalized/CreateCareerRequestRejectionDetailPage.vue'
import CreateCareerRequestRejectionPage from '../views/personalized/CreateCareerRequestRejectionPage.vue'

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
    path: '/create-identity-request-detail/:user_account_id',
    name: 'CreateIdentityRequestDetailPage',
    component: CreateIdentityRequestDetailPage
  },
  {
    path: '/create-identity-request-approval',
    name: 'CreateIdentityRequestApprovalPage',
    component: CreateIdentityRequestApprovalPage
  },
  {
    path: '/create-identity-request-rejection-detail/:user_account_id',
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
    path: '/update-identity-request-detail/:user_account_id',
    name: 'UpdateIdentityRequestDetailPage',
    component: UpdateIdentityRequestDetailPage
  },
  {
    path: '/update-identity-request-approval',
    name: 'UpdateIdentityRequestApprovalPage',
    component: UpdateIdentityRequestApprovalPage
  },
  {
    path: '/update-identity-request-rejection-detail/:user_account_id',
    name: 'UpdateIdentityRequestRejectionDetailPage',
    component: UpdateIdentityRequestRejectionDetailPage
  },
  {
    path: '/update-identity-request-rejection',
    name: 'UpdateIdentityRequestRejectionPage',
    component: UpdateIdentityRequestRejectionPage
  },
  {
    path: '/create-career-request-list',
    name: 'CreateCareerRequestListPage',
    component: CreateCareerRequestListPage
  },
  {
    path: '/create-career-request-detail/:create_career_req_id',
    name: 'CreateCareerRequestDetailPage',
    component: CreateCareerRequestDetailPage
  },
  {
    path: '/create-career-request-approval',
    name: 'CreateCareerRequestApprovalPage',
    component: CreateCareerRequestApprovalPage
  },
  {
    path: '/create-career-request-rejection-detail/:create_career_req_id',
    name: 'CreateCareerRequestRejectionDetailPage',
    component: CreateCareerRequestRejectionDetailPage
  },
  {
    path: '/create-career-request-rejection',
    name: 'CreateCareerRequestRejectionPage',
    component: CreateCareerRequestRejectionPage
  }
]

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes
})

export default router
