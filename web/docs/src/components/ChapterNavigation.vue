<template>
  <nav class="chapter-navigation" :class="`chapter-${currentChapter}-theme`">
    <div class="max-w-5xl mx-auto">
      <div class="grid md:grid-cols-2 gap-4">
        <!-- Previous Chapter -->
        <router-link
          v-if="previousChapter"
          :to="`/chapter-${previousChapter.number}`"
          class="nav-card prev-card"
        >
          <div class="nav-arrow">←</div>
          <div class="nav-content">
            <div class="nav-label">Previous</div>
            <div class="nav-title">{{ previousChapter.title }}</div>
            <div class="nav-chapter">Chapter {{ previousChapter.number }}</div>
          </div>
        </router-link>
        <div v-else></div>

        <!-- Next Chapter -->
        <router-link
          v-if="nextChapter"
          :to="`/chapter-${nextChapter.number}`"
          class="nav-card next-card"
        >
          <div class="nav-content">
            <div class="nav-label">Next</div>
            <div class="nav-title">{{ nextChapter.title }}</div>
            <div class="nav-chapter">Chapter {{ nextChapter.number }}</div>
          </div>
          <div class="nav-arrow">→</div>
        </router-link>
        <router-link
          v-else
          to="/research"
          class="nav-card next-card"
        >
          <div class="nav-content">
            <div class="nav-label">Explore</div>
            <div class="nav-title">Research Papers</div>
            <div class="nav-chapter">Learn More</div>
          </div>
          <div class="nav-arrow">→</div>
        </router-link>
      </div>

      <!-- Home Link -->
      <div class="text-center mt-6">
        <router-link to="/" class="home-link">
          ← Back to Home
        </router-link>
      </div>
    </div>
  </nav>
</template>

<script setup>
const props = defineProps({
  currentChapter: {
    type: Number,
    required: true
  }
})

const chapters = [
  { number: 1, title: 'Tracts to Graphs', color: 'blue' },
  { number: 2, title: 'Splitting in Two', color: 'orange' },
  { number: 3, title: 'Recursive Magic', color: 'green' },
  { number: 4, title: 'Making it Compact', color: 'purple' },
  { number: 5, title: 'Voting Rights Act', color: 'red' },
  { number: 6, title: 'Edge-Factor Solution', color: 'yellow' }
]

const previousChapter = props.currentChapter > 1
  ? chapters[props.currentChapter - 2]
  : null

const nextChapter = props.currentChapter < 6
  ? chapters[props.currentChapter]
  : null
</script>

<style scoped>
.chapter-navigation {
  @apply py-16 px-6;
}

/* Chapter theme backgrounds */
.chapter-1-theme { @apply bg-gradient-to-br from-blue-50 to-indigo-50; }
.chapter-2-theme { @apply bg-gradient-to-br from-orange-50 to-red-50; }
.chapter-3-theme { @apply bg-gradient-to-br from-green-50 to-emerald-50; }
.chapter-4-theme { @apply bg-gradient-to-br from-purple-50 to-violet-50; }
.chapter-5-theme { @apply bg-gradient-to-br from-red-50 to-pink-50; }
.chapter-6-theme { @apply bg-gradient-to-br from-yellow-50 to-amber-50; }

.nav-card {
  @apply bg-white rounded-xl shadow-lg p-6 flex items-center gap-4;
  @apply transition-all duration-200 hover:shadow-2xl hover:-translate-y-1;
  @apply border-2 border-transparent hover:border-current;
}

.prev-card {
  @apply justify-start;
}

.next-card {
  @apply justify-end text-right;
}

.nav-arrow {
  @apply text-4xl font-bold opacity-50;
}

.nav-content {
  @apply flex-1;
}

.nav-label {
  @apply text-sm font-semibold uppercase tracking-wide opacity-60 mb-1;
}

.nav-title {
  @apply text-xl font-heading font-bold mb-1;
}

.nav-chapter {
  @apply text-sm opacity-75;
}

.home-link {
  @apply text-gray-600 hover:text-gray-900 font-semibold;
  @apply transition-colors duration-200;
}

/* Color themes for nav cards */
.chapter-1-theme .nav-card { @apply hover:border-schoolhouse-blue text-schoolhouse-blue; }
.chapter-2-theme .nav-card { @apply hover:border-schoolhouse-orange text-schoolhouse-orange; }
.chapter-3-theme .nav-card { @apply hover:border-schoolhouse-green text-schoolhouse-green; }
.chapter-4-theme .nav-card { @apply hover:border-schoolhouse-purple text-schoolhouse-purple; }
.chapter-5-theme .nav-card { @apply hover:border-schoolhouse-red text-schoolhouse-red; }
.chapter-6-theme .nav-card { @apply hover:border-schoolhouse-yellow text-schoolhouse-yellow; }
</style>
