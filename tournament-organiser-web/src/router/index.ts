import { createRouter, createWebHistory } from 'vue-router'
import HomePage from '../views/CreateTournamentView.vue'
import { useUserStore } from '@/stores/user'

export const RouteNames = {
  home: 'home',
  logout: 'logout',
  users: {
    register: 'registerUser',
    dashboard: 'userDashboard',
    tournaments: 'userTournament',
  },
  tournaments: {
    create: 'tournamentCreate',
    show: 'tournamentShow',
    guest: 'tournament-guest',
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
      component: HomePage,
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
      path: '/tournaments/create',
      name: RouteNames.tournaments.create,
      component: () => import('../views/PlayerRegistrationView.vue'),
    },
    {
      path: '/tournaments/:tournamentId',
      name: RouteNames.tournaments.show,
      props: { isGuest: false },
      component: () => import('../views/TournamentView.vue'),
    },
    {
      path: '/tournaments/guest',
      name: RouteNames.tournaments.guest,
      props: { isGuest: true },
      component: () => import('../views/TournamentView.vue'),
    },
    {
      path: '/404',
      name: RouteNames.notFound,
      component: () => import('../views/NotFoundView.vue'),
    },
    {
      path: '/register',
      name: RouteNames.users.register,
      component: () => import('../views/users/UserRegistrationView.vue'),
    },
    {
      path: '/users',
      meta: { requiresAuth: true },
      children: [
        {
          path: '/logout',
          name: RouteNames.logout,
          component: () => import('../views/users/UserLogoutView.vue'),
        },
        {
          path: 'dashboard',
          name: RouteNames.users.dashboard,
          component: () => import('../views/users/UserDashboardView.vue'),
        },
        {
          path: 'tournaments',
          name: RouteNames.users.tournaments,
          component: () => import('../views/users/UserTournamentView.vue'),
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
    next({ name: RouteNames.home })
  } else {
    next()
  }
})

export default router
