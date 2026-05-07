<template>
  <section class="section-hero" :class="bgClass">
    <div class="container-custom text-center">
      <!-- Chapter Number -->
      <div
        v-if="chapter"
        class="inline-block mb-4 px-6 py-2 rounded-full text-sm font-heading font-bold uppercase tracking-wide"
        :class="badgeClass"
      >
        Chapter {{ chapter }}
      </div>

      <!-- Title -->
      <h1 class="text-hero mb-6 animate-fade-in" :class="accentClass">
        <slot name="title"></slot>
      </h1>

      <!-- Subtitle -->
      <p class="text-subhero text-gray-700 mb-8 animate-slide-up">
        <slot name="subtitle"></slot>
      </p>

      <!-- CTA Button (optional) -->
      <div v-if="$slots.cta" class="animate-slide-up" style="animation-delay: 0.2s">
        <slot name="cta"></slot>
      </div>

      <!-- Hero Image/Animation (optional) -->
      <div
        v-if="$slots.visual"
        class="mt-12 animate-fade-in"
        style="animation-delay: 0.4s"
      >
        <slot name="visual"></slot>
      </div>
    </div>

    <!-- Scroll Indicator -->
    <div class="absolute bottom-8 left-1/2 transform -translate-x-1/2 animate-bounce-subtle">
      <svg
        class="w-6 h-6"
        :class="accentClass"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M19 14l-7 7m0 0l-7-7m7 7V3"
        />
      </svg>
    </div>
  </section>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  chapter: {
    type: Number,
    default: null,
  },
  color: {
    type: String,
    default: 'blue',
    validator: (value) => ['blue', 'orange', 'green', 'purple', 'red', 'yellow'].includes(value),
  },
})

const bgClass = computed(() => `chapter-${props.chapter}-bg`)
const accentClass = computed(() => `chapter-${props.chapter}-accent`)
const badgeClass = computed(() => {
  const colorMap = {
    blue: 'bg-schoolhouse-blue text-white',
    orange: 'bg-schoolhouse-orange text-white',
    green: 'bg-schoolhouse-green text-white',
    purple: 'bg-schoolhouse-purple text-white',
    red: 'bg-schoolhouse-red text-white',
    yellow: 'bg-schoolhouse-yellow text-schoolhouse-gray-dark',
  }
  return colorMap[props.color]
})
</script>

<style scoped>
.section-hero {
  position: relative;
}
</style>
