<template>
  <div class="chapter-5">
    <Hero :chapter="5" color="red">
      <template #title>
        The Voting<br />Rights Act
      </template>
      <template #subtitle>
        Ensuring minority representation
      </template>
    </Hero>

    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto text-center">
        <h2 class="text-4xl font-heading font-bold mb-6 chapter-5-accent">
          Coming Soon!
        </h2>
        <p class="text-story mb-8">
          This chapter will explain the VRA and the critical 42% threshold discovery.
        </p>
        <div class="flex gap-4 justify-center">
          <router-link to="/chapter-4" class="btn btn-outline">
            ← Previous Chapter
          </router-link>
          <router-link to="/chapter-6" class="btn btn-primary">
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
