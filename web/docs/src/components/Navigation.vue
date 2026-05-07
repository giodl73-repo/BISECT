<template>
  <nav class="nav-sticky no-print">
    <div class="container-custom py-4">
      <div class="flex items-center justify-between">
        <!-- Logo/Title -->
        <router-link to="/" class="flex items-center space-x-3">
          <div class="w-10 h-10 bg-gradient-schoolhouse rounded-full flex items-center justify-center">
            <span class="text-white font-bold text-xl">AR</span>
          </div>
          <span class="font-heading font-bold text-xl hidden md:block">
            Algorithmic Redistricting
          </span>
        </router-link>

        <!-- Desktop Navigation -->
        <div class="hidden lg:flex items-center space-x-6">
          <router-link
            v-for="chapter in chapters"
            :key="chapter.path"
            :to="chapter.path"
            class="nav-link"
            :class="`nav-link-${chapter.color}`"
          >
            {{ chapter.label }}
          </router-link>
          <router-link to="/research" class="btn btn-outline btn-sm">
            Research
          </router-link>
        </div>

        <!-- Mobile Menu Button -->
        <button
          @click="mobileMenuOpen = !mobileMenuOpen"
          class="lg:hidden p-2 rounded-lg hover:bg-gray-100 transition-colors"
          aria-label="Toggle menu"
        >
          <svg
            class="w-6 h-6"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              v-if="!mobileMenuOpen"
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M4 6h16M4 12h16M4 18h16"
            />
            <path
              v-else
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      </div>
    </div>

    <!-- Progress Bar -->
    <div
      v-if="currentChapter"
      class="nav-progress"
      :style="{ width: `${progress}%` }"
    ></div>

    <!-- Mobile Menu -->
    <transition name="mobile-menu">
      <div
        v-if="mobileMenuOpen"
        class="lg:hidden bg-white border-t border-gray-200"
      >
        <div class="container-custom py-4 space-y-2">
          <router-link
            v-for="chapter in chapters"
            :key="chapter.path"
            :to="chapter.path"
            class="block px-4 py-2 rounded-lg hover:bg-gray-100 transition-colors"
            @click="mobileMenuOpen = false"
          >
            {{ chapter.label }}
          </router-link>
          <router-link
            to="/research"
            class="block px-4 py-2 rounded-lg hover:bg-gray-100 transition-colors font-semibold"
            @click="mobileMenuOpen = false"
          >
            Research Papers
          </router-link>
        </div>
      </div>
    </transition>
  </nav>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const mobileMenuOpen = ref(false)

const chapters = [
  { path: '/chapter-1', label: 'Ch 1', color: 'blue' },
  { path: '/chapter-2', label: 'Ch 2', color: 'orange' },
  { path: '/chapter-3', label: 'Ch 3', color: 'green' },
  { path: '/chapter-4', label: 'Ch 4', color: 'purple' },
  { path: '/chapter-5', label: 'Ch 5', color: 'red' },
  { path: '/chapter-6', label: 'Ch 6', color: 'yellow' },
]

const currentChapter = computed(() => {
  const chapterMatch = route.path.match(/\/chapter-(\d+)/)
  return chapterMatch ? parseInt(chapterMatch[1]) : null
})

const progress = computed(() => {
  if (!currentChapter.value) return 0
  return (currentChapter.value / 6) * 100
})
</script>

<style scoped>
.nav-link {
  @apply font-heading font-semibold text-sm uppercase tracking-wide;
  @apply transition-colors duration-200;
  @apply hover:opacity-80;
}

.nav-link-blue {
  @apply text-schoolhouse-blue;
}

.nav-link-orange {
  @apply text-schoolhouse-orange;
}

.nav-link-green {
  @apply text-schoolhouse-green;
}

.nav-link-purple {
  @apply text-schoolhouse-purple;
}

.nav-link-red {
  @apply text-schoolhouse-red;
}

.nav-link-yellow {
  @apply text-yellow-600;
}

.btn-sm {
  @apply px-4 py-2 text-sm;
}

/* Mobile menu transition */
.mobile-menu-enter-active,
.mobile-menu-leave-active {
  transition: all 0.3s ease;
}

.mobile-menu-enter-from,
.mobile-menu-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
