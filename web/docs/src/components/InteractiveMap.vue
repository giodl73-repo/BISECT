<template>
  <div class="interactive-map">
    <div class="map-header text-center mb-6">
      <h3 class="text-2xl font-heading font-bold mb-2">{{ title }}</h3>
      <p class="text-gray-600">{{ description }}</p>
    </div>

    <div class="controls mb-6 flex justify-center gap-4 flex-wrap">
      <button
        v-for="filter in filters"
        :key="filter.id"
        @click="currentFilter = filter.id"
        :class="[
          'px-4 py-2 rounded-lg font-semibold transition-all duration-200',
          currentFilter === filter.id
            ? 'bg-schoolhouse-red text-white shadow-lg'
            : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
        ]"
      >
        {{ filter.label }}
      </button>
    </div>

    <div class="map-container bg-white rounded-xl shadow-lg p-8">
      <svg ref="mapRef" :width="mapWidth" :height="mapHeight" class="mx-auto"></svg>
    </div>

    <div v-if="hoveredState" class="state-tooltip bg-white rounded-xl shadow-xl p-6 mt-6 max-w-md mx-auto">
      <h4 class="text-xl font-heading font-bold mb-4 text-schoolhouse-red">
        {{ hoveredState.name }}
      </h4>
      <div class="grid grid-cols-2 gap-4">
        <div v-for="(stat, key) in hoveredState.stats" :key="key">
          <p class="text-sm text-gray-600">{{ stat.label }}</p>
          <p class="text-2xl font-black" :style="{ color: stat.color || '#64748b' }">
            {{ stat.value }}
          </p>
        </div>
      </div>
    </div>

    <div class="legend mt-8 flex justify-center gap-6">
      <div v-for="item in legend" :key="item.label" class="flex items-center gap-2">
        <div
          class="w-6 h-6 rounded"
          :style="{ backgroundColor: item.color }"
        ></div>
        <span class="text-sm font-semibold">{{ item.label }}</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch } from 'vue'
import * as d3 from 'd3'

const props = defineProps({
  title: {
    type: String,
    required: true
  },
  description: {
    type: String,
    default: ''
  },
  statesData: {
    type: Array,
    required: true
    // Format: [{ name: 'Alabama', abbr: 'AL', stats: { pct: { label: 'Minority %', value: '27%' } } }]
  },
  filters: {
    type: Array,
    default: () => [
      { id: 'all', label: 'All States' }
    ]
  },
  colorScale: {
    type: Function,
    default: (value) => '#cbd5e1' // default gray
  },
  legend: {
    type: Array,
    default: () => []
  }
})

const mapRef = ref(null)
const mapWidth = ref(900)
const mapHeight = ref(550)
const currentFilter = ref('all')
const hoveredState = ref(null)

// Simplified US state positions for a schematic map
const statePositions = {
  'AL': { x: 660, y: 420 }, 'AK': { x: 100, y: 500 }, 'AZ': { x: 250, y: 380 },
  'AR': { x: 580, y: 380 }, 'CA': { x: 150, y: 300 }, 'CO': { x: 370, y: 300 },
  'CT': { x: 820, y: 240 }, 'DE': { x: 800, y: 280 }, 'FL': { x: 740, y: 480 },
  'GA': { x: 700, y: 420 }, 'HI': { x: 350, y: 520 }, 'ID': { x: 270, y: 180 },
  'IL': { x: 620, y: 270 }, 'IN': { x: 660, y: 280 }, 'IA': { x: 560, y: 250 },
  'KS': { x: 490, y: 330 }, 'KY': { x: 680, y: 330 }, 'LA': { x: 600, y: 440 },
  'ME': { x: 850, y: 150 }, 'MD': { x: 780, y: 290 }, 'MA': { x: 840, y: 220 },
  'MI': { x: 680, y: 220 }, 'MN': { x: 540, y: 180 }, 'MS': { x: 620, y: 420 },
  'MO': { x: 570, y: 320 }, 'MT': { x: 330, y: 160 }, 'NE': { x: 460, y: 270 },
  'NV': { x: 200, y: 280 }, 'NH': { x: 840, y: 200 }, 'NJ': { x: 810, y: 260 },
  'NM': { x: 340, y: 390 }, 'NY': { x: 780, y: 220 }, 'NC': { x: 740, y: 360 },
  'ND': { x: 460, y: 160 }, 'OH': { x: 700, y: 280 }, 'OK': { x: 500, y: 380 },
  'OR': { x: 180, y: 180 }, 'PA': { x: 760, y: 270 }, 'RI': { x: 840, y: 230 },
  'SC': { x: 730, y: 390 }, 'SD': { x: 460, y: 210 }, 'TN': { x: 660, y: 360 },
  'TX': { x: 480, y: 450 }, 'UT': { x: 290, y: 300 }, 'VT': { x: 820, y: 190 },
  'VA': { x: 760, y: 320 }, 'WA': { x: 200, y: 130 }, 'WV': { x: 730, y: 310 },
  'WI': { x: 610, y: 200 }, 'WY': { x: 360, y: 230 }
}

onMounted(() => {
  renderMap()
})

watch(currentFilter, () => {
  renderMap()
})

const renderMap = () => {
  if (!mapRef.value) return

  const svg = d3.select(mapRef.value)
  svg.selectAll('*').remove()

  // Filter states based on current filter
  const filteredStates = props.statesData.filter(state => {
    if (currentFilter.value === 'all') return true
    // Custom filter logic can be added here
    return true
  })

  // Create state circles
  const states = svg.selectAll('.state')
    .data(filteredStates)
    .enter()
    .append('g')
    .attr('class', 'state')
    .attr('transform', d => {
      const pos = statePositions[d.abbr] || { x: 400, y: 300 }
      return `translate(${pos.x}, ${pos.y})`
    })
    .on('mouseenter', (event, d) => {
      hoveredState.value = d
      d3.select(event.currentTarget)
        .select('circle')
        .transition()
        .duration(200)
        .attr('r', 22)
        .attr('stroke-width', 4)
    })
    .on('mouseleave', (event, d) => {
      d3.select(event.currentTarget)
        .select('circle')
        .transition()
        .duration(200)
        .attr('r', 18)
        .attr('stroke-width', 2)
    })

  // State circles
  states.append('circle')
    .attr('r', 18)
    .attr('fill', d => getStateColor(d))
    .attr('stroke', '#fff')
    .attr('stroke-width', 2)
    .style('cursor', 'pointer')
    .style('opacity', 0)
    .transition()
    .duration(500)
    .delay((d, i) => i * 10)
    .style('opacity', 0.9)

  // State abbreviations
  states.append('text')
    .attr('text-anchor', 'middle')
    .attr('dy', '0.35em')
    .text(d => d.abbr)
    .attr('fill', '#fff')
    .attr('font-size', '10px')
    .attr('font-weight', 'bold')
    .style('pointer-events', 'none')
    .style('opacity', 0)
    .transition()
    .duration(500)
    .delay((d, i) => i * 10)
    .style('opacity', 1)
}

const getStateColor = (state) => {
  // Use the colorScale function if provided
  if (props.colorScale && state.colorValue !== undefined) {
    return props.colorScale(state.colorValue)
  }
  return '#cbd5e1' // default gray
}
</script>

<style scoped>
.interactive-map {
  @apply w-full;
}

.map-container {
  @apply overflow-visible;
}

.state-tooltip {
  animation: fadeIn 0.3s ease-in-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
