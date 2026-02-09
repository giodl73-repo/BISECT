import { createRouter, createWebHistory } from 'vue-router'

// Lazy-load pages for better performance
const Home = () => import('./pages/Home.vue')
const Chapter1 = () => import('./pages/Chapter1_TractsToGraphs.vue')
const Chapter2 = () => import('./pages/Chapter2_Splitting.vue')
const Chapter3 = () => import('./pages/Chapter3_Recursion.vue')
const Chapter4 = () => import('./pages/Chapter4_Compactness.vue')
const Chapter5 = () => import('./pages/Chapter5_VRA.vue')
const Chapter6 = () => import('./pages/Chapter6_EdgeFactor.vue')
const Research = () => import('./pages/Research.vue')

const routes = [
  {
    path: '/',
    name: 'Home',
    component: Home,
    meta: {
      title: 'Algorithmic Redistricting: A Visual Journey',
      description: 'Learn about fair redistricting through interactive visualizations',
    },
  },
  {
    path: '/chapter-1',
    name: 'Chapter1',
    component: Chapter1,
    meta: {
      title: 'Chapter 1: From Tracts to Graphs',
      description: 'How census tracts become connected networks',
      chapter: 1,
      color: 'blue',
    },
  },
  {
    path: '/chapter-2',
    name: 'Chapter2',
    component: Chapter2,
    meta: {
      title: 'Chapter 2: Splitting in Two',
      description: 'Dividing regions into balanced parts',
      chapter: 2,
      color: 'orange',
    },
  },
  {
    path: '/chapter-3',
    name: 'Chapter3',
    component: Chapter3,
    meta: {
      title: 'Chapter 3: The Recursive Magic',
      description: 'Creating any number of districts through recursion',
      chapter: 3,
      color: 'green',
    },
  },
  {
    path: '/chapter-4',
    name: 'Chapter4',
    component: Chapter4,
    meta: {
      title: 'Chapter 4: Making it Compact',
      description: 'Geographic sensibility through edge-weighting',
      chapter: 4,
      color: 'purple',
    },
  },
  {
    path: '/chapter-5',
    name: 'Chapter5',
    component: Chapter5,
    meta: {
      title: 'Chapter 5: The Voting Rights Act',
      description: 'Ensuring minority representation',
      chapter: 5,
      color: 'red',
    },
  },
  {
    path: '/chapter-6',
    name: 'Chapter6',
    component: Chapter6,
    meta: {
      title: 'Chapter 6: The Edge-Factor Solution',
      description: 'Balancing compactness with representation',
      chapter: 6,
      color: 'yellow',
    },
  },
  {
    path: '/research',
    name: 'Research',
    component: Research,
    meta: {
      title: 'Research Papers',
      description: 'Explore the detailed research behind these concepts',
    },
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/',
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
  scrollBehavior(to, from, savedPosition) {
    if (savedPosition) {
      return savedPosition
    } else if (to.hash) {
      return {
        el: to.hash,
        behavior: 'smooth',
      }
    } else {
      return { top: 0, behavior: 'smooth' }
    }
  },
})

// Update document title on route change
router.beforeEach((to, from, next) => {
  document.title = to.meta.title || 'Algorithmic Redistricting'
  next()
})

export default router
