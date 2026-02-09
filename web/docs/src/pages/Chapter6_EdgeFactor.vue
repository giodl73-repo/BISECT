<template>
  <div class="chapter-6">
    <Hero :chapter="6" color="yellow">
      <template #title>
        The Edge-Factor<br />Solution
      </template>
      <template #subtitle>
        Balancing compactness with representation
      </template>
    </Hero>

    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto text-center">
        <h2 class="text-4xl font-heading font-bold mb-6 text-yellow-600">
          Coming Soon!
        </h2>
        <p class="text-story mb-8">
          This chapter will explain the edge-factor solution and achieving 137 MM districts.
        </p>
        <div class="flex gap-4 justify-center">
          <router-link to="/chapter-5" class="btn btn-outline">
            ← Previous Chapter
          </router-link>
          <router-link to="/research" class="btn btn-primary">
            View Research →
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
