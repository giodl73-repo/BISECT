<template>
  <div class="card-interactive" :class="colorClass">
    <!-- Icon (optional) -->
    <div v-if="icon" class="mb-4">
      <div
        class="w-16 h-16 rounded-full flex items-center justify-center text-3xl"
        :class="iconBgClass"
      >
        {{ icon }}
      </div>
    </div>

    <!-- Title -->
    <h3 class="text-2xl font-heading font-bold mb-3" :class="accentClass">
      <slot name="title"></slot>
    </h3>

    <!-- Content -->
    <div class="text-story text-gray-700">
      <slot></slot>
    </div>

    <!-- Optional Action -->
    <div v-if="$slots.action" class="mt-6">
      <slot name="action"></slot>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  color: {
    type: String,
    default: 'blue',
    validator: (value) => ['blue', 'orange', 'green', 'purple', 'red', 'yellow'].includes(value),
  },
  icon: {
    type: String,
    default: null,
  },
})

const colorClass = computed(() => {
  const colorMap = {
    blue: 'border-l-4 border-schoolhouse-blue',
    orange: 'border-l-4 border-schoolhouse-orange',
    green: 'border-l-4 border-schoolhouse-green',
    purple: 'border-l-4 border-schoolhouse-purple',
    red: 'border-l-4 border-schoolhouse-red',
    yellow: 'border-l-4 border-schoolhouse-yellow',
  }
  return colorMap[props.color]
})

const accentClass = computed(() => {
  const colorMap = {
    blue: 'text-schoolhouse-blue',
    orange: 'text-schoolhouse-orange',
    green: 'text-schoolhouse-green',
    purple: 'text-schoolhouse-purple',
    red: 'text-schoolhouse-red',
    yellow: 'text-yellow-600',
  }
  return colorMap[props.color]
})

const iconBgClass = computed(() => {
  const colorMap = {
    blue: 'bg-blue-100',
    orange: 'bg-orange-100',
    green: 'bg-green-100',
    purple: 'bg-purple-100',
    red: 'bg-red-100',
    yellow: 'bg-yellow-100',
  }
  return colorMap[props.color]
})
</script>
