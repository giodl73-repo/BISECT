<template>
  <div class="split-simulator">
    <div class="simulator-header text-center mb-8">
      <h3 class="text-2xl font-heading font-bold mb-4">{{ title }}</h3>
      <p class="text-gray-600">{{ description }}</p>
    </div>

    <div class="controls mb-6 flex justify-center gap-4 flex-wrap">
      <button
        @click="reset"
        class="btn btn-secondary"
        :disabled="!isSplit"
      >
        Reset
      </button>
      <button
        @click="runSplit"
        class="btn btn-primary"
        :disabled="isSplit || isSplitting"
      >
        {{ isSplitting ? 'Splitting...' : 'Run METIS Split' }}
      </button>
      <select
        v-model="selectedState"
        @change="reset"
        class="px-4 py-2 rounded-lg border-2 border-gray-300 focus:border-schoolhouse-orange focus:outline-none"
        :disabled="isSplit || isSplitting"
      >
        <option value="alabama">Alabama (7)</option>
        <option value="colorado">Colorado (8)</option>
        <option value="vermont">Vermont (1)</option>
      </select>
    </div>

    <div class="visualization-container bg-white rounded-xl shadow-lg p-8">
      <!-- Population Map -->
      <div class="map-container relative">
        <svg ref="svgRef" :width="width" :height="height"></svg>

        <!-- Split Line (animated during split) -->
        <div
          v-if="showSplitLine"
          class="split-line absolute bg-schoolhouse-orange opacity-75"
          :style="splitLineStyle"
        ></div>
      </div>

      <!-- Status Message -->
      <div class="status-message mt-6 text-center">
        <p class="text-lg font-semibold" :class="getStatusColor()">
          {{ statusMessage }}
        </p>
      </div>
    </div>

    <!-- Region Stats (after split) -->
    <div v-if="isSplit" class="region-stats mt-8 grid md:grid-cols-2 gap-8">
      <div class="region-card bg-blue-50 rounded-xl p-6 border-4 border-schoolhouse-blue">
        <div class="flex items-center justify-between mb-4">
          <h4 class="text-2xl font-heading font-bold text-schoolhouse-blue">Region A</h4>
          <div class="region-color w-12 h-12 rounded-full bg-blue-400"></div>
        </div>

        <div class="stats-grid space-y-3">
          <div class="stat-row flex justify-between">
            <span class="text-gray-600">Districts:</span>
            <span class="font-bold text-xl">{{ regionADistricts }}</span>
          </div>
          <div class="stat-row flex justify-between">
            <span class="text-gray-600">Population:</span>
            <span class="font-bold text-xl">{{ regionAPopulation.toLocaleString() }}</span>
          </div>
          <div class="stat-row flex justify-between">
            <span class="text-gray-600">% of Total:</span>
            <span class="font-bold text-xl">{{ regionAPercent }}%</span>
          </div>
          <div class="stat-row flex justify-between">
            <span class="text-gray-600">Balance:</span>
            <span class="font-bold text-xl text-green-600">{{ regionABalance }}</span>
          </div>
        </div>
      </div>

      <div class="region-card bg-red-50 rounded-xl p-6 border-4 border-schoolhouse-red">
        <div class="flex items-center justify-between mb-4">
          <h4 class="text-2xl font-heading font-bold text-schoolhouse-red">Region B</h4>
          <div class="region-color w-12 h-12 rounded-full bg-red-400"></div>
        </div>

        <div class="stats-grid space-y-3">
          <div class="stat-row flex justify-between">
            <span class="text-gray-600">Districts:</span>
            <span class="font-bold text-xl">{{ regionBDistricts }}</span>
          </div>
          <div class="stat-row flex justify-between">
            <span class="text-gray-600">Population:</span>
            <span class="font-bold text-xl">{{ regionBPopulation.toLocaleString() }}</span>
          </div>
          <div class="stat-row flex justify-between">
            <span class="text-gray-600">% of Total:</span>
            <span class="font-bold text-xl">{{ regionBPercent }}%</span>
          </div>
          <div class="stat-row flex justify-between">
            <span class="text-gray-600">Balance:</span>
            <span class="font-bold text-xl text-green-600">{{ regionBBalance }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Key Insight -->
    <div class="insight-panel mt-8 bg-gradient-to-r from-orange-100 to-yellow-100 rounded-xl p-6">
      <h4 class="text-xl font-heading font-bold mb-3 text-schoolhouse-orange">
        💡 How METIS Works
      </h4>
      <ul class="space-y-2 text-sm">
        <li>• <strong>Coarsen:</strong> Group similar tracts together to simplify the graph</li>
        <li>• <strong>Partition:</strong> Split the simplified graph into balanced parts</li>
        <li>• <strong>Uncoarsen:</strong> Expand back to original tract resolution</li>
        <li>• <strong>Refine:</strong> Fine-tune the boundary to minimize edge cut</li>
      </ul>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import * as d3 from 'd3'

const props = defineProps({
  title: {
    type: String,
    default: 'METIS Split Simulator'
  },
  description: {
    type: String,
    default: 'Watch METIS split a state into two balanced regions'
  }
})

const svgRef = ref(null)
const width = ref(600)
const height = ref(400)

const selectedState = ref('alabama')
const isSplit = ref(false)
const isSplitting = ref(false)
const showSplitLine = ref(false)
const splitLineStyle = ref({})

const statusMessage = ref('Click "Run METIS Split" to begin')

// State data
const stateData = {
  alabama: {
    name: 'Alabama',
    totalDistricts: 7,
    totalPopulation: 5030053,
    split: [3, 4]
  },
  colorado: {
    name: 'Colorado',
    totalDistricts: 8,
    totalPopulation: 5773714,
    split: [4, 4]
  },
  vermont: {
    name: 'Vermont',
    totalDistricts: 1,
    totalPopulation: 643077,
    split: [1, 0] // Cannot split
  }
}

const currentState = computed(() => stateData[selectedState.value])

const regionADistricts = computed(() => currentState.value.split[0])
const regionBDistricts = computed(() => currentState.value.split[1])

const regionAPopulation = computed(() => {
  const ratio = regionADistricts.value / currentState.value.totalDistricts
  return Math.round(currentState.value.totalPopulation * ratio)
})

const regionBPopulation = computed(() => {
  const ratio = regionBDistricts.value / currentState.value.totalDistricts
  return Math.round(currentState.value.totalPopulation * ratio)
})

const regionAPercent = computed(() => {
  return ((regionAPopulation.value / currentState.value.totalPopulation) * 100).toFixed(1)
})

const regionBPercent = computed(() => {
  return ((regionBPopulation.value / currentState.value.totalPopulation) * 100).toFixed(1)
})

const regionABalance = computed(() => {
  const target = currentState.value.totalPopulation / 2
  const diff = Math.abs(regionAPopulation.value - target)
  const pct = ((diff / target) * 100).toFixed(2)
  return `±${pct}%`
})

const regionBBalance = computed(() => {
  const target = currentState.value.totalPopulation / 2
  const diff = Math.abs(regionBPopulation.value - target)
  const pct = ((diff / target) * 100).toFixed(2)
  return `±${pct}%`
})

const getStatusColor = () => {
  if (isSplitting.value) return 'text-schoolhouse-orange'
  if (isSplit.value) return 'text-schoolhouse-green'
  return 'text-gray-600'
}

const reset = () => {
  isSplit.value = false
  isSplitting.value = false
  showSplitLine.value = false
  statusMessage.value = 'Click "Run METIS Split" to begin'
  renderMap()
}

const runSplit = async () => {
  if (selectedState.value === 'vermont') {
    statusMessage.value = 'Vermont has only 1 district - cannot split!'
    return
  }

  isSplitting.value = true
  statusMessage.value = 'METIS is analyzing the graph...'

  // Step 1: Coarsen (500ms)
  await new Promise(resolve => setTimeout(resolve, 500))
  statusMessage.value = 'Coarsening graph...'

  // Step 2: Partition (800ms)
  await new Promise(resolve => setTimeout(resolve, 800))
  statusMessage.value = 'Finding optimal partition...'

  // Step 3: Show split line animation
  showSplitLine.value = true
  animateSplitLine()

  // Step 4: Uncoarsen (500ms)
  await new Promise(resolve => setTimeout(resolve, 1500))
  statusMessage.value = 'Uncoarsening to original resolution...'

  // Step 5: Refine (500ms)
  await new Promise(resolve => setTimeout(resolve, 500))
  statusMessage.value = 'Refining boundary...'

  // Step 6: Complete
  await new Promise(resolve => setTimeout(resolve, 500))
  showSplitLine.value = false
  isSplitting.value = false
  isSplit.value = true
  statusMessage.value = `✓ Split complete! Regions are balanced within ±0.5%`

  // Render final split map
  renderSplitMap()
}

const animateSplitLine = () => {
  // Animate split line from top to bottom
  let progress = 0
  const interval = setInterval(() => {
    progress += 2
    splitLineStyle.value = {
      left: '50%',
      top: '0',
      width: '4px',
      height: `${progress}%`,
      transform: 'translateX(-50%)'
    }

    if (progress >= 100) {
      clearInterval(interval)
    }
  }, 20)
}

const renderMap = () => {
  if (!svgRef.value) return

  const svg = d3.select(svgRef.value)
  svg.selectAll('*').remove()

  // Draw simplified state outline
  const stateOutline = svg.append('g')

  // Simple rectangle representing the state
  stateOutline.append('rect')
    .attr('x', 50)
    .attr('y', 50)
    .attr('width', 500)
    .attr('height', 300)
    .attr('fill', '#e0e7ff')
    .attr('stroke', '#6366f1')
    .attr('stroke-width', 3)
    .attr('rx', 10)

  // Add scattered "census tracts" (dots)
  const tractCount = 50
  const tracts = []

  for (let i = 0; i < tractCount; i++) {
    tracts.push({
      x: 75 + Math.random() * 450,
      y: 75 + Math.random() * 250
    })
  }

  stateOutline.selectAll('.tract')
    .data(tracts)
    .enter()
    .append('circle')
    .attr('class', 'tract')
    .attr('cx', d => d.x)
    .attr('cy', d => d.y)
    .attr('r', 4)
    .attr('fill', '#6366f1')
    .attr('opacity', 0.6)
}

const renderSplitMap = () => {
  if (!svgRef.value) return

  const svg = d3.select(svgRef.value)
  svg.selectAll('*').remove()

  // Draw split regions
  const leftWidth = (regionADistricts.value / currentState.value.totalDistricts) * 500

  // Region A (left/blue)
  svg.append('rect')
    .attr('x', 50)
    .attr('y', 50)
    .attr('width', leftWidth)
    .attr('height', 300)
    .attr('fill', '#93c5fd')
    .attr('stroke', '#2563eb')
    .attr('stroke-width', 3)
    .attr('rx', 10)

  svg.append('text')
    .attr('x', 50 + leftWidth / 2)
    .attr('y', 200)
    .attr('text-anchor', 'middle')
    .attr('font-size', '24px')
    .attr('font-weight', 'bold')
    .attr('fill', '#1e40af')
    .text(`Region A (${regionADistricts.value})`)

  // Region B (right/red)
  svg.append('rect')
    .attr('x', 50 + leftWidth)
    .attr('y', 50)
    .attr('width', 500 - leftWidth)
    .attr('height', 300)
    .attr('fill', '#fca5a5')
    .attr('stroke', '#ef4444')
    .attr('stroke-width', 3)
    .attr('rx', 10)

  svg.append('text')
    .attr('x', 50 + leftWidth + (500 - leftWidth) / 2)
    .attr('y', 200)
    .attr('text-anchor', 'middle')
    .attr('font-size', '24px')
    .attr('font-weight', 'bold')
    .attr('fill', '#991b1b')
    .text(`Region B (${regionBDistricts.value})`)

  // Split line
  svg.append('line')
    .attr('x1', 50 + leftWidth)
    .attr('y1', 50)
    .attr('x2', 50 + leftWidth)
    .attr('y2', 350)
    .attr('stroke', '#f97316')
    .attr('stroke-width', 4)
    .attr('stroke-dasharray', '10,5')
}

// Initial render
setTimeout(() => renderMap(), 100)
</script>

<style scoped>
.split-simulator {
  @apply w-full;
}

.btn {
  @apply px-6 py-3 rounded-lg font-semibold text-white transition-all duration-200;
}

.btn-primary {
  @apply bg-gradient-to-r from-schoolhouse-orange to-red-500;
}

.btn-primary:hover:not(:disabled) {
  @apply shadow-lg transform -translate-y-0.5;
}

.btn-secondary {
  @apply bg-gray-500;
}

.btn-secondary:hover:not(:disabled) {
  @apply bg-gray-600 shadow-lg;
}

.btn:disabled {
  @apply opacity-50 cursor-not-allowed;
}

.map-container {
  @apply relative;
}

.split-line {
  @apply pointer-events-none;
  transition: height 0.05s linear;
}
</style>
