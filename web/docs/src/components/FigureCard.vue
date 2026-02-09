<template>
  <div
    ref="figureRef"
    class="figure-card"
    :class="{ 'figure-reveal': animate, 'active': isVisible }"
  >
    <div class="card">
      <!-- Figure Image -->
      <div class="figure-image-container mb-4">
        <img
          :src="src"
          :alt="alt"
          class="w-full h-auto rounded-lg"
          loading="lazy"
        />
      </div>

      <!-- Caption -->
      <div class="figure-caption">
        <p class="text-sm text-gray-600 leading-relaxed">
          <slot></slot>
        </p>
      </div>

      <!-- Optional Research Link -->
      <div v-if="paperLink" class="mt-4 pt-4 border-t border-gray-200">
        <a
          :href="paperLink"
          target="_blank"
          rel="noopener noreferrer"
          class="text-schoolhouse-blue hover:underline text-sm font-semibold"
        >
          View in Research Paper →
        </a>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'

const props = defineProps({
  src: {
    type: String,
    required: true,
  },
  alt: {
    type: String,
    required: true,
  },
  paperLink: {
    type: String,
    default: null,
  },
  animate: {
    type: Boolean,
    default: true,
  },
})

const figureRef = ref(null)
const isVisible = ref(false)

let observer = null

onMounted(() => {
  if (props.animate && figureRef.value) {
    observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            isVisible.value = true
          }
        })
      },
      {
        threshold: 0.2,
      }
    )
    observer.observe(figureRef.value)
  }
})

onUnmounted(() => {
  if (observer && figureRef.value) {
    observer.unobserve(figureRef.value)
  }
})
</script>

<style scoped>
.figure-image-container {
  overflow: hidden;
  border-radius: 0.5rem;
}

.figure-card:hover .figure-image-container img {
  transform: scale(1.05);
  transition: transform 0.3s ease;
}
</style>
