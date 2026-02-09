<template>
  <section
    ref="sectionRef"
    class="section"
    :class="[bgClass, { 'reveal': animate, 'active': isVisible }]"
  >
    <div class="container-custom">
      <slot></slot>
    </div>
  </section>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'

const props = defineProps({
  bgClass: {
    type: String,
    default: '',
  },
  animate: {
    type: Boolean,
    default: true,
  },
})

const sectionRef = ref(null)
const isVisible = ref(false)

let observer = null

onMounted(() => {
  if (props.animate && sectionRef.value) {
    observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            isVisible.value = true
          }
        })
      },
      {
        threshold: 0.1,
        rootMargin: '0px 0px -100px 0px',
      }
    )
    observer.observe(sectionRef.value)
  }
})

onUnmounted(() => {
  if (observer && sectionRef.value) {
    observer.unobserve(sectionRef.value)
  }
})
</script>
