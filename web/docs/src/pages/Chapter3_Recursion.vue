<template>
  <div class="chapter-3">
    <Hero :chapter="3" color="green">
      <template #title>
        The Recursive<br />Magic
      </template>
      <template #subtitle>
        Creating any number of districts through repeated splitting
      </template>
    </Hero>

    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto text-center">
        <h2 class="text-4xl font-heading font-bold mb-6 chapter-3-accent">
          Coming Soon!
        </h2>
        <p class="text-story mb-8">
          This chapter will explain recursive bisection and how we build the partition tree.
        </p>
        <div class="flex gap-4 justify-center">
          <router-link to="/chapter-2" class="btn btn-outline">
            ← Previous Chapter
          </router-link>
          <router-link to="/chapter-4" class="btn btn-primary">
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
