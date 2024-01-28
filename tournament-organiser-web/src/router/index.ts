import { createRouter, createWebHistory } from 'vue-router'
import CreateBracket from '../views/CreateBracketView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'create-bracket',
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
      path: '/:pathMatch(.*)*',
      redirect: '/404',
    },
  ],
})

export default router
