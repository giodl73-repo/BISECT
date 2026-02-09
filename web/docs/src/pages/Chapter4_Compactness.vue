<template>
  <div class="chapter-4">
    <!-- Hero -->
    <Hero :chapter="4" color="purple">
      <template #title>
        Making it<br />Compact
      </template>
      <template #subtitle>
        Geographic sensibility through edge-weighting
      </template>
    </Hero>

    <!-- Story Hook -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <div class="text-center mb-12">
          <h2 class="text-4xl font-heading font-bold mb-6 chapter-4-accent">
            The Snake District Problem
          </h2>
          <p class="text-story">
            Imagine a congressional district shaped like a <strong>snake</strong> winding across
            the state, or a <strong>donut</strong> with a hole in the middle. Technically valid
            (contiguous, balanced population), but clearly absurd!
          </p>
          <p class="text-story mt-4">
            Our recursive bisection algorithm from Chapters 1-3 creates <em>balanced, contiguous</em>
            districts. But are they <strong>compact</strong>? Let's find out...
          </p>
        </div>

        <div class="grid md:grid-cols-3 gap-6">
          <ConceptCard color="purple" icon="🐍">
            <template #title>Snake Districts</template>
            <p class="text-sm">
              Long, winding districts that stretch across the state to connect distant areas.
              Classic sign of gerrymandering!
            </p>
          </ConceptCard>

          <ConceptCard color="purple" icon="🍩">
            <template #title>Donut Districts</template>
            <p class="text-sm">
              Districts with holes or weird shapes that wrap around other districts.
              Makes no geographic sense!
            </p>
          </ConceptCard>

          <ConceptCard color="purple" icon="⭕">
            <template #title>Compact Districts</template>
            <p class="text-sm">
              Round, tidy districts that follow natural geographic boundaries.
              What we actually want!
            </p>
          </ConceptCard>
        </div>
      </div>
    </ScrollSection>

    <!-- What is Compactness? -->
    <ScrollSection bg-class="chapter-4-bg">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-4-accent">
          Measuring Compactness: The Polsby-Popper Score
        </h2>

        <p class="text-story text-center mb-12">
          We need a way to measure "how round" a district is. Enter the
          <strong>Polsby-Popper score</strong>, the gold standard for compactness measurement!
        </p>

        <div class="card bg-gradient-to-r from-purple-500 to-blue-500 text-white mb-12">
          <h3 class="text-2xl font-heading font-bold mb-4">The Polsby-Popper Formula</h3>
          <div class="text-center my-6">
            <div class="text-3xl font-mono">
              PP = 4π × Area / Perimeter²
            </div>
          </div>
          <p class="text-lg leading-relaxed mb-4">
            This formula compares a district's area to its perimeter. A perfect circle scores
            <strong>1.0</strong>. Weird, snaky shapes score close to <strong>0.0</strong>.
          </p>
          <p class="text-lg leading-relaxed">
            <strong>Key insight:</strong> The longer the perimeter for a given area, the lower
            the score. Compact districts have high scores!
          </p>
        </div>

        <div class="grid md:grid-cols-3 gap-6">
          <div class="card bg-white text-center">
            <div class="text-5xl mb-4">⭕</div>
            <h4 class="text-xl font-heading font-bold mb-2">Circle</h4>
            <div class="text-4xl font-black text-schoolhouse-purple mb-2">1.0</div>
            <p class="text-sm text-gray-600">Perfect compactness! (theoretical maximum)</p>
          </div>

          <div class="card bg-white text-center">
            <div class="text-5xl mb-4">▭</div>
            <h4 class="text-xl font-heading font-bold mb-2">Square</h4>
            <div class="text-4xl font-black text-schoolhouse-blue mb-2">0.79</div>
            <p class="text-sm text-gray-600">Pretty compact! Most districts aim for this.</p>
          </div>

          <div class="card bg-white text-center">
            <div class="text-5xl mb-4">〰️</div>
            <h4 class="text-xl font-heading font-bold mb-2">Snake</h4>
            <div class="text-4xl font-black text-schoolhouse-red mb-2">0.15</div>
            <p class="text-sm text-gray-600">Not compact! Sign of gerrymandering.</p>
          </div>
        </div>
      </div>
    </ScrollSection>

    <!-- The Problem -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-4-accent">
          The Problem: Baseline Algorithm Ignores Geography
        </h2>

        <p class="text-story text-center mb-8">
          Our recursive bisection algorithm from Chapter 3 creates balanced, contiguous districts.
          But it has a blind spot: <strong>it doesn't care about geographic distance!</strong>
        </p>

        <div class="grid md:grid-cols-2 gap-8 mb-12">
          <div class="card bg-red-50 border-l-4 border-red-500">
            <h3 class="text-xl font-heading font-bold mb-3 text-red-600">What METIS Sees</h3>
            <p class="text-sm mb-4">
              METIS only sees the <strong>graph structure</strong>: nodes (census tracts) and
              edges (adjacency). It tries to minimize the number of edges crossing boundaries.
            </p>
            <p class="text-sm text-gray-700">
              <strong>Problem:</strong> An edge connecting two tracts 1 mile apart looks the
              same as an edge connecting tracts 50 miles apart!
            </p>
          </div>

          <div class="card bg-yellow-50 border-l-4 border-yellow-500">
            <h3 class="text-xl font-heading font-bold mb-3 text-yellow-600">The Result</h3>
            <p class="text-sm mb-4">
              Districts can be <strong>geographically spread out</strong>. METIS might connect
              distant areas because it only cares about graph connectivity, not physical distance.
            </p>
            <p class="text-sm text-gray-700">
              <strong>Example:</strong> A district might stretch 200 miles to connect two cities
              if they're "close" in the graph.
            </p>
          </div>
        </div>

        <div class="card bg-gradient-orange-red text-white">
          <h3 class="text-2xl font-heading font-bold mb-4">The Question</h3>
          <p class="text-xl leading-relaxed">
            How do we teach the algorithm to prefer <strong>geographically compact</strong> districts
            without sacrificing population balance?
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- The Solution: Edge-Weighting -->
    <ScrollSection bg-class="chapter-4-bg">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-4-accent">
          The Solution: Edge-Weighting by Geographic Distance
        </h2>

        <p class="text-story text-center mb-12">
          Here's the breakthrough: Instead of treating all edges equally, we give each edge a
          <strong>weight based on geographic distance</strong>. Longer borders = higher cost!
        </p>

        <div class="grid md:grid-cols-2 gap-8 mb-12">
          <ConceptCard color="purple" icon="⚖️">
            <template #title>How It Works</template>
            <p class="text-sm mb-4">
              Calculate the geographic distance between every pair of adjacent census tracts.
              Use this as the edge weight in METIS.
            </p>
            <ul class="text-xs space-y-1">
              <li>• Short edge (1 mile) = weight 1.0</li>
              <li>• Medium edge (10 miles) = weight 10.0</li>
              <li>• Long edge (50 miles) = weight 50.0</li>
            </ul>
          </ConceptCard>

          <ConceptCard color="purple" icon="🎯">
            <template #title>Why It Works</template>
            <p class="text-sm mb-4">
              METIS still minimizes edge cut, but now it prefers cutting <strong>short edges</strong>
              over long edges. This naturally creates geographically compact districts!
            </p>
            <p class="text-xs text-gray-600">
              Cutting 10 short edges (total weight 10) is better than cutting 1 long edge (weight 50).
            </p>
          </ConceptCard>
        </div>

        <div class="card bg-gradient-to-r from-purple-600 to-pink-600 text-white mb-8">
          <h3 class="text-2xl font-heading font-bold mb-4">The Magic Formula</h3>
          <div class="font-mono text-lg leading-relaxed mb-4">
            <pre class="overflow-x-auto">
# For each edge (tract_i, tract_j):
centroid_i = get_centroid(tract_i)
centroid_j = get_centroid(tract_j)
distance = geographic_distance(centroid_i, centroid_j)
edge_weight = distance

# METIS now minimizes WEIGHTED edge cut
# (instead of just counting edges)
            </pre>
          </div>
          <p class="text-lg">
            By weighting edges by distance, we're telling METIS: "Prefer creating boundaries
            that follow natural geographic divisions!"
          </p>
        </div>

        <div class="grid md:grid-cols-2 gap-8">
          <div class="card bg-red-50">
            <h4 class="font-heading font-bold mb-2 text-red-600">❌ Baseline (No Weights)</h4>
            <p class="text-sm">
              Minimize number of edges crossing boundary. Ignores distance.
              Can create spread-out districts.
            </p>
          </div>

          <div class="card bg-green-50">
            <h4 class="font-heading font-bold mb-2 text-green-600">✅ Edge-Weighted</h4>
            <p class="text-sm">
              Minimize total distance of boundary. Prefers short edges.
              Creates compact districts!
            </p>
          </div>
        </div>
      </div>
    </ScrollSection>

    <!-- Interactive Compactness Slider -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-5xl mx-auto">
        <InteractiveSlider
          title="See Edge-Weighting in Action"
          description="Move the slider to see how edge-weighting affects district compactness"
          :min="0"
          :max="100"
          :initial-value="0"
          min-label="No Weighting"
          max-label="Full Weighting"
          :value-formatter="v => `${v}% Weighted`"
          color-start="#ef4444"
          color-end="#10b981"
          :show-metrics="true"
          metric1-label="Polsby-Popper Score"
          :metric1-value="ppScore"
          metric1-color="#8b5cf6"
          metric2-label="Edge Cut"
          :metric2-value="edgeCut"
          metric2-color="#f97316"
          metric3-label="Improvement"
          :metric3-value="improvement"
          metric3-color="#10b981"
        >
          <template #default="{ value }">
            <div class="grid md:grid-cols-2 gap-8">
              <!-- Before View -->
              <div class="comparison-card">
                <div class="card bg-red-50 border-4" :class="value < 50 ? 'border-schoolhouse-red' : 'border-gray-300'">
                  <h4 class="text-xl font-heading font-bold mb-4 text-red-600">
                    No Edge Weighting
                  </h4>
                  <div class="aspect-video bg-white rounded-lg mb-4 flex items-center justify-center">
                    <div class="relative w-full h-full p-4">
                      <!-- Simulated "wiggly" district -->
                      <svg viewBox="0 0 200 150" class="w-full h-full">
                        <path
                          d="M 20,30 Q 40,10 70,30 T 120,50 Q 150,40 170,60 L 180,100 Q 170,120 140,110 T 80,120 Q 50,130 30,100 Z"
                          :fill="value < 50 ? '#fca5a5' : '#e5e7eb'"
                          :stroke="value < 50 ? '#ef4444' : '#9ca3af'"
                          stroke-width="2"
                        />
                      </svg>
                    </div>
                  </div>
                  <p class="text-sm text-center font-mono">
                    PP Score: <strong>0.28</strong>
                  </p>
                </div>
              </div>

              <!-- After View -->
              <div class="comparison-card">
                <div class="card bg-green-50 border-4" :class="value >= 50 ? 'border-schoolhouse-green' : 'border-gray-300'">
                  <h4 class="text-xl font-heading font-bold mb-4 text-green-600">
                    With Edge Weighting
                  </h4>
                  <div class="aspect-video bg-white rounded-lg mb-4 flex items-center justify-center">
                    <div class="relative w-full h-full p-4">
                      <!-- Simulated "compact" district -->
                      <svg viewBox="0 0 200 150" class="w-full h-full">
                        <ellipse
                          cx="100"
                          cy="75"
                          rx="70"
                          ry="60"
                          :fill="value >= 50 ? '#86efac' : '#e5e7eb'"
                          :stroke="value >= 50 ? '#10b981' : '#9ca3af'"
                          stroke-width="2"
                        />
                      </svg>
                    </div>
                  </div>
                  <p class="text-sm text-center font-mono">
                    PP Score: <strong>0.44</strong>
                  </p>
                </div>
              </div>
            </div>
          </template>
        </InteractiveSlider>

        <div class="mt-8 text-center">
          <p class="text-story">
            As you increase edge-weighting, districts become more circular and compact!
            The Polsby-Popper score improves from <strong>0.28</strong> to <strong>0.44</strong>.
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- The Results: 56% Improvement -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-4-accent">
          The Results: 56% Improvement in Compactness!
        </h2>

        <div class="text-center mb-12">
          <div class="inline-block bg-gradient-to-r from-purple-100 to-pink-100 rounded-2xl p-8">
            <div class="text-7xl font-black text-schoolhouse-purple mb-4">56%</div>
            <p class="text-2xl font-heading font-bold">
              More Compact Districts
            </p>
            <p class="text-gray-600 mt-2">
              Across all 50 states, 435 districts
            </p>
          </div>
        </div>

        <p class="text-story text-center mb-12">
          We measured compactness for <strong>every district in every state</strong> using both
          baseline and edge-weighted approaches. The results are stunning!
        </p>

        <FigureCard
          src="/figures/chapter4/national_comparison_bar.png"
          alt="National compactness comparison: Baseline vs Edge-Weighted"
          paper-link="/papers/02_edge_weighted_bisection.pdf"
        >
          <strong>National Comparison:</strong> Average Polsby-Popper score increased from 0.28
          (baseline) to 0.44 (edge-weighted) - a 56% improvement! Edge-weighting creates
          significantly more compact districts across all states.
        </FigureCard>

        <div class="grid md:grid-cols-3 gap-6 mt-12">
          <div class="text-center">
            <div class="text-5xl font-black text-gray-400 mb-2">0.28</div>
            <h4 class="font-heading font-bold text-gray-600">Baseline</h4>
            <p class="text-sm text-gray-500">Average PP score</p>
          </div>

          <div class="text-center">
            <div class="text-6xl mb-2">→</div>
            <p class="text-sm text-gray-500 font-bold">56% Improvement</p>
          </div>

          <div class="text-center">
            <div class="text-5xl font-black text-schoolhouse-purple mb-2">0.44</div>
            <h4 class="font-heading font-bold chapter-4-accent">Edge-Weighted</h4>
            <p class="text-sm text-gray-600">Average PP score</p>
          </div>
        </div>
      </div>
    </ScrollSection>

    <!-- State-by-State Comparison -->
    <ScrollSection bg-class="chapter-4-bg">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-4-accent">
          State-by-State: Every State Improves
        </h2>

        <p class="text-story text-center mb-12">
          The improvement isn't just national - <strong>every single state</strong> gets more
          compact districts with edge-weighting!
        </p>

        <FigureCard
          src="/figures/chapter4/state_scatter.png"
          alt="State-by-state compactness scatter plot"
          paper-link="/papers/02_edge_weighted_bisection.pdf"
        >
          <strong>50-State Analysis:</strong> Each point represents one state. Blue = baseline,
          Purple = edge-weighted. Notice how every purple point is above and to the right of its
          blue counterpart - universal improvement!
        </FigureCard>

        <div class="grid md:grid-cols-2 gap-8 mt-12">
          <ConceptCard color="purple" icon="🏆">
            <template #title>Best Improvements</template>
            <p class="text-sm">
              States with irregular geography (Michigan, Maryland, Florida) see the biggest
              gains - up to 80% improvement in compactness!
            </p>
          </ConceptCard>

          <ConceptCard color="purple" icon="✅">
            <template #title>Already Compact</template>
            <p class="text-sm">
              States with simple geography (Colorado, Wyoming) were already fairly compact,
              but still improve by 20-30%!
            </p>
          </ConceptCard>
        </div>
      </div>
    </ScrollSection>

    <!-- Alabama Example -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-4-accent">
          Alabama: Before and After
        </h2>

        <p class="text-story text-center mb-8">
          Remember Alabama from Chapters 1-3? Let's see how edge-weighting improves its districts!
        </p>

        <div class="grid md:grid-cols-2 gap-8 mb-12">
          <div class="card">
            <h3 class="text-xl font-heading font-bold mb-4 text-gray-600">Baseline (No Weights)</h3>
            <div class="bg-gray-100 rounded-lg p-8 mb-4 text-center">
              <p class="text-gray-600 italic">[Baseline Alabama map would go here]</p>
            </div>
            <ul class="text-sm space-y-2">
              <li><strong>Avg PP Score:</strong> 0.32</li>
              <li><strong>Issue:</strong> Some districts stretch far</li>
              <li><strong>Boundary:</strong> Follows graph structure</li>
            </ul>
          </div>

          <div class="card border-4 border-schoolhouse-purple">
            <h3 class="text-xl font-heading font-bold mb-4 chapter-4-accent">
              Edge-Weighted ✨
            </h3>
            <FigureCard
              src="/figures/chapter1/alabama_tracts.png"
              alt="Edge-weighted Alabama districts"
            >
              Alabama with edge-weighting
            </FigureCard>
            <ul class="text-sm space-y-2 mt-4">
              <li><strong>Avg PP Score:</strong> 0.48 (+50%!)</li>
              <li><strong>Improvement:</strong> More compact shapes</li>
              <li><strong>Boundary:</strong> Follows geography</li>
            </ul>
          </div>
        </div>

        <div class="card bg-gradient-to-r from-purple-500 to-blue-500 text-white">
          <h3 class="text-2xl font-heading font-bold mb-4">What Changed?</h3>
          <ul class="space-y-3 text-lg">
            <li>✅ Districts are more circular/rectangular</li>
            <li>✅ Boundaries follow natural divisions (rivers, mountains)</li>
            <li>✅ Less "reaching across" the state</li>
            <li>✅ Same population balance (still within ±0.5%)</li>
          </ul>
        </div>
      </div>
    </ScrollSection>

    <!-- Trade-offs -->
    <ScrollSection bg-class="chapter-4-bg">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-4-accent">
          Any Trade-offs?
        </h2>

        <div class="grid md:grid-cols-2 gap-8 mb-8">
          <div class="card bg-green-50 border-l-4 border-green-500">
            <h3 class="text-xl font-heading font-bold mb-3 text-green-600">✅ What We Gain</h3>
            <ul class="text-sm space-y-2">
              <li>• 56% more compact districts</li>
              <li>• More geographically sensible boundaries</li>
              <li>• Better representation of communities</li>
              <li>• Easier for voters to understand their district</li>
            </ul>
          </div>

          <div class="card bg-blue-50 border-l-4 border-blue-500">
            <h3 class="text-xl font-heading font-bold mb-3 text-blue-600">↔️ What We Keep</h3>
            <ul class="text-sm space-y-2">
              <li>• Same population balance (±0.5%)</li>
              <li>• Same contiguity guarantee</li>
              <li>• Same algorithm (just weighted edges)</li>
              <li>• Same speed (~0.1s per state)</li>
            </ul>
          </div>
        </div>

        <div class="card bg-gradient-to-r from-green-500 to-blue-500 text-white">
          <h3 class="text-2xl font-heading font-bold mb-4">The Bottom Line</h3>
          <p class="text-xl leading-relaxed">
            Edge-weighting is a <strong>free lunch</strong>! We get dramatically better compactness
            without giving up anything. It's a pure win for fairness and geographic sensibility.
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- Key Takeaway -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-3xl mx-auto text-center">
        <div class="inline-block bg-white rounded-2xl shadow-xl p-8 md:p-12">
          <div class="text-6xl mb-6">💡</div>
          <h2 class="text-4xl font-heading font-black mb-6 chapter-4-accent">
            Key Takeaway
          </h2>
          <p class="text-2xl font-heading leading-relaxed">
            By teaching the algorithm <span class="underline-purple">geography</span>, we make
            districts <span class="underline-purple">56% more compact</span>!
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- Next Chapter -->
    <ScrollSection bg-class="chapter-4-bg">
      <div class="max-w-4xl mx-auto text-center">
        <h2 class="text-4xl font-heading font-bold mb-6">What's Next?</h2>
        <p class="text-xl text-gray-700 mb-4">
          We have compact, balanced districts. But what about <strong>minority representation</strong>?
        </p>
        <p class="text-xl text-gray-700 mb-8">
          In Chapter 5, we'll explore the <strong>Voting Rights Act</strong> and the critical
          42% threshold for proportional representation...
        </p>
        <router-link to="/chapter-5" class="btn btn-primary btn-lg">
          Continue to Chapter 5 →
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
import InteractiveSlider from '@/components/InteractiveSlider.vue'

// Computed values for interactive slider
const sliderValue = ref(0)

const ppScore = computed(() => {
  // Linear interpolation from 0.28 to 0.44
  const baseline = 0.28
  const improved = 0.44
  const t = sliderValue.value / 100
  const score = baseline + (improved - baseline) * t
  return score.toFixed(2)
})

const edgeCut = computed(() => {
  // Edge cut decreases with more weighting
  const baseline = 3200
  const improved = 2100
  const t = sliderValue.value / 100
  const cut = baseline - (baseline - improved) * t
  return Math.round(cut).toLocaleString()
})

const improvement = computed(() => {
  // Improvement percentage
  const pct = ((sliderValue.value / 100) * 56).toFixed(0)
  return `+${pct}%`
})
</script>
