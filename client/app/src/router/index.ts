import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'
import Landing from '../views/Landing.vue'

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
    // route level code-splitting
    // this generates a separate chunk (about.[hash].js) for this route
    // which is lazy-loaded when the route is visited.
    component: () => import('../views/Login.vue')
  },
  {
    path: '/new-account',
    name: 'NewAccount',
    component: () => import('../views/NewAccount.vue')
  }
]

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes
})

export default router
