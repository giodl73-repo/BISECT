<template>
  <div class="graph-builder">
    <div class="builder-header text-center mb-8">
      <h3 class="text-2xl font-heading font-bold mb-4">{{ title }}</h3>
      <p class="text-gray-600">{{ description }}</p>
    </div>

    <div class="controls mb-6 flex justify-center gap-4">
      <button
        @click="reset"
        class="btn btn-secondary"
        :disabled="currentStep === 0"
      >
        Reset
      </button>
      <button
        @click="nextStep"
        class="btn btn-primary"
        :disabled="currentStep >= totalSteps"
      >
        {{ getStepLabel() }}
      </button>
      <button
        @click="toggleView"
        class="btn btn-secondary"
        :disabled="currentStep < 2"
      >
        {{ viewMode === 'geographic' ? 'Show Network' : 'Show Geographic' }}
      </button>
    </div>

    <div class="visualization bg-white rounded-xl shadow-lg p-8">
      <svg ref="svgRef" :width="width" :height="height"></svg>

      <div class="step-info mt-4 text-center">
        <p class="text-lg font-semibold">
          {{ getStepDescription() }}
        </p>
      </div>
    </div>

    <div class="info-panel mt-6 grid md:grid-cols-3 gap-4">
      <div class="info-card bg-blue-50 rounded-lg p-4">
        <h4 class="font-bold text-sm mb-2 text-schoolhouse-blue">Census Tracts</h4>
        <p class="text-2xl font-black">{{ tractCount }}</p>
        <p class="text-xs text-gray-600">Neighborhoods in {{ stateName }}</p>
      </div>

      <div class="info-card bg-orange-50 rounded-lg p-4">
        <h4 class="font-bold text-sm mb-2 text-schoolhouse-orange">Edges</h4>
        <p class="text-2xl font-black">{{ visibleEdges }}</p>
        <p class="text-xs text-gray-600">Connections between neighbors</p>
      </div>

      <div class="info-card bg-green-50 rounded-lg p-4">
        <h4 class="font-bold text-sm mb-2 text-schoolhouse-green">Graph Density</h4>
        <p class="text-2xl font-black">{{ graphDensity }}</p>
        <p class="text-xs text-gray-600">Avg edges per tract</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch, computed } from 'vue'
import * as d3 from 'd3'

const props = defineProps({
  title: {
    type: String,
    default: 'From Tracts to Graphs'
  },
  description: {
    type: String,
    default: 'Watch census tracts transform into a network graph'
  },
  stateName: {
    type: String,
    default: 'Alabama'
  },
  tractCount: {
    type: Number,
    default: 30 // Simplified for visualization (actual Alabama has 1,181)
  }
})

const svgRef = ref(null)
const width = ref(800)
const height = ref(500)
const currentStep = ref(0)
const totalSteps = ref(3) // 0: empty, 1: tracts, 2: edges, 3: complete
const viewMode = ref('geographic') // 'geographic' or 'network'

// Generate synthetic tract data
const generateTracts = () => {
  const tracts = []
  const cols = 6
  const rows = 5
  const spacing = 100
  const offsetX = 100
  const offsetY = 50

  for (let i = 0; i < props.tractCount; i++) {
    const col = i % cols
    const row = Math.floor(i / cols)

    // Add some randomness to make it look more natural
    const jitterX = (Math.random() - 0.5) * 30
    const jitterY = (Math.random() - 0.5) * 30

    tracts.push({
      id: i,
      x: offsetX + col * spacing + jitterX,
      y: offsetY + row * spacing + jitterY,
      neighbors: []
    })
  }

  // Generate edges (simplified grid adjacency)
  tracts.forEach((tract, i) => {
    const col = i % cols
    const row = Math.floor(i / cols)

    // Right neighbor
    if (col < cols - 1 && i + 1 < tracts.length) {
      tract.neighbors.push(i + 1)
    }

    // Bottom neighbor
    if (row < rows - 1 && i + cols < tracts.length) {
      tract.neighbors.push(i + cols)
    }

    // Diagonal (some tracts)
    if (Math.random() > 0.6 && col < cols - 1 && row < rows - 1 && i + cols + 1 < tracts.length) {
      tract.neighbors.push(i + cols + 1)
    }
  })

  return tracts
}

const tracts = ref(generateTracts())
const edges = ref([])

// Calculate edges from tracts
const calculateEdges = () => {
  const edgeList = []
  tracts.value.forEach(tract => {
    tract.neighbors.forEach(neighborId => {
      const neighbor = tracts.value[neighborId]
      if (neighbor) {
        edgeList.push({
          source: tract,
          target: neighbor
        })
      }
    })
  })
  return edgeList
}

edges.value = calculateEdges()

const visibleEdges = computed(() => {
  if (currentStep.value < 2) return 0
  return edges.value.length
})

const graphDensity = computed(() => {
  if (currentStep.value < 2) return '0.0'
  const avgDegree = (edges.value.length * 2) / tracts.value.length
  return avgDegree.toFixed(1)
})

onMounted(() => {
  renderVisualization()
})

watch([currentStep, viewMode], () => {
  renderVisualization()
})

const reset = () => {
  currentStep.value = 0
  viewMode.value = 'geographic'
}

const nextStep = () => {
  if (currentStep.value < totalSteps.value) {
    currentStep.value++
  }
}

const toggleView = () => {
  viewMode.value = viewMode.value === 'geographic' ? 'network' : 'geographic'
}

const getStepLabel = () => {
  const labels = ['Show Tracts', 'Add Edges', 'Complete Graph']
  return labels[currentStep.value] || 'Done'
}

const getStepDescription = () => {
  const descriptions = [
    'Empty state',
    `Step 1: ${props.tractCount} census tracts appear as nodes`,
    `Step 2: ${edges.value.length} edges connect neighboring tracts`,
    'Step 3: Complete graph ready for partitioning!'
  ]
  return descriptions[currentStep.value] || ''
}

const renderVisualization = () => {
  if (!svgRef.value) return

  const svg = d3.select(svgRef.value)
  svg.selectAll('*').remove()

  if (currentStep.value === 0) return

  const g = svg.append('g')
    .attr('transform', 'translate(0, 0)')

  // Step 2+: Draw edges
  if (currentStep.value >= 2) {
    g.selectAll('.edge')
      .data(edges.value)
      .enter()
      .append('line')
      .attr('class', 'edge')
      .attr('x1', d => d.source.x)
      .attr('y1', d => d.source.y)
      .attr('x2', d => d.target.x)
      .attr('y2', d => d.target.y)
      .attr('stroke', '#cbd5e1')
      .attr('stroke-width', 2)
      .style('opacity', 0)
      .transition()
      .duration(1000)
      .delay((d, i) => i * 5)
      .style('opacity', 0.5)
  }

  // Step 1+: Draw tracts (nodes)
  if (currentStep.value >= 1) {
    const nodes = g.selectAll('.node')
      .data(tracts.value)
      .enter()
      .append('g')
      .attr('class', 'node')
      .attr('transform', d => `translate(${d.x}, ${d.y})`)

    nodes.append('circle')
      .attr('r', 8)
      .attr('fill', '#2563eb')
      .attr('stroke', '#fff')
      .attr('stroke-width', 2)
      .style('opacity', 0)
      .transition()
      .duration(500)
      .delay((d, i) => currentStep.value === 1 ? i * 20 : 0)
      .style('opacity', 1)

    // Add labels (show only for first few tracts)
    nodes.filter((d, i) => i < 5)
      .append('text')
      .attr('dy', -15)
      .attr('text-anchor', 'middle')
      .attr('font-size', '10px')
      .attr('fill', '#64748b')
      .text(d => `T${d.id}`)
      .style('opacity', 0)
      .transition()
      .duration(500)
      .delay((d, i) => currentStep.value === 1 ? i * 20 + 500 : 0)
      .style('opacity', 1)
  }

  // Network view: Apply force simulation
  if (viewMode.value === 'network' && currentStep.value >= 2) {
    const simulation = d3.forceSimulation(tracts.value)
      .force('link', d3.forceLink(edges.value).id(d => d.id).distance(80))
      .force('charge', d3.forceManyBody().strength(-200))
      .force('center', d3.forceCenter(width.value / 2, height.value / 2))
      .force('collision', d3.forceCollide().radius(20))

    simulation.on('tick', () => {
      g.selectAll('.edge')
        .attr('x1', d => d.source.x)
        .attr('y1', d => d.source.y)
        .attr('x2', d => d.target.x)
        .attr('y2', d => d.target.y)

      g.selectAll('.node')
        .attr('transform', d => `translate(${d.x}, ${d.y})`)
    })

    // Run simulation for a bit then stop
    setTimeout(() => simulation.stop(), 2000)
  }
}
</script>

<style scoped>
.graph-builder {
  @apply w-full;
}

.btn {
  @apply px-6 py-3 rounded-lg font-semibold text-white transition-all duration-200;
}

.btn-primary {
  @apply bg-gradient-to-r from-schoolhouse-blue to-blue-600;
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

.info-card {
  @apply transform transition-all duration-200;
}

.info-card:hover {
  @apply -translate-y-1 shadow-lg;
}
</style>
