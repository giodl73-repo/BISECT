<template>
  <div class="chapter-2">
    <Hero :chapter="2" color="orange">
      <template #title>
        Splitting<br />in Two
      </template>
      <template #subtitle>
        How to divide any region into 2 balanced parts
      </template>
    </Hero>

    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto text-center">
        <h2 class="text-4xl font-heading font-bold mb-6 chapter-2-accent">
          Coming Soon!
        </h2>
        <p class="text-story mb-8">
          This chapter will explain the METIS algorithm and how it splits graphs
          into perfectly balanced partitions.
        </p>
        <div class="flex gap-4 justify-center">
          <router-link to="/chapter-1" class="btn btn-outline">
            ← Previous Chapter
          </router-link>
          <router-link to="/chapter-3" class="btn btn-secondary">
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
