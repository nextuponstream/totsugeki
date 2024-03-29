import { createRouter, createWebHistory } from 'vue-router'
import CreateBracket from '../views/CreateBracketView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'createBracket',
      component: CreateBracket,
    },
    {
      path: '/about',
      name: 'about',
      // route level code-splitting
      // this generates a separate chunk (About.[hash].js) for this route
      // which is lazy-loaded when the route is visited.
      component: () => import('../views/AboutView.vue'),
    },
    {
      path: '/bracket/register',
      name: 'bracketRegistration',
      component: () => import('../views/PlayerRegistrationView.vue'),
    },
    {
      path: '/bracket',
      name: 'bracket',
      component: () => import('../views/BracketView.vue'),
    },
    {
      path: '/404',
      name: 'notFound',
      component: () => import('../views/NotFoundView.vue'),
    },
    {
      path: '/register',
      name: 'registerUser',
      component: () => import('../views/RegistrationView.vue'),
    },
    {
      path: '/user/dashboard',
      name: 'userDashboard',
      meta: { requiresAuth: true },
      component: () => import('../views/UserDashboard.vue'),
    },
    {
      path: '/:pathMatch(.*)*',
      redirect: '/404',
    },
  ],
})

router.beforeEach((to, from, next) => {
  // TODO use pinia instead of localstorage
  let userId = localStorage.getItem('user_id')
  if (userId === null && to.meta.requiresAuth) {
    console.warn('unauthenticated, redirecting to homepage...')
    next({ name: 'createBracket' })
  } else {
    next()
  }
})

export default router
