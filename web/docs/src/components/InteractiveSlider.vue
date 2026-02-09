<template>
  <div class="interactive-slider">
    <div class="slider-header text-center mb-8">
      <h3 class="text-2xl font-heading font-bold mb-4">{{ title }}</h3>
      <p class="text-gray-600">{{ description }}</p>
    </div>

    <div class="slider-control bg-white rounded-xl shadow-lg p-8 mb-6">
      <div class="flex items-center gap-6">
        <label class="text-sm font-semibold whitespace-nowrap">{{ minLabel }}</label>
        <input
          v-model.number="currentValue"
          type="range"
          :min="min"
          :max="max"
          :step="step"
          class="flex-1 h-3 bg-gray-200 rounded-lg appearance-none cursor-pointer slider-thumb"
        />
        <label class="text-sm font-semibold whitespace-nowrap">{{ maxLabel }}</label>
      </div>

      <div class="text-center mt-6">
        <div class="inline-block bg-gradient-to-r from-purple-100 to-blue-100 rounded-full px-8 py-4">
          <p class="text-sm text-gray-600 mb-1">Current Value</p>
          <p class="text-4xl font-black" :style="{ color: interpolateColor(currentValue) }">
            {{ formatValue(currentValue) }}
          </p>
        </div>
      </div>
    </div>

    <div class="comparison-view">
      <slot :value="currentValue"></slot>
    </div>

    <div v-if="showMetrics" class="metrics-panel grid md:grid-cols-3 gap-6 mt-8">
      <div class="metric-card bg-white rounded-xl shadow p-6 text-center">
        <p class="text-sm text-gray-600 mb-2">{{ metric1Label }}</p>
        <p class="text-3xl font-black" :style="{ color: metric1Color }">
          {{ metric1Value }}
        </p>
      </div>
      <div class="metric-card bg-white rounded-xl shadow p-6 text-center">
        <p class="text-sm text-gray-600 mb-2">{{ metric2Label }}</p>
        <p class="text-3xl font-black" :style="{ color: metric2Color }">
          {{ metric2Value }}
        </p>
      </div>
      <div class="metric-card bg-white rounded-xl shadow p-6 text-center">
        <p class="text-sm text-gray-600 mb-2">{{ metric3Label }}</p>
        <p class="text-3xl font-black" :style="{ color: metric3Color }">
          {{ metric3Value }}
        </p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'

const props = defineProps({
  title: {
    type: String,
    required: true
  },
  description: {
    type: String,
    default: ''
  },
  min: {
    type: Number,
    default: 0
  },
  max: {
    type: Number,
    default: 100
  },
  step: {
    type: Number,
    default: 1
  },
  initialValue: {
    type: Number,
    default: null
  },
  minLabel: {
    type: String,
    default: 'Min'
  },
  maxLabel: {
    type: String,
    default: 'Max'
  },
  valueFormatter: {
    type: Function,
    default: (v) => v.toString()
  },
  colorStart: {
    type: String,
    default: '#ef4444' // red
  },
  colorEnd: {
    type: String,
    default: '#10b981' // green
  },
  showMetrics: {
    type: Boolean,
    default: false
  },
  metric1Label: {
    type: String,
    default: 'Metric 1'
  },
  metric1Value: {
    type: String,
    default: '-'
  },
  metric1Color: {
    type: String,
    default: '#64748b'
  },
  metric2Label: {
    type: String,
    default: 'Metric 2'
  },
  metric2Value: {
    type: String,
    default: '-'
  },
  metric2Color: {
    type: String,
    default: '#64748b'
  },
  metric3Label: {
    type: String,
    default: 'Metric 3'
  },
  metric3Value: {
    type: String,
    default: '-'
  },
  metric3Color: {
    type: String,
    default: '#64748b'
  }
})

const emit = defineEmits(['update:value'])

const currentValue = ref(props.initialValue !== null ? props.initialValue : (props.min + props.max) / 2)

const formatValue = (value) => {
  return props.valueFormatter(value)
}

const interpolateColor = (value) => {
  const t = (value - props.min) / (props.max - props.min)

  // Parse start color
  const start = hexToRgb(props.colorStart)
  const end = hexToRgb(props.colorEnd)

  // Interpolate
  const r = Math.round(start.r + (end.r - start.r) * t)
  const g = Math.round(start.g + (end.g - start.g) * t)
  const b = Math.round(start.b + (end.b - start.b) * t)

  return `rgb(${r}, ${g}, ${b})`
}

const hexToRgb = (hex) => {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex)
  return result ? {
    r: parseInt(result[1], 16),
    g: parseInt(result[2], 16),
    b: parseInt(result[3], 16)
  } : { r: 0, g: 0, b: 0 }
}

// Emit value changes
const onValueChange = () => {
  emit('update:value', currentValue.value)
}
</script>

<style scoped>
.interactive-slider {
  @apply w-full;
}

/* Custom slider thumb styling */
.slider-thumb::-webkit-slider-thumb {
  @apply appearance-none w-6 h-6 bg-schoolhouse-purple rounded-full cursor-pointer shadow-lg;
  transition: all 0.2s;
}

.slider-thumb::-webkit-slider-thumb:hover {
  @apply transform scale-110;
}

.slider-thumb::-moz-range-thumb {
  @apply w-6 h-6 bg-schoolhouse-purple rounded-full cursor-pointer shadow-lg border-0;
  transition: all 0.2s;
}

.slider-thumb::-moz-range-thumb:hover {
  @apply transform scale-110;
}

.metric-card {
  @apply transform transition-all duration-200;
}

.metric-card:hover {
  @apply -translate-y-1 shadow-xl;
}
</style>
