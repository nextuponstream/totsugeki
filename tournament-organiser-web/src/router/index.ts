import { createRouter, createWebHistory } from 'vue-router'
import CreateBracket from '../views/CreateBracketView.vue'
import { useUserStore } from '@/stores/user'

export const RouteNames = {
  home: 'home',
  logout: 'logout',
  user: {
    register: 'registerUser',
    dashboard: 'userDashboard',
    brackets: 'userBrackets',
  },
  bracket: {
    create: 'createBracket',
    show: 'bracket',
    registration: 'bracketRegistration',
    guest: 'bracket-guest',
  },
  about: 'about',
  notFound: 'notFound',
}

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: RouteNames.home,
      component: CreateBracket,
    },
    {
      path: '/',
      name: RouteNames.bracket.create,
      component: CreateBracket,
    },
    {
      path: '/about',
      name: RouteNames.about,
      // route level code-splitting
      // this generates a separate chunk (About.[hash].js) for this route
      // which is lazy-loaded when the route is visited.
      component: () => import('../views/AboutView.vue'),
    },
    {
      path: '/brackets/register',
      name: RouteNames.bracket.registration,
      component: () => import('../views/PlayerRegistrationView.vue'),
    },
    {
      path: '/brackets/:bracketId',
      name: RouteNames.bracket.show,
      component: () => import('../views/BracketView.vue'),
    },
    {
      path: '/brackets/guest',
      name: RouteNames.bracket.guest,
      component: () => import('../views/BracketView.vue'),
    },
    {
      path: '/404',
      name: RouteNames.notFound,
      component: () => import('../views/NotFoundView.vue'),
    },
    {
      path: '/register',
      name: RouteNames.user.register,
      component: () => import('../views/users/UserRegistrationView.vue'),
    },
    {
      path: '/user',
      meta: { requiresAuth: true },
      children: [
        {
          path: '/logout',
          name: RouteNames.logout,
          component: () => import('../views/users/UserLogoutView.vue'),
        },
        {
          path: 'dashboard',
          name: RouteNames.user.dashboard,
          component: () => import('../views/users/UserDashboardView.vue'),
        },
        {
          path: 'brackets',
          name: RouteNames.user.brackets,
          component: () => import('../views/users/UserBracketsView.vue'),
        },
      ],
    },
    {
      path: '/:pathMatch(.*)*',
      redirect: '/404',
    },
  ],
})

router.beforeEach((to, from, next) => {
  const userStore = useUserStore()

  if (userStore.id === null && to.meta.requiresAuth) {
    console.warn('unauthenticated, redirecting to homepage...')
    next({ name: 'createBracket' })
  } else {
    next()
  }
})

export default router
