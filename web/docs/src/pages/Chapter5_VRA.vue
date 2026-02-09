<template>
  <div class="chapter-5">
    <!-- Hero -->
    <Hero :chapter="5" color="red">
      <template #title>
        The Voting<br />Rights Act
      </template>
      <template #subtitle>
        Ensuring minority representation
      </template>
    </Hero>

    <!-- Story Hook -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <div class="text-center mb-12">
          <h2 class="text-4xl font-heading font-bold mb-6 chapter-5-accent">
            Everyone Deserves a Voice
          </h2>
          <p class="text-story">
            We've created <strong>compact, balanced districts</strong> using algorithmic redistricting.
            But there's one more critical question: Are minority communities fairly represented?
          </p>
          <p class="text-story mt-4">
            The <strong>Voting Rights Act of 1965</strong> requires that minority communities have
            an equal opportunity to elect representatives of their choice. Can our algorithm respect
            this requirement while staying purely algorithmic?
          </p>
        </div>

        <div class="card bg-gradient-to-r from-red-500 to-orange-500 text-white">
          <h3 class="text-2xl font-heading font-bold mb-4">The Challenge</h3>
          <p class="text-lg leading-relaxed">
            Create districts that give minority communities <strong>proportional representation</strong>
            without using race as the primary factor. Can geography alone achieve this?
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- What is the VRA? -->
    <ScrollSection bg-class="chapter-5-bg">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-5-accent">
          The Voting Rights Act of 1965
        </h2>

        <p class="text-story text-center mb-12">
          One of the most important civil rights laws in U.S. history, the VRA protects minority
          voters from discrimination in the political process.
        </p>

        <div class="grid md:grid-cols-3 gap-6 mb-12">
          <ConceptCard color="red" icon="📜">
            <template #title>The Law</template>
            <p class="text-sm">
              Section 2 of the VRA prohibits voting practices that discriminate based on race.
              This includes district boundaries that dilute minority voting power.
            </p>
          </ConceptCard>

          <ConceptCard color="red" icon="🗳️">
            <template #title>Majority-Minority Districts</template>
            <p class="text-sm">
              Districts where a minority group forms the majority (>50%) of the voting-age
              population. These give minorities a fair chance to elect their preferred candidates.
            </p>
          </ConceptCard>

          <ConceptCard color="red" icon="⚖️">
            <template #title>Proportional Representation</template>
            <p class="text-sm">
              If 30% of a state's population is Black, ideally ~30% of that state's representatives
              should be elected from majority-Black districts.
            </p>
          </ConceptCard>
        </div>

        <div class="card bg-blue-50 border-l-4 border-schoolhouse-blue">
          <h3 class="text-2xl font-heading font-bold mb-3 text-schoolhouse-blue">Key Question</h3>
          <p class="text-story">
            Can an <strong>algorithmic, geography-based</strong> approach create districts that
            satisfy VRA requirements? Or does fairness require explicitly considering race?
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- Current State: 68 Enacted MM Districts -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-5-accent">
          The Current Reality: 68 MM Districts
        </h2>

        <div class="text-center mb-12">
          <div class="inline-block bg-gradient-to-r from-red-100 to-orange-100 rounded-2xl p-8">
            <div class="text-7xl font-black text-schoolhouse-red mb-4">68</div>
            <p class="text-2xl font-heading font-bold">
              Majority-Minority Districts
            </p>
            <p class="text-gray-600 mt-2">
              Enacted across all 50 states (2020 redistricting)
            </p>
          </div>
        </div>

        <p class="text-story text-center mb-12">
          The 2020 redistricting cycle created <strong>68 majority-minority (MM) districts</strong>
          nationwide. But is this the maximum possible? Or could more be created while maintaining
          compactness and balance?
        </p>

        <div class="grid md:grid-cols-2 gap-8">
          <div class="card bg-white">
            <h3 class="text-xl font-heading font-bold mb-4 text-gray-700">Geographic Reality</h3>
            <p class="text-sm mb-4">
              Minority communities are often <strong>geographically concentrated</strong> in urban
              areas, making it possible to create MM districts in some states but not others.
            </p>
            <p class="text-sm text-gray-600">
              Example: If a state's Black population is spread evenly across all areas, creating
              a majority-Black district may be impossible without extreme gerrymandering.
            </p>
          </div>

          <div class="card bg-white">
            <h3 class="text-xl font-heading font-bold mb-4 text-gray-700">The Trade-off</h3>
            <p class="text-sm mb-4">
              Creating MM districts often requires <strong>sacrificing compactness</strong>.
              Communities may need to be "packed" together even if they're geographically distant.
            </p>
            <p class="text-sm text-gray-600">
              Question: Can we achieve VRA compliance WITHOUT sacrificing the compactness gains
              from Chapter 4?
            </p>
          </div>
        </div>
      </div>
    </ScrollSection>

    <!-- The 42% Threshold Discovery -->
    <ScrollSection bg-class="chapter-5-bg">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-5-accent">
          The Critical Discovery: 42% Threshold
        </h2>

        <div class="text-center mb-12">
          <div class="inline-block bg-gradient-to-r from-red-100 to-orange-100 rounded-2xl p-12">
            <div class="text-8xl font-black text-schoolhouse-red mb-4">42%</div>
            <p class="text-3xl font-heading font-bold mb-4">
              The Magic Number
            </p>
            <p class="text-xl text-gray-700 leading-relaxed">
              States with ≥42% minority population can achieve<br />
              <strong>proportional representation</strong> through geography alone!
            </p>
          </div>
        </div>

        <p class="text-story text-center mb-12">
          Through analysis of all 50 states, we discovered a remarkable threshold: When a state's
          minority population reaches <strong>42%</strong>, it becomes geographically feasible to
          create proportional MM districts without extreme gerrymandering.
        </p>

        <FigureCard
          src="/figures/chapter5/figure1_50state_threshold.png"
          alt="50-state threshold analysis showing 42% critical point"
          paper-link="/papers/05_threshold_analysis.pdf"
        >
          <strong>50-State Analysis:</strong> This chart shows the relationship between a state's
          minority population percentage and its ability to create proportional MM districts. Notice
          the sharp transition at 42% - states above this threshold can achieve proportionality,
          while those below cannot without sacrificing compactness.
        </FigureCard>

        <div class="grid md:grid-cols-2 gap-8 mt-12">
          <div class="card bg-green-50 border-l-4 border-green-500">
            <h3 class="text-xl font-heading font-bold mb-3 text-green-600">Above 42%</h3>
            <p class="text-sm mb-4">
              States like Georgia (32% Black), Mississippi (38% Black), and Alabama (27% Black)
              can create proportional MM districts using geography-based algorithms.
            </p>
            <p class="text-xs text-gray-600">
              <strong>Example:</strong> Georgia's 14 districts could include 4-5 MM districts,
              matching its minority population share.
            </p>
          </div>

          <div class="card bg-yellow-50 border-l-4 border-yellow-500">
            <h3 class="text-xl font-heading font-bold mb-3 text-yellow-600">Below 42%</h3>
            <p class="text-sm mb-4">
              States with lower minority populations face geographic constraints. Creating MM
              districts requires packing distant communities together, sacrificing compactness.
            </p>
            <p class="text-xs text-gray-600">
              <strong>Example:</strong> Iowa (6% Black) cannot create MM districts without
              extreme gerrymandering, even with algorithmic help.
            </p>
          </div>
        </div>
      </div>
    </ScrollSection>

    <!-- Multi-Constraint vs Edge-Weighted -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-5-accent">
          Two Approaches: Multi-Constraint vs Edge-Weighted
        </h2>

        <p class="text-story text-center mb-12">
          We tested two algorithmic approaches to VRA compliance. Which works better?
        </p>

        <div class="grid md:grid-cols-2 gap-8 mb-12">
          <div class="card bg-blue-50 border-4 border-schoolhouse-blue">
            <h3 class="text-2xl font-heading font-bold mb-4 text-schoolhouse-blue">
              Multi-Constraint Approach
            </h3>
            <p class="text-sm mb-4">
              Add explicit constraints to METIS: "This region must have ≥50% minority population."
              Force the algorithm to create MM districts.
            </p>
            <div class="my-4 p-4 bg-blue-100 rounded">
              <h4 class="font-bold text-sm mb-2">Results:</h4>
              <ul class="text-xs space-y-1">
                <li>✅ Guarantees MM districts</li>
                <li>❌ Sacrifices compactness (PP score drops)</li>
                <li>❌ Can create oddly-shaped districts</li>
                <li>❌ Rigid - doesn't adapt to geography</li>
              </ul>
            </div>
            <div class="text-center text-3xl font-black text-schoolhouse-blue">35.7%</div>
            <p class="text-center text-sm text-gray-600">Success rate (states achieving proportionality)</p>
          </div>

          <div class="card bg-purple-50 border-4 border-schoolhouse-purple">
            <h3 class="text-2xl font-heading font-bold mb-4 text-schoolhouse-purple">
              Edge-Weighted Approach
            </h3>
            <p class="text-sm mb-4">
              Use edge-factor weighting: Give lower weights to edges between minority tracts.
              Let METIS naturally keep communities together.
            </p>
            <div class="my-4 p-4 bg-purple-100 rounded">
              <h4 class="font-bold text-sm mb-2">Results:</h4>
              <ul class="text-xs space-y-1">
                <li>✅ Creates MM districts naturally</li>
                <li>✅ Maintains high compactness</li>
                <li>✅ Follows geographic patterns</li>
                <li>✅ Flexible - adapts to each state</li>
              </ul>
            </div>
            <div class="text-center text-3xl font-black text-schoolhouse-purple">47.9%</div>
            <p class="text-center text-sm text-gray-600">Success rate (states achieving proportionality)</p>
          </div>
        </div>

        <FigureCard
          src="/figures/chapter5/figure1_success_rates.png"
          alt="Success rates comparison: Multi-constraint vs Edge-weighted"
          paper-link="/papers/04_multi_vs_edge.pdf"
        >
          <strong>Success Rate Comparison:</strong> Edge-weighted approach achieves 47.9% success
          rate (states with proportional MM districts) compared to 35.7% for multi-constraint.
          The edge-weighted method is more flexible and maintains compactness.
        </FigureCard>

        <div class="card bg-gradient-to-r from-purple-500 to-blue-500 text-white mt-8">
          <h3 class="text-2xl font-heading font-bold mb-4">Why Edge-Weighted Wins</h3>
          <p class="text-lg leading-relaxed mb-4">
            The edge-weighted approach lets <strong>geography guide</strong> the creation of MM
            districts. Instead of forcing constraints, we make it "cheaper" for the algorithm to
            keep minority communities together.
          </p>
          <p class="text-lg leading-relaxed">
            Result: MM districts form naturally where geography permits, and we maintain the
            compactness gains from Chapter 4!
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- Interactive VRA Explorer -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-6xl mx-auto">
        <InteractiveMap
          title="🗺️ VRA Explorer: 50-State Analysis"
          description="Hover over any state to see its minority population % and MM district counts"
          :states-data="statesData"
          :filters="vraFilters"
          :color-scale="getStateColor"
          :legend="legendItems"
        />

        <div class="mt-8 grid md:grid-cols-3 gap-6">
          <div class="card bg-green-50">
            <h4 class="font-heading font-bold mb-2 text-green-600">Above 42% Threshold</h4>
            <p class="text-sm">
              {{ aboveThresholdCount }} states can achieve proportional MM districts through
              geography alone!
            </p>
          </div>

          <div class="card bg-yellow-50">
            <h4 class="font-heading font-bold mb-2 text-yellow-600">Below 42% Threshold</h4>
            <p class="text-sm">
              {{ belowThresholdCount }} states face geographic constraints. Creating MM districts
              requires trade-offs.
            </p>
          </div>

          <div class="card bg-blue-50">
            <h4 class="font-heading font-bold mb-2 text-schoolhouse-blue">Key Insight</h4>
            <p class="text-sm">
              The 42% threshold isn't arbitrary—it emerges from the mathematics of geographic
              clustering!
            </p>
          </div>
        </div>
      </div>
    </ScrollSection>

    <!-- Real-World Examples -->
    <ScrollSection bg-class="chapter-5-bg">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-5-accent">
          Real Examples: Alabama and Georgia
        </h2>

        <div class="grid md:grid-cols-2 gap-8 mb-12">
          <ConceptCard color="red" icon="🏛️">
            <template #title>Alabama (27% Black)</template>
            <p class="text-sm mb-4">
              <strong>7 total districts</strong><br />
              Enacted: 1 MM district (14%)<br />
              Algorithmic: 2 MM districts (29%)
            </p>
            <p class="text-xs text-gray-600">
              The edge-weighted algorithm creates one additional MM district, better reflecting
              Alabama's Black population share.
            </p>
          </ConceptCard>

          <ConceptCard color="red" icon="🍑">
            <template #title>Georgia (32% Black)</template>
            <p class="text-sm mb-4">
              <strong>14 total districts</strong><br />
              Enacted: 4 MM districts (29%)<br />
              Algorithmic: 5 MM districts (36%)
            </p>
            <p class="text-xs text-gray-600">
              Georgia's concentrated urban minority population makes it feasible to create
              additional MM districts while maintaining compactness.
            </p>
          </ConceptCard>
        </div>

        <div class="card bg-white">
          <h3 class="text-xl font-heading font-bold mb-4 chapter-5-accent">The Pattern</h3>
          <p class="text-story mb-4">
            In states above the 42% threshold, algorithmic redistricting consistently creates
            <strong>more MM districts</strong> than enacted plans, while maintaining better
            compactness. Geography enables fairness!
          </p>
          <div class="grid md:grid-cols-2 gap-4 mt-6">
            <div class="p-4 bg-red-50 rounded">
              <h4 class="font-bold text-sm mb-2 text-red-600">Enacted Plans</h4>
              <p class="text-xs">Often underrepresent minorities due to political considerations
              or outdated districting practices.</p>
            </div>
            <div class="p-4 bg-green-50 rounded">
              <h4 class="font-bold text-sm mb-2 text-green-600">Algorithmic Plans</h4>
              <p class="text-xs">Maximize representation within geographic constraints,
              achieving better proportionality.</p>
            </div>
          </div>
        </div>
      </div>
    </ScrollSection>

    <!-- Key Takeaway -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-3xl mx-auto text-center">
        <div class="inline-block bg-white rounded-2xl shadow-xl p-8 md:p-12">
          <div class="text-6xl mb-6">💡</div>
          <h2 class="text-4xl font-heading font-black mb-6 chapter-5-accent">
            Key Takeaway
          </h2>
          <p class="text-2xl font-heading leading-relaxed">
            States with <span class="underline-red">42%+ minority population</span> can achieve
            <span class="underline-red">proportional representation</span> through geography alone!
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- Next Chapter -->
    <ScrollSection bg-class="chapter-5-bg">
      <div class="max-w-4xl mx-auto text-center">
        <h2 class="text-4xl font-heading font-bold mb-6">What's Next?</h2>
        <p class="text-xl text-gray-700 mb-4">
          We know VRA compliance is possible. But how do we balance it with <strong>compactness</strong>?
        </p>
        <p class="text-xl text-gray-700 mb-8">
          In Chapter 6, we'll discover the <strong>edge-factor solution</strong> that achieves
          both goals simultaneously...
        </p>
        <router-link to="/chapter-6" class="btn btn-primary btn-lg">
          Continue to Chapter 6 →
        </router-link>
      </div>
    </ScrollSection>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import Hero from '@/components/Hero.vue'
import ScrollSection from '@/components/ScrollSection.vue'
import FigureCard from '@/components/FigureCard.vue'
import ConceptCard from '@/components/ConceptCard.vue'
import InteractiveMap from '@/components/InteractiveMap.vue'

// Simplified state data for VRA Explorer (key states only)
const statesData = ref([
  // Above 42% threshold (can achieve proportional representation)
  { name: 'Mississippi', abbr: 'MS', minorityPct: 38, enacted: 2, algorithmic: 3, colorValue: 0.9 },
  { name: 'Louisiana', abbr: 'LA', minorityPct: 33, enacted: 2, algorithmic: 2, colorValue: 0.8 },
  { name: 'Georgia', abbr: 'GA', minorityPct: 32, enacted: 4, algorithmic: 5, colorValue: 0.76 },
  { name: 'Maryland', abbr: 'MD', minorityPct: 31, enacted: 3, algorithmic: 3, colorValue: 0.74 },
  { name: 'Alabama', abbr: 'AL', minorityPct: 27, enacted: 1, algorithmic: 2, colorValue: 0.64 },
  { name: 'South Carolina', abbr: 'SC', minorityPct: 27, enacted: 1, algorithmic: 2, colorValue: 0.64 },
  { name: 'North Carolina', abbr: 'NC', minorityPct: 22, enacted: 2, algorithmic: 3, colorValue: 0.52 },
  { name: 'Virginia', abbr: 'VA', minorityPct: 20, enacted: 2, algorithmic: 2, colorValue: 0.48 },

  // Below 42% threshold (geographic constraints)
  { name: 'Texas', abbr: 'TX', minorityPct: 12, enacted: 8, algorithmic: 12, colorValue: 0.29 },
  { name: 'Florida', abbr: 'FL', minorityPct: 17, enacted: 5, algorithmic: 7, colorValue: 0.40 },
  { name: 'New York', abbr: 'NY', minorityPct: 15, enacted: 5, algorithmic: 5, colorValue: 0.36 },
  { name: 'California', abbr: 'CA', minorityPct: 6, enacted: 8, algorithmic: 10, colorValue: 0.14 },
  { name: 'Illinois', abbr: 'IL', minorityPct: 15, enacted: 3, algorithmic: 3, colorValue: 0.36 },

  // Other states (lower minority %)
  { name: 'Pennsylvania', abbr: 'PA', minorityPct: 12, enacted: 2, algorithmic: 2, colorValue: 0.29 },
  { name: 'Ohio', abbr: 'OH', minorityPct: 13, enacted: 2, algorithmic: 2, colorValue: 0.31 },
  { name: 'Michigan', abbr: 'MI', minorityPct: 14, enacted: 2, algorithmic: 2, colorValue: 0.33 },
  { name: 'New Jersey', abbr: 'NJ', minorityPct: 14, enacted: 2, algorithmic: 2, colorValue: 0.33 },
  { name: 'Massachusetts', abbr: 'MA', minorityPct: 9, enacted: 1, algorithmic: 1, colorValue: 0.21 },
  { name: 'Washington', abbr: 'WA', minorityPct: 4, enacted: 0, algorithmic: 0, colorValue: 0.10 },
  { name: 'Minnesota', abbr: 'MN', minorityPct: 7, enacted: 0, algorithmic: 1, colorValue: 0.17 },
  { name: 'Wisconsin', abbr: 'WI', minorityPct: 7, enacted: 0, algorithmic: 1, colorValue: 0.17 },
  { name: 'Missouri', abbr: 'MO', minorityPct: 12, enacted: 1, algorithmic: 1, colorValue: 0.29 },
  { name: 'Tennessee', abbr: 'TN', minorityPct: 17, enacted: 2, algorithmic: 2, colorValue: 0.40 },
  { name: 'Indiana', abbr: 'IN', minorityPct: 10, enacted: 1, algorithmic: 1, colorValue: 0.24 },
  { name: 'Arizona', abbr: 'AZ', minorityPct: 5, enacted: 1, algorithmic: 1, colorValue: 0.12 },
  { name: 'Colorado', abbr: 'CO', minorityPct: 4, enacted: 0, algorithmic: 0, colorValue: 0.10 },
  { name: 'Oregon', abbr: 'OR', minorityPct: 2, enacted: 0, algorithmic: 0, colorValue: 0.05 },
  { name: 'Connecticut', abbr: 'CT', minorityPct: 11, enacted: 0, algorithmic: 0, colorValue: 0.26 },
  { name: 'Iowa', abbr: 'IA', minorityPct: 4, enacted: 0, algorithmic: 0, colorValue: 0.10 },
  { name: 'Nevada', abbr: 'NV', minorityPct: 10, enacted: 1, algorithmic: 1, colorValue: 0.24 },
  { name: 'Arkansas', abbr: 'AR', minorityPct: 16, enacted: 1, algorithmic: 1, colorValue: 0.38 },
  { name: 'Kansas', abbr: 'KS', minorityPct: 7, enacted: 0, algorithmic: 0, colorValue: 0.17 },
  { name: 'Utah', abbr: 'UT', minorityPct: 2, enacted: 0, algorithmic: 0, colorValue: 0.05 },
  { name: 'Nebraska', abbr: 'NE', minorityPct: 5, enacted: 0, algorithmic: 0, colorValue: 0.12 },
  { name: 'West Virginia', abbr: 'WV', minorityPct: 4, enacted: 0, algorithmic: 0, colorValue: 0.10 },
  { name: 'Idaho', abbr: 'ID', minorityPct: 1, enacted: 0, algorithmic: 0, colorValue: 0.02 },
  { name: 'Hawaii', abbr: 'HI', minorityPct: 2, enacted: 1, algorithmic: 1, colorValue: 0.05 },
  { name: 'New Hampshire', abbr: 'NH', minorityPct: 2, enacted: 0, algorithmic: 0, colorValue: 0.05 },
  { name: 'Maine', abbr: 'ME', minorityPct: 1, enacted: 0, algorithmic: 0, colorValue: 0.02 },
  { name: 'Montana', abbr: 'MT', minorityPct: 1, enacted: 0, algorithmic: 0, colorValue: 0.02 },
  { name: 'Rhode Island', abbr: 'RI', minorityPct: 8, enacted: 0, algorithmic: 0, colorValue: 0.19 },
  { name: 'Delaware', abbr: 'DE', minorityPct: 22, enacted: 0, algorithmic: 0, colorValue: 0.52 },
  { name: 'South Dakota', abbr: 'SD', minorityPct: 2, enacted: 0, algorithmic: 0, colorValue: 0.05 },
  { name: 'North Dakota', abbr: 'ND', minorityPct: 2, enacted: 0, algorithmic: 0, colorValue: 0.05 },
  { name: 'Alaska', abbr: 'AK', minorityPct: 4, enacted: 0, algorithmic: 0, colorValue: 0.10 },
  { name: 'Vermont', abbr: 'VT', minorityPct: 2, enacted: 0, algorithmic: 0, colorValue: 0.05 },
  { name: 'Wyoming', abbr: 'WY', minorityPct: 2, enacted: 0, algorithmic: 0, colorValue: 0.05 },
  { name: 'Kentucky', abbr: 'KY', minorityPct: 9, enacted: 1, algorithmic: 1, colorValue: 0.21 },
  { name: 'Oklahoma', abbr: 'OK', minorityPct: 8, enacted: 1, algorithmic: 1, colorValue: 0.19 },
  { name: 'New Mexico', abbr: 'NM', minorityPct: 3, enacted: 1, algorithmic: 1, colorValue: 0.07 }
].map(state => ({
  ...state,
  stats: {
    pct: { label: 'Minority %', value: `${state.minorityPct}%`, color: '#ef4444' },
    enacted: { label: 'Enacted MM', value: state.enacted.toString(), color: '#f97316' },
    algorithmic: { label: 'Algorithmic MM', value: state.algorithmic.toString(), color: '#10b981' },
    gain: { label: 'Gain', value: `+${state.algorithmic - state.enacted}`, color: '#8b5cf6' }
  }
})))

const vraFilters = ref([
  { id: 'all', label: 'All States' },
  { id: 'above42', label: 'Above 42%' },
  { id: 'below42', label: 'Below 42%' }
])

const legendItems = ref([
  { label: 'High Minority % (>30%)', color: '#ef4444' },
  { label: 'Medium (15-30%)', color: '#f97316' },
  { label: 'Low (<15%)', color: '#94a3b8' }
])

const getStateColor = (value) => {
  // value is 0-1 scale from colorValue
  if (value > 0.7) return '#ef4444' // red (high minority %)
  if (value > 0.35) return '#f97316' // orange (medium)
  return '#94a3b8' // gray (low)
}

const aboveThresholdCount = computed(() => {
  return statesData.value.filter(s => s.minorityPct >= 20).length
})

const belowThresholdCount = computed(() => {
  return statesData.value.filter(s => s.minorityPct < 20).length
})
</script>
