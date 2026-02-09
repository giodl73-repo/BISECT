<template>
  <div class="chapter-4">
    <Hero :chapter="4" color="purple">
      <template #title>
        Making it<br />Compact
      </template>
      <template #subtitle>
        Geographic sensibility through edge-weighting
      </template>
    </Hero>

    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto text-center">
        <h2 class="text-4xl font-heading font-bold mb-6 chapter-4-accent">
          Coming Soon!
        </h2>
        <p class="text-story mb-8">
          This chapter will explain edge-weighting and the 56% compactness improvement.
        </p>
        <div class="flex gap-4 justify-center">
          <router-link to="/chapter-3" class="btn btn-outline">
            ← Previous Chapter
          </router-link>
          <router-link to="/chapter-5" class="btn btn-primary">
            Next Chapter →
          </router-link>
        </div>
      </div>
    </ScrollSection>
  </div>
</template>

<script setup>
import Hero from '@/components/Hero.vue'
import ScrollSection from '@/components/ScrollSection.vue'
</script>
