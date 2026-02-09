<template>
  <div class="chapter-6">
    <!-- Hero -->
    <Hero :chapter="6" color="yellow">
      <template #title>
        The Edge-Factor<br />Solution
      </template>
      <template #subtitle>
        Balancing compactness with representation
      </template>
    </Hero>

    <!-- Story Hook -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <div class="text-center mb-12">
          <h2 class="text-4xl font-heading font-bold mb-6 text-yellow-600">
            Having Your Cake and Eating It Too
          </h2>
          <p class="text-story">
            We've learned two powerful techniques:
          </p>
          <div class="grid md:grid-cols-2 gap-6 my-8">
            <div class="card bg-purple-50 border-l-4 border-schoolhouse-purple">
              <h3 class="font-heading font-bold text-purple-600 mb-2">Chapter 4</h3>
              <p class="text-sm">Edge-weighting by <strong>geographic distance</strong> makes
              districts 56% more compact</p>
            </div>
            <div class="card bg-red-50 border-l-4 border-schoolhouse-red">
              <h3 class="font-heading font-bold text-red-600 mb-2">Chapter 5</h3>
              <p class="text-sm">Edge-weighting by <strong>minority boundaries</strong> creates
              more VRA-compliant districts</p>
            </div>
          </div>
          <p class="text-story">
            But what if we could do <strong>both at the same time</strong>? That's where
            <em>edge-factor weighting</em> comes in!
          </p>
        </div>

        <div class="card bg-gradient-to-r from-yellow-400 to-orange-500 text-white">
          <h3 class="text-2xl font-heading font-bold mb-4">The Ultimate Goal</h3>
          <p class="text-xl leading-relaxed">
            Create districts that are <strong>compact</strong> (following geography),
            <strong>balanced</strong> (equal population), AND provide
            <strong>proportional minority representation</strong> - all automatically!
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- The Trade-off Problem -->
    <ScrollSection bg-class="chapter-6-bg">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center text-yellow-600">
          The Trade-off Problem
        </h2>

        <p class="text-story text-center mb-12">
          Traditionally, compactness and minority representation are seen as
          <strong>competing goals</strong>. You have to choose one or the other.
        </p>

        <div class="grid md:grid-cols-2 gap-8 mb-12">
          <div class="card bg-white">
            <h3 class="text-xl font-heading font-bold mb-4 text-schoolhouse-purple">
              Prioritize Compactness
            </h3>
            <div class="mb-4 p-4 bg-purple-50 rounded">
              <p class="text-sm font-bold mb-2">Result:</p>
              <ul class="text-xs space-y-1">
                <li>✅ Very compact districts (PP score: 0.44)</li>
                <li>✅ Follow natural geography</li>
                <li>❌ Fewer MM districts (~80)</li>
                <li>❌ Underrepresents minorities</li>
              </ul>
            </div>
            <p class="text-xs text-gray-600">
              Using only geographic distance weighting gives us beautiful, compact districts,
              but may not provide adequate minority representation.
            </p>
          </div>

          <div class="card bg-white">
            <h3 class="text-xl font-heading font-bold mb-4 text-schoolhouse-red">
              Prioritize VRA Compliance
            </h3>
            <div class="mb-4 p-4 bg-red-50 rounded">
              <p class="text-sm font-bold mb-2">Result:</p>
              <ul class="text-xs space-y-1">
                <li>✅ Many MM districts (~120)</li>
                <li>✅ Proportional representation</li>
                <li>❌ Lower compactness (PP score: 0.32)</li>
                <li>❌ Oddly-shaped districts</li>
              </ul>
            </div>
            <p class="text-xs text-gray-600">
              Forcing MM district creation through constraints gives us representation,
              but sacrifices the compactness gains from Chapter 4.
            </p>
          </div>
        </div>

        <div class="card bg-yellow-50 border-l-4 border-yellow-500">
          <h3 class="text-2xl font-heading font-bold mb-3 text-yellow-600">The Question</h3>
          <p class="text-story">
            Can we find a <strong>middle ground</strong> that achieves high compactness
            AND proportional representation? Is there a way to optimize for both simultaneously?
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- The Edge-Factor Solution -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center text-yellow-600">
          The Edge-Factor Solution
        </h2>

        <p class="text-story text-center mb-12">
          The breakthrough: <strong>Combine both weighting schemes</strong> into a single
          unified edge weight that balances geography and community!
        </p>

        <div class="card bg-gradient-to-r from-yellow-500 via-orange-500 to-red-500 text-white mb-12">
          <h3 class="text-2xl font-heading font-bold mb-4">The Formula</h3>
          <div class="font-mono text-lg leading-relaxed mb-4 bg-white/20 p-4 rounded">
            <pre class="overflow-x-auto">
# For each edge (tract_i, tract_j):

# Geographic distance (from Chapter 4)
distance = geographic_distance(tract_i, tract_j)

# Minority factor (from Chapter 5)
# If both tracts have high minority %, reduce weight
if both_high_minority(tract_i, tract_j):
    minority_factor = 0.5  # Cheaper to cross
else:
    minority_factor = 1.0  # Normal cost

# Combined weight
edge_weight = distance × minority_factor
            </pre>
          </div>
          <p class="text-lg leading-relaxed">
            By multiplying distance by minority_factor, we tell METIS: "Prefer short edges
            (compactness), but make edges between minority tracts even shorter (representation)!"
          </p>
        </div>

        <div class="grid md:grid-cols-3 gap-6">
          <ConceptCard color="yellow" icon="🌍">
            <template #title>Geographic Distance</template>
            <p class="text-sm">
              Base weight = physical distance between tract centroids.
              This preserves compactness from Chapter 4.
            </p>
          </ConceptCard>

          <ConceptCard color="yellow" icon="👥">
            <template #title>Minority Factor</template>
            <p class="text-sm">
              Multiply by 0.5 if both tracts have high minority population.
              This encourages keeping communities together.
            </p>
          </ConceptCard>

          <ConceptCard color="yellow" icon="⚖️">
            <template #title>Balanced Result</template>
            <p class="text-sm">
              METIS minimizes total weighted edge cut, naturally finding
              districts that are both compact AND representative!
            </p>
          </ConceptCard>
        </div>
      </div>
    </ScrollSection>

    <!-- How It Works -->
    <ScrollSection bg-class="chapter-6-bg">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center text-yellow-600">
          How It Works: A Visual Example
        </h2>

        <p class="text-story text-center mb-12">
          Let's see how edge-factor weighting guides METIS to create better districts!
        </p>

        <div class="grid md:grid-cols-2 gap-8 mb-12">
          <div class="card bg-white">
            <h3 class="text-xl font-heading font-bold mb-4">Scenario: Alabama District Boundary</h3>
            <p class="text-sm mb-4">
              METIS needs to decide where to place the boundary between two districts.
              Should it split through the urban minority community or around it?
            </p>
            <div class="bg-gray-100 p-4 rounded mb-4">
              <p class="text-xs font-mono mb-2">Option A: Split through community</p>
              <ul class="text-xs space-y-1">
                <li>• 3 edges × 10 miles each × factor 0.5 = weight 15</li>
                <li>• Breaks up minority community</li>
              </ul>
            </div>
            <div class="bg-gray-100 p-4 rounded">
              <p class="text-xs font-mono mb-2">Option B: Go around community</p>
              <ul class="text-xs space-y-1">
                <li>• 5 edges × 15 miles each × factor 1.0 = weight 75</li>
                <li>• Keeps minority community intact</li>
              </ul>
            </div>
          </div>

          <div class="card bg-green-50 border-4 border-green-500">
            <h3 class="text-xl font-heading font-bold mb-4 text-green-600">METIS Chooses Option A!</h3>
            <p class="text-sm mb-4">
              Even though Option A has MORE edges, its total weight (15) is much lower than
              Option B (75) because of the minority factor.
            </p>
            <div class="bg-white p-4 rounded">
              <h4 class="font-bold text-sm mb-2 text-green-600">Result:</h4>
              <ul class="text-xs space-y-2">
                <li>✅ Minority community stays together (representation)</li>
                <li>✅ Boundary still reasonably short (compactness)</li>
                <li>✅ METIS finds this naturally - no constraints!</li>
              </ul>
            </div>
          </div>
        </div>

        <div class="card bg-gradient-to-r from-green-500 to-blue-500 text-white">
          <h3 class="text-2xl font-heading font-bold mb-4">The Magic</h3>
          <p class="text-xl leading-relaxed">
            METIS doesn't know about the VRA or minority representation - it just minimizes
            weighted edge cut like always. But by setting the weights cleverly, we
            <strong>encode our goals into the graph itself</strong>!
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- The Results: 137 MM Districts -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center text-yellow-600">
          The Results: 137 MM Districts!
        </h2>

        <div class="text-center mb-12">
          <div class="inline-block bg-gradient-to-r from-yellow-100 to-orange-100 rounded-2xl p-8">
            <div class="text-7xl font-black text-yellow-600 mb-4">137</div>
            <p class="text-2xl font-heading font-bold">
              Majority-Minority Districts
            </p>
            <p class="text-gray-600 mt-2">
              vs 68 enacted (2020) - that's <strong>+69 additional districts</strong>!
            </p>
          </div>
        </div>

        <p class="text-story text-center mb-12">
          The edge-factor approach creates <strong>twice as many MM districts</strong> as enacted
          plans, while maintaining high compactness. It's not a trade-off - it's a win-win!
        </p>

        <FigureCard
          src="/figures/chapter6/figure2_compactness_tradeoff.png"
          alt="Compactness vs MM districts trade-off curve"
          paper-link="/papers/04_multi_vs_edge.pdf"
        >
          <strong>The Trade-off Curve:</strong> This chart shows compactness (y-axis) vs number
          of MM districts (x-axis). Edge-factor weighting (purple) achieves high compactness
          while creating 137 MM districts - the best of both worlds!
        </FigureCard>

        <div class="grid md:grid-cols-3 gap-6 mt-12">
          <div class="text-center">
            <div class="text-5xl font-black text-gray-400 mb-2">68</div>
            <h4 class="font-heading font-bold text-gray-600">Enacted Plans</h4>
            <p class="text-sm text-gray-500">2020 redistricting</p>
          </div>

          <div class="text-center">
            <div class="text-6xl mb-2">→</div>
            <p class="text-sm text-gray-500 font-bold">+102% More Districts</p>
          </div>

          <div class="text-center">
            <div class="text-5xl font-black text-yellow-600 mb-2">137</div>
            <h4 class="font-heading font-bold text-yellow-600">Edge-Factor</h4>
            <p class="text-sm text-gray-600">Algorithmic approach</p>
          </div>
        </div>
      </div>
    </ScrollSection>

    <!-- State Examples -->
    <ScrollSection bg-class="chapter-6-bg">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center text-yellow-600">
          Success Stories: Texas, California, Florida
        </h2>

        <p class="text-story text-center mb-12">
          The largest gains come from states with substantial minority populations concentrated
          in urban areas. Let's look at three examples:
        </p>

        <div class="grid md:grid-cols-3 gap-6 mb-12">
          <ConceptCard color="yellow" icon="⭐">
            <template #title>Texas (38 districts)</template>
            <p class="text-sm mb-2">
              <strong>Minority population:</strong> 40% Hispanic, 12% Black
            </p>
            <ul class="text-xs space-y-1">
              <li>• Enacted: 10 MM districts (26%)</li>
              <li>• Edge-factor: 18 MM districts (47%)</li>
              <li>• <strong>+8 districts!</strong></li>
            </ul>
          </ConceptCard>

          <ConceptCard color="yellow" icon="🌟">
            <template #title>California (52 districts)</template>
            <p class="text-sm mb-2">
              <strong>Minority population:</strong> 39% Hispanic, 6% Black
            </p>
            <ul class="text-xs space-y-1">
              <li>• Enacted: 14 MM districts (27%)</li>
              <li>• Edge-factor: 22 MM districts (42%)</li>
              <li>• <strong>+8 districts!</strong></li>
            </ul>
          </ConceptCard>

          <ConceptCard color="yellow" icon="💫">
            <template #title>Florida (28 districts)</template>
            <p class="text-sm mb-2">
              <strong>Minority population:</strong> 26% Hispanic, 16% Black
            </p>
            <ul class="text-xs space-y-1">
              <li>• Enacted: 6 MM districts (21%)</li>
              <li>• Edge-factor: 11 MM districts (39%)</li>
              <li>• <strong>+5 districts!</strong></li>
            </ul>
          </ConceptCard>
        </div>

        <div class="card bg-gradient-to-r from-yellow-500 to-orange-500 text-white">
          <h3 class="text-2xl font-heading font-bold mb-4">The Pattern</h3>
          <p class="text-lg leading-relaxed">
            In every state with significant minority population, edge-factor weighting creates
            <strong>substantially more MM districts</strong> than enacted plans, while maintaining
            or improving compactness. Geography enables fairness when the algorithm respects
            community boundaries!
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- Compactness Maintained -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center text-yellow-600">
          Compactness: Still Excellent!
        </h2>

        <p class="text-story text-center mb-12">
          Creating 137 MM districts sounds great, but did we sacrifice compactness?
          <strong>No!</strong> Edge-factor districts remain highly compact.
        </p>

        <div class="grid md:grid-cols-3 gap-6 mb-12">
          <div class="card bg-white text-center">
            <h4 class="text-lg font-heading font-bold mb-2">Baseline</h4>
            <div class="text-4xl font-black text-gray-400 mb-2">0.28</div>
            <p class="text-sm text-gray-500">Polsby-Popper score</p>
            <p class="text-xs text-gray-400 mt-2">68 MM districts</p>
          </div>

          <div class="card bg-purple-50 text-center border-2 border-purple-300">
            <h4 class="text-lg font-heading font-bold mb-2 text-schoolhouse-purple">Geographic Only</h4>
            <div class="text-4xl font-black text-schoolhouse-purple mb-2">0.44</div>
            <p class="text-sm text-gray-600">Polsby-Popper score</p>
            <p class="text-xs text-gray-600 mt-2">80 MM districts</p>
          </div>

          <div class="card bg-yellow-50 text-center border-4 border-yellow-500">
            <h4 class="text-lg font-heading font-bold mb-2 text-yellow-600">Edge-Factor ✨</h4>
            <div class="text-4xl font-black text-yellow-600 mb-2">0.41</div>
            <p class="text-sm text-gray-600">Polsby-Popper score</p>
            <p class="text-xs font-bold text-yellow-600 mt-2">137 MM districts</p>
          </div>
        </div>

        <div class="card bg-gradient-to-r from-purple-100 to-yellow-100 border-l-4 border-yellow-500">
          <h3 class="text-2xl font-heading font-bold mb-3 text-yellow-600">The Sweet Spot</h3>
          <p class="text-story">
            Edge-factor weighting achieves <strong>0.41 compactness</strong> (just 7% below
            pure geographic weighting) while creating <strong>71% more MM districts</strong>!
            That's an incredible trade-off - we give up almost nothing to gain huge representation benefits.
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- Why It Works -->
    <ScrollSection bg-class="chapter-6-bg">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center text-yellow-600">
          Why Does This Work So Well?
        </h2>

        <div class="grid md:grid-cols-2 gap-8">
          <ConceptCard color="yellow" icon="🏙️">
            <template #title>Geographic Concentration</template>
            <p class="text-sm">
              Minority communities are often <strong>naturally clustered</strong> in urban areas.
              Keeping them together doesn't require stretching districts across the state -
              it just means drawing boundaries that respect existing communities.
            </p>
          </ConceptCard>

          <ConceptCard color="yellow" icon="🎯">
            <template #title>Aligned Incentives</template>
            <p class="text-sm">
              Compactness and community representation often <strong>point the same direction</strong>!
              A compact district that follows natural boundaries is likely to keep communities
              together. The goals complement each other more often than they conflict.
            </p>
          </ConceptCard>

          <ConceptCard color="yellow" icon="📐">
            <template #title>Smart Optimization</template>
            <p class="text-sm">
              METIS is incredibly good at finding optimal solutions. By encoding both goals
              into edge weights, we let METIS find the <strong>perfect balance</strong> between
              compactness and representation automatically.
            </p>
          </ConceptCard>

          <ConceptCard color="yellow" icon="✅">
            <template #title>No Hard Constraints</template>
            <p class="text-sm">
              Unlike the multi-constraint approach (Chapter 5), edge-factor weighting uses
              <strong>soft incentives</strong>. This gives METIS flexibility to find creative
              solutions that satisfy both goals where possible.
            </p>
          </ConceptCard>
        </div>
      </div>
    </ScrollSection>

    <!-- Key Takeaway -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-3xl mx-auto text-center">
        <div class="inline-block bg-white rounded-2xl shadow-xl p-8 md:p-12">
          <div class="text-6xl mb-6">💡</div>
          <h2 class="text-4xl font-heading font-black mb-6 text-yellow-600">
            Key Takeaway
          </h2>
          <p class="text-2xl font-heading leading-relaxed">
            By making minority community boundaries <span class="text-yellow-600 font-black">"cheaper"</span>
            to cross, we keep communities together <span class="text-yellow-600 font-black">naturally</span>!
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- The Journey Complete -->
    <ScrollSection bg-class="chapter-6-bg">
      <div class="max-w-4xl mx-auto text-center">
        <h2 class="text-4xl font-heading font-bold mb-8">The Journey Complete</h2>

        <div class="grid md:grid-cols-3 lg:grid-cols-6 gap-4 mb-12">
          <div class="card bg-blue-50 text-center">
            <div class="text-3xl mb-2">🗺️</div>
            <h4 class="text-sm font-heading font-bold text-schoolhouse-blue">Ch 1</h4>
            <p class="text-xs">Tracts → Graphs</p>
          </div>

          <div class="card bg-orange-50 text-center">
            <div class="text-3xl mb-2">✂️</div>
            <h4 class="text-sm font-heading font-bold text-schoolhouse-orange">Ch 2</h4>
            <p class="text-xs">Split in Two</p>
          </div>

          <div class="card bg-green-50 text-center">
            <div class="text-3xl mb-2">🌳</div>
            <h4 class="text-sm font-heading font-bold text-schoolhouse-green">Ch 3</h4>
            <p class="text-xs">Recursion</p>
          </div>

          <div class="card bg-purple-50 text-center">
            <div class="text-3xl mb-2">📐</div>
            <h4 class="text-sm font-heading font-bold text-schoolhouse-purple">Ch 4</h4>
            <p class="text-xs">Compactness</p>
          </div>

          <div class="card bg-red-50 text-center">
            <div class="text-3xl mb-2">🗳️</div>
            <h4 class="text-sm font-heading font-bold text-schoolhouse-red">Ch 5</h4>
            <p class="text-xs">VRA</p>
          </div>

          <div class="card bg-yellow-50 text-center border-4 border-yellow-500">
            <div class="text-3xl mb-2">⚖️</div>
            <h4 class="text-sm font-heading font-bold text-yellow-600">Ch 6</h4>
            <p class="text-xs font-bold">Together!</p>
          </div>
        </div>

        <div class="card bg-gradient-schoolhouse text-white mb-12">
          <h3 class="text-3xl font-heading font-bold mb-6">What We've Learned</h3>
          <div class="grid md:grid-cols-2 gap-6 text-left">
            <ul class="space-y-3">
              <li>✅ Census tracts → graph networks</li>
              <li>✅ METIS partitions graphs optimally</li>
              <li>✅ Recursion creates any number of districts</li>
            </ul>
            <ul class="space-y-3">
              <li>✅ Edge-weighting improves compactness 56%</li>
              <li>✅ 42% threshold enables proportional representation</li>
              <li>✅ Edge-factor achieves both goals: 137 MM districts!</li>
            </ul>
          </div>
        </div>

        <div class="mb-8">
          <h3 class="text-2xl font-heading font-bold mb-4">Want to Dive Deeper?</h3>
          <p class="text-lg text-gray-700 mb-6">
            Explore the detailed research papers that back up everything you've learned!
          </p>
          <router-link to="/research" class="btn btn-primary btn-lg">
            View Research Papers →
          </router-link>
        </div>

        <div class="text-gray-600 italic">
          <p>Thank you for joining us on this visual journey through algorithmic redistricting!</p>
          <p class="mt-2">Fair districts. Fair representation. Fair democracy.</p>
        </div>
      </div>
    </ScrollSection>
  </div>
</template>

<script setup>
import Hero from '@/components/Hero.vue'
import ScrollSection from '@/components/ScrollSection.vue'
import FigureCard from '@/components/FigureCard.vue'
import ConceptCard from '@/components/ConceptCard.vue'
</script>
