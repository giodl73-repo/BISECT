<template>
  <div id="app" class="min-h-screen">
    <!-- Navigation Component -->
    <Navigation />

    <!-- Main Content -->
    <main id="main-content">
      <router-view v-slot="{ Component }">
        <transition name="page-fade" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </main>

    <!-- Footer -->
    <footer class="bg-schoolhouse-gray-dark text-white py-12 px-4 no-print">
      <div class="container-custom">
        <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
          <!-- About -->
          <div>
            <h3 class="text-2xl font-heading font-bold mb-4">About This Project</h3>
            <p class="text-gray-300 leading-relaxed">
              An educational exploration of algorithmic redistricting, making complex
              computational geometry accessible through interactive visualizations.
            </p>
          </div>

          <!-- Quick Links -->
          <div>
            <h3 class="text-2xl font-heading font-bold mb-4">Chapters</h3>
            <ul class="space-y-2">
              <li v-for="i in 6" :key="i">
                <router-link
                  :to="`/chapter-${i}`"
                  class="text-gray-300 hover:text-white transition-colors"
                >
                  Chapter {{ i }}
                </router-link>
              </li>
            </ul>
          </div>

          <!-- Research -->
          <div>
            <h3 class="text-2xl font-heading font-bold mb-4">Learn More</h3>
            <ul class="space-y-2">
              <li>
                <router-link to="/research" class="text-gray-300 hover:text-white transition-colors">
                  Research Papers
                </router-link>
              </li>
              <li>
                <a
                  href="https://github.com/your-repo/apportionment"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="text-gray-300 hover:text-white transition-colors"
                >
                  View on GitHub
                </a>
              </li>
            </ul>
          </div>
        </div>

        <!-- Copyright -->
        <div class="mt-8 pt-8 border-t border-gray-700 text-center text-gray-400">
          <p>&copy; 2026 Algorithmic Redistricting Research Project. Educational use only.</p>
        </div>
      </div>
    </footer>
  </div>
</template>

<script setup>
import Navigation from './components/Navigation.vue'
</script>

<style scoped>
/* Page transition animations */
.page-fade-enter-active,
.page-fade-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.page-fade-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

.page-fade-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
