<template>
  <div class="research-page">
    <!-- Hero -->
    <section class="section-hero bg-gradient-blue-purple text-white">
      <div class="container-custom text-center">
        <h1 class="text-hero mb-6">Research Papers</h1>
        <p class="text-subhero">
          Explore the detailed research behind these concepts
        </p>
      </div>
    </section>

    <!-- Papers Grid -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-6xl mx-auto">
        <div class="text-center mb-12">
          <h2 class="text-4xl font-heading font-bold mb-4">10 Research Papers</h2>
          <p class="text-xl text-gray-600">
            Peer-reviewed research with 210+ figures across all 50 states
          </p>
        </div>

        <!-- Filter/Search (placeholder) -->
        <div class="mb-12 flex gap-4 justify-center flex-wrap">
          <button
            v-for="category in categories"
            :key="category"
            class="btn btn-outline btn-sm"
            :class="{ 'btn-primary': selectedCategory === category }"
            @click="selectedCategory = category"
          >
            {{ category }}
          </button>
        </div>

        <!-- Papers -->
        <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
          <div
            v-for="paper in filteredPapers"
            :key="paper.id"
            class="card-interactive"
          >
            <div class="text-sm text-gray-500 uppercase tracking-wide mb-2">
              {{ paper.category }}
            </div>
            <h3 class="text-xl font-heading font-bold mb-3">
              {{ paper.title }}
            </h3>
            <p class="text-sm text-gray-600 mb-4 line-clamp-3">
              {{ paper.summary }}
            </p>
            <div class="flex gap-2 mb-4 flex-wrap">
              <span
                v-for="tag in paper.tags"
                :key="tag"
                class="px-2 py-1 bg-gray-100 text-gray-700 rounded text-xs"
              >
                {{ tag }}
              </span>
            </div>
            <div class="flex gap-2">
              <a
                v-if="paper.pdfPath"
                :href="paper.pdfPath"
                target="_blank"
                rel="noopener noreferrer"
                class="btn btn-primary btn-sm flex-1"
              >
                Read PDF
              </a>
              <button class="btn btn-outline btn-sm" @click="showFigures(paper)">
                Figures
              </button>
            </div>
          </div>
        </div>
      </div>
    </ScrollSection>

    <!-- Call to Action -->
    <ScrollSection bg-class="bg-gradient-to-br from-blue-50 to-purple-50">
      <div class="max-w-3xl mx-auto text-center">
        <h2 class="text-4xl font-heading font-bold mb-6">Want to Learn More?</h2>
        <p class="text-xl text-gray-700 mb-8">
          Start with Chapter 1 to understand the foundations, or dive directly
          into any topic that interests you.
        </p>
        <router-link to="/chapter-1" class="btn btn-primary btn-lg">
          Start Chapter 1 →
        </router-link>
      </div>
    </ScrollSection>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import ScrollSection from '@/components/ScrollSection.vue'

const selectedCategory = ref('All')

const categories = ['All', 'Foundation', 'Compactness', 'VRA', 'Validation']

const papers = ref([
  {
    id: 1,
    title: 'Comparative Analysis of Redistricting Plans: Enacted, Random, and Algorithmic',
    category: 'Foundation',
    summary: 'Establishes baseline comparison showing random redistricting as middle ground between enacted plans and purely algorithmic approaches.',
    tags: ['comparison', 'baseline', 'compactness'],
    pdfPath: '/papers/01_comparison_plans.pdf',
  },
  {
    id: 2,
    title: 'Edge-Weighted Recursive Bisection for Compact Districts',
    category: 'Compactness',
    summary: 'Introduces edge-weighting methodology achieving 56% improvement in district compactness through geographic distance incorporation.',
    tags: ['edge-weighting', '56% improvement', 'METIS'],
    pdfPath: '/papers/02_edge_weighted_bisection.pdf',
  },
  {
    id: 3,
    title: 'Combined Recursive Bisection: Algorithm and Implementation',
    category: 'Foundation',
    summary: 'Core technical paper detailing the recursive bisection algorithm, METIS integration, and partition tree construction.',
    tags: ['algorithm', 'METIS', 'recursion', 'implementation'],
    pdfPath: '/papers/03_combined_recursive_bisection.pdf',
  },
  {
    id: 4,
    title: 'Multi-Constraint vs Edge-Weighted Approaches to VRA Compliance',
    category: 'VRA',
    summary: 'Compares two algorithmic approaches to Voting Rights Act compliance, showing edge-weighted method achieves both compactness and representation.',
    tags: ['VRA', 'multi-constraint', 'edge-factor', 'comparison'],
    pdfPath: '/papers/04_multi_vs_edge.pdf',
  },
  {
    id: 5,
    title: 'Geographic Threshold Analysis for Minority Representation',
    category: 'VRA',
    summary: 'Discovers critical 42% minority population threshold determining feasibility of proportional representation in redistricting.',
    tags: ['42% threshold', 'VRA', 'geographic constraints'],
    pdfPath: '/papers/05_threshold_analysis.pdf',
  },
  {
    id: 6,
    title: 'Multi-Year Redistricting Analysis: 2000, 2010, 2020',
    category: 'Validation',
    summary: 'Longitudinal study across three census periods validating algorithm stability and demographic trend analysis.',
    tags: ['2000-2020', 'temporal', 'validation'],
    pdfPath: '/papers/06_multiyear_analysis.pdf',
  },
  {
    id: 7,
    title: 'Minnesota Case Study: Progressive Bisection Visualization',
    category: 'Validation',
    summary: 'Detailed case study of Minnesota showing round-by-round bisection process creating 8 congressional districts.',
    tags: ['Minnesota', 'case study', 'visualization'],
    pdfPath: '/papers/07_minnesota_case.pdf',
  },
  {
    id: 8,
    title: 'Alabama Case Study: VRA-Compliant Redistricting',
    category: 'Validation',
    summary: 'Alabama case study demonstrating algorithm capability to create majority-minority districts while maintaining compactness.',
    tags: ['Alabama', 'VRA', 'case study'],
    pdfPath: '/papers/08_alabama_case.pdf',
  },
  {
    id: 9,
    title: 'National Compactness Comparison Across 50 States',
    category: 'Compactness',
    summary: 'Comprehensive analysis comparing algorithmic districts against enacted plans across all states using Polsby-Popper scores.',
    tags: ['50 states', 'compactness', 'national'],
    pdfPath: '/papers/09_national_compactness.pdf',
  },
  {
    id: 10,
    title: 'Synthesis: Algorithmic Redistricting for Fair Democracy',
    category: 'Foundation',
    summary: 'Synthesis paper summarizing all findings and implications for redistricting policy and democratic representation.',
    tags: ['synthesis', 'policy', 'democracy'],
    pdfPath: '/papers/10_synthesis.pdf',
  },
])

const filteredPapers = computed(() => {
  if (selectedCategory.value === 'All') {
    return papers.value
  }
  return papers.value.filter((p) => p.category === selectedCategory.value)
})

function showFigures(paper) {
  // TODO: Implement figure gallery modal
  alert(`Figure gallery for "${paper.title}" coming soon!`)
}
</script>

<style scoped>
.line-clamp-3 {
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
