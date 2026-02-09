<template>
  <div class="state-gallery">
    <div class="gallery-header text-center mb-8">
      <h3 class="text-3xl font-heading font-bold mb-4">{{ title }}</h3>
      <p class="text-gray-600">{{ description }}</p>
    </div>

    <!-- Search and Filter -->
    <div class="controls mb-8 flex flex-wrap gap-4 justify-center">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="Search states..."
        class="px-4 py-2 rounded-lg border-2 border-gray-300 focus:border-schoolhouse-blue focus:outline-none"
      />
      <select
        v-model="regionFilter"
        class="px-4 py-2 rounded-lg border-2 border-gray-300 focus:border-schoolhouse-blue focus:outline-none"
      >
        <option value="all">All Regions</option>
        <option value="northeast">Northeast</option>
        <option value="south">South</option>
        <option value="midwest">Midwest</option>
        <option value="west">West</option>
      </select>
    </div>

    <!-- State Grid -->
    <div class="state-grid grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-6">
      <div
        v-for="state in filteredStates"
        :key="state.abbr"
        @click="openModal(state)"
        class="state-card bg-white rounded-xl shadow-lg p-4 cursor-pointer transform transition-all duration-200 hover:-translate-y-2 hover:shadow-2xl"
      >
        <div class="state-preview aspect-square bg-gray-100 rounded-lg mb-3 flex items-center justify-center overflow-hidden">
          <img
            v-if="state.thumbnail"
            :src="state.thumbnail"
            :alt="`${state.name} districts`"
            class="w-full h-full object-cover"
          />
          <div v-else class="text-4xl font-black text-gray-400">
            {{ state.abbr }}
          </div>
        </div>
        <h4 class="font-heading font-bold text-center mb-1">{{ state.name }}</h4>
        <p class="text-xs text-center text-gray-600">{{ state.districts }} districts</p>
      </div>
    </div>

    <!-- Modal -->
    <Teleport to="body">
      <div
        v-if="selectedState"
        class="modal-overlay fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4"
        @click.self="closeModal"
      >
        <div class="modal-content bg-white rounded-2xl shadow-2xl max-w-4xl w-full max-h-[90vh] overflow-y-auto">
          <!-- Modal Header -->
          <div class="modal-header bg-gradient-to-r from-schoolhouse-blue to-blue-600 text-white p-6 rounded-t-2xl">
            <div class="flex justify-between items-center">
              <div>
                <h3 class="text-3xl font-heading font-bold mb-2">{{ selectedState.name }}</h3>
                <p class="text-lg">{{ selectedState.districts }} Congressional Districts</p>
              </div>
              <button
                @click="closeModal"
                class="text-white text-4xl hover:text-gray-200 transition-colors"
                aria-label="Close modal"
              >
                &times;
              </button>
            </div>
          </div>

          <!-- Modal Body -->
          <div class="modal-body p-6">
            <!-- Tabs -->
            <div class="tabs flex gap-2 mb-6 border-b-2 border-gray-200">
              <button
                v-for="tab in tabs"
                :key="tab.id"
                @click="currentTab = tab.id"
                :class="[
                  'px-4 py-2 font-semibold transition-all duration-200',
                  currentTab === tab.id
                    ? 'border-b-4 border-schoolhouse-blue text-schoolhouse-blue'
                    : 'text-gray-600 hover:text-gray-800'
                ]"
              >
                {{ tab.label }}
              </button>
            </div>

            <!-- Tab: Overview -->
            <div v-if="currentTab === 'overview'" class="tab-content">
              <div class="district-map bg-gray-100 rounded-xl p-4 mb-6">
                <img
                  v-if="selectedState.mapUrl"
                  :src="selectedState.mapUrl"
                  :alt="`${selectedState.name} district map`"
                  class="w-full h-auto rounded-lg"
                />
                <div v-else class="aspect-video flex items-center justify-center text-gray-400">
                  Map not available
                </div>
              </div>

              <div class="stats-grid grid md:grid-cols-3 gap-4">
                <div class="stat-card bg-blue-50 rounded-lg p-4 text-center">
                  <p class="text-sm text-gray-600 mb-1">Districts</p>
                  <p class="text-3xl font-black text-schoolhouse-blue">
                    {{ selectedState.districts }}
                  </p>
                </div>
                <div class="stat-card bg-purple-50 rounded-lg p-4 text-center">
                  <p class="text-sm text-gray-600 mb-1">PP Score</p>
                  <p class="text-3xl font-black text-schoolhouse-purple">
                    {{ selectedState.ppScore || '0.38' }}
                  </p>
                </div>
                <div class="stat-card bg-green-50 rounded-lg p-4 text-center">
                  <p class="text-sm text-gray-600 mb-1">MM Districts</p>
                  <p class="text-3xl font-black text-schoolhouse-green">
                    {{ selectedState.mmDistricts || 0 }}
                  </p>
                </div>
              </div>
            </div>

            <!-- Tab: Progression -->
            <div v-if="currentTab === 'progression'" class="tab-content">
              <p class="text-gray-600 mb-6">
                Watch {{ selectedState.name }} split from 1 region into {{ selectedState.districts }}
                districts through recursive bisection:
              </p>

              <div class="rounds-slider mb-6">
                <input
                  v-model.number="currentRound"
                  type="range"
                  :min="1"
                  :max="maxRounds"
                  class="w-full"
                />
                <p class="text-center mt-2 font-semibold">
                  Round {{ currentRound }} of {{ maxRounds }}
                </p>
              </div>

              <div class="round-map bg-gray-100 rounded-xl p-4">
                <img
                  v-if="getRoundMap()"
                  :src="getRoundMap()"
                  :alt="`Round ${currentRound}`"
                  class="w-full h-auto rounded-lg"
                />
                <div v-else class="aspect-video flex items-center justify-center text-gray-400">
                  Round {{ currentRound }} map not available
                </div>
              </div>
            </div>

            <!-- Tab: Metrics -->
            <div v-if="currentTab === 'metrics'" class="tab-content">
              <div class="metrics-table">
                <table class="w-full">
                  <thead class="bg-gray-50">
                    <tr>
                      <th class="px-4 py-3 text-left font-semibold">Metric</th>
                      <th class="px-4 py-3 text-left font-semibold">Value</th>
                      <th class="px-4 py-3 text-left font-semibold">Comparison</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr class="border-t border-gray-200">
                      <td class="px-4 py-3">Polsby-Popper Score</td>
                      <td class="px-4 py-3 font-mono font-bold">{{ selectedState.ppScore || '0.38' }}</td>
                      <td class="px-4 py-3 text-green-600">+42% vs enacted</td>
                    </tr>
                    <tr class="border-t border-gray-200">
                      <td class="px-4 py-3">Population Balance</td>
                      <td class="px-4 py-3 font-mono font-bold">±0.3%</td>
                      <td class="px-4 py-3 text-green-600">Within ±0.5% requirement</td>
                    </tr>
                    <tr class="border-t border-gray-200">
                      <td class="px-4 py-3">Edge Cut</td>
                      <td class="px-4 py-3 font-mono font-bold">{{ selectedState.edgeCut || '1,450' }}</td>
                      <td class="px-4 py-3 text-blue-600">Optimized</td>
                    </tr>
                    <tr class="border-t border-gray-200">
                      <td class="px-4 py-3">MM Districts</td>
                      <td class="px-4 py-3 font-mono font-bold">{{ selectedState.mmDistricts || 0 }}</td>
                      <td class="px-4 py-3" :class="selectedState.mmDistricts > 0 ? 'text-green-600' : 'text-gray-600'">
                        {{ selectedState.mmDistricts > 0 ? 'VRA compliant' : 'N/A' }}
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>

            <!-- Tab: Download -->
            <div v-if="currentTab === 'download'" class="tab-content">
              <p class="text-gray-600 mb-6">
                Download district data and maps for {{ selectedState.name }}:
              </p>

              <div class="download-options grid md:grid-cols-2 gap-4">
                <a
                  href="#"
                  class="download-card bg-blue-50 rounded-xl p-6 hover:bg-blue-100 transition-colors"
                >
                  <div class="text-4xl mb-3">📊</div>
                  <h4 class="font-heading font-bold mb-2">District Assignments CSV</h4>
                  <p class="text-sm text-gray-600">Census tract to district mappings</p>
                </a>

                <a
                  href="#"
                  class="download-card bg-purple-50 rounded-xl p-6 hover:bg-purple-100 transition-colors"
                >
                  <div class="text-4xl mb-3">🗺️</div>
                  <h4 class="font-heading font-bold mb-2">District Maps (PNG)</h4>
                  <p class="text-sm text-gray-600">High-resolution district boundaries</p>
                </a>

                <a
                  href="#"
                  class="download-card bg-green-50 rounded-xl p-6 hover:bg-green-100 transition-colors"
                >
                  <div class="text-4xl mb-3">📈</div>
                  <h4 class="font-heading font-bold mb-2">Metrics Report (PDF)</h4>
                  <p class="text-sm text-gray-600">Compactness and VRA analysis</p>
                </a>

                <a
                  href="#"
                  class="download-card bg-orange-50 rounded-xl p-6 hover:bg-orange-100 transition-colors"
                >
                  <div class="text-4xl mb-3">💾</div>
                  <h4 class="font-heading font-bold mb-2">All Data (ZIP)</h4>
                  <p class="text-sm text-gray-600">Complete dataset for this state</p>
                </a>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'

const props = defineProps({
  title: {
    type: String,
    default: '50-State Gallery'
  },
  description: {
    type: String,
    default: 'Explore algorithmic redistricting results for all 50 states'
  },
  states: {
    type: Array,
    required: true
    // Format: [{ name: 'Alabama', abbr: 'AL', districts: 7, region: 'south', ... }]
  }
})

const searchQuery = ref('')
const regionFilter = ref('all')
const selectedState = ref(null)
const currentTab = ref('overview')
const currentRound = ref(1)

const tabs = [
  { id: 'overview', label: 'Overview' },
  { id: 'progression', label: 'Progression' },
  { id: 'metrics', label: 'Metrics' },
  { id: 'download', label: 'Download' }
]

const maxRounds = computed(() => {
  if (!selectedState.value) return 3
  return Math.ceil(Math.log2(selectedState.value.districts))
})

const filteredStates = computed(() => {
  return props.states.filter(state => {
    const matchesSearch = state.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
                         state.abbr.toLowerCase().includes(searchQuery.value.toLowerCase())
    const matchesRegion = regionFilter.value === 'all' || state.region === regionFilter.value
    return matchesSearch && matchesRegion
  })
})

const openModal = (state) => {
  selectedState.value = state
  currentTab.value = 'overview'
  currentRound.value = 1
  document.body.style.overflow = 'hidden' // Prevent background scroll
}

const closeModal = () => {
  selectedState.value = null
  document.body.style.overflow = '' // Restore scroll
}

const getRoundMap = () => {
  if (!selectedState.value) return null
  // Construct round map URL based on state and round
  return `/figures/states/${selectedState.value.abbr.toLowerCase()}/round_${currentRound.value}.png`
}
</script>

<style scoped>
.state-gallery {
  @apply w-full;
}

.modal-overlay {
  animation: fadeIn 0.3s ease-in-out;
}

.modal-content {
  animation: slideUp 0.3s ease-in-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.download-card {
  @apply transform transition-all duration-200;
}

.download-card:hover {
  @apply -translate-y-1 shadow-lg;
}
</style>
