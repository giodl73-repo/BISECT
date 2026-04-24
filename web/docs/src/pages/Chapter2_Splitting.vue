<template>
  <div class="chapter-2">
    <!-- Hero -->
    <Hero :chapter="2" color="orange">
      <template #title>
        Splitting<br />in Two
      </template>
      <template #subtitle>
        How to divide any region into 2 balanced parts
      </template>
    </Hero>

    <!-- Story Hook -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <div class="text-center mb-12">
          <h2 class="text-4xl font-heading font-bold mb-6 chapter-2-accent">
            The Birthday Cake Problem
          </h2>
          <p class="text-story">
            Imagine you need to cut a birthday cake <strong>perfectly in half</strong> so two people
            get exactly the same amount. Easy with a round cake, right? But what if the cake has a
            weird shape, like... <em>Alabama</em>?
          </p>
        </div>

        <div class="grid md:grid-cols-2 gap-8 mb-12">
          <ConceptCard color="orange" icon="🎂">
            <template #title>The Challenge</template>
            <p>
              Alabama has 5.03 million people across 1,181 tracts. We need to split it into
              <strong>2 regions</strong> that each have exactly half the population (±0.5%).
            </p>
          </ConceptCard>

          <ConceptCard color="orange" icon="✂️">
            <template #title>The Constraint</template>
            <p>
              Both regions must be <strong>contiguous</strong> (all tracts connected) and as
              <strong>compact</strong> as possible. No snaky tendrils reaching across the state!
            </p>
          </ConceptCard>
        </div>
      </div>
    </ScrollSection>

    <!-- The Wrong Way -->
    <ScrollSection bg-class="chapter-2-bg">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-2-accent">
          The Wrong Way to Split
        </h2>

        <div class="grid md:grid-cols-3 gap-6 mb-12">
          <div class="card bg-red-50 border-l-4 border-red-500">
            <h3 class="text-xl font-heading font-bold mb-3 text-red-600">❌ Random Line</h3>
            <p class="text-sm text-gray-700">
              Draw a line down the middle? Probably won't balance population (cities vs rural).
              Could cut through neighborhoods.
            </p>
          </div>

          <div class="card bg-red-50 border-l-4 border-red-500">
            <h3 class="text-xl font-heading font-bold mb-3 text-red-600">❌ County Borders</h3>
            <p class="text-sm text-gray-700">
              Follow county lines? Alabama has 67 counties of wildly different sizes. Won't balance.
            </p>
          </div>

          <div class="card bg-red-50 border-l-4 border-red-500">
            <h3 class="text-xl font-heading font-bold mb-3 text-red-600">❌ Trial & Error</h3>
            <p class="text-sm text-gray-700">
              Manually pick tracts until balanced? With 1,181 tracts, there are billions of
              possible combinations!
            </p>
          </div>
        </div>

        <div class="text-center">
          <p class="text-xl text-gray-700 italic">
            We need an algorithm that can find the perfect split <strong>automatically</strong>.
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- Enter METIS -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-2-accent">
          Enter METIS: The Graph Partitioner
        </h2>

        <p class="text-story text-center mb-12">
          Remember how we converted Alabama into a <strong>graph</strong> in Chapter 1? Now we can
          use a powerful algorithm called <strong>METIS</strong> to partition that graph into 2
          balanced parts.
        </p>

        <div class="grid md:grid-cols-2 gap-8 mb-12">
          <ConceptCard color="orange" icon="🧮">
            <template #title>What is METIS?</template>
            <p>
              METIS is a graph partitioning library from the University of Minnesota. It's been
              used for decades in supercomputing, circuit design, and scientific computing.
            </p>
          </ConceptCard>

          <ConceptCard color="orange" icon="⚡">
            <template #title>Why METIS?</template>
            <p>
              METIS can partition a graph with <strong>perfect population balance</strong> while
              minimizing the "edge cut" (the number of edges crossing the boundary).
            </p>
          </ConceptCard>
        </div>

        <div class="card bg-gradient-orange-red text-white">
          <h3 class="text-2xl font-heading font-bold mb-4">The METIS Magic Formula</h3>
          <div class="space-y-2 text-lg">
            <p>1. <strong>Input:</strong> Graph (nodes + edges) + target population per partition</p>
            <p>2. <strong>Constraint:</strong> Each partition within ±0.5% of target</p>
            <p>3. <strong>Objective:</strong> Minimize edges crossing the boundary</p>
            <p>4. <strong>Output:</strong> Label for each node (0 or 1)</p>
          </div>
        </div>
      </div>
    </ScrollSection>

    <!-- How METIS Works -->
    <ScrollSection bg-class="chapter-2-bg">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-2-accent">
          How METIS Works (Simplified)
        </h2>

        <div class="grid md:grid-cols-4 gap-4 mb-12">
          <ConceptCard color="orange" icon="1️⃣">
            <template #title>Coarsen</template>
            <p class="text-sm">
              Merge similar nodes together to create a smaller, simpler graph.
            </p>
          </ConceptCard>

          <ConceptCard color="orange" icon="2️⃣">
            <template #title>Partition</template>
            <p class="text-sm">
              Split the coarsened graph into 2 balanced parts using a greedy algorithm.
            </p>
          </ConceptCard>

          <ConceptCard color="orange" icon="3️⃣">
            <template #title>Uncoarsen</template>
            <p class="text-sm">
              Expand back to the original graph, maintaining the partition.
            </p>
          </ConceptCard>

          <ConceptCard color="orange" icon="4️⃣">
            <template #title>Refine</template>
            <p class="text-sm">
              Fine-tune by moving nodes across the boundary to improve the cut.
            </p>
          </ConceptCard>
        </div>

        <div class="card bg-blue-50 border-l-4 border-schoolhouse-blue">
          <h3 class="text-2xl font-heading font-bold mb-3 text-schoolhouse-blue">Key Insight</h3>
          <p class="text-story">
            METIS is <strong>fast</strong> (partitions Alabama in ~0.1 seconds) and
            <strong>smart</strong> (finds near-optimal solutions). It's the workhorse of our
            redistricting algorithm!
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- Interactive Split Simulator -->
    <ScrollSection bg-class="bg-gradient-to-br from-orange-50 to-red-50">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-2-accent">
          ⚡ Interactive: Run METIS Yourself
        </h2>

        <p class="text-story text-center mb-8">
          Click <strong>"Run METIS Split"</strong> to watch the algorithm split a state
          into two perfectly balanced regions!
        </p>

        <SplitSimulator
          title="METIS Split Simulator"
          description="See the 4-step METIS process in action"
        />

        <div class="mt-8 card bg-gradient-to-r from-orange-100 to-yellow-100">
          <h3 class="text-xl font-heading font-bold mb-3 text-schoolhouse-orange">
            🎯 What Just Happened?
          </h3>
          <p class="text-sm mb-4">
            METIS found the optimal split in milliseconds—minimizing edge cut while
            balancing population perfectly.
          </p>
          <p class="text-sm">
            The split isn't exactly 50-50 (42.8% vs 57.2%) because tracts can't be divided.
            The algorithm gets as close as possible.
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- Alabama's First Split -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-2-accent">
          Alabama's Journey Begins: The First Split
        </h2>

        <p class="text-story text-center mb-8">
          Remember Alabama from Chapter 1? Those 1,181 tracts we turned into a graph?
          Now watch METIS split them into 2 regions. Alabama needs <strong>7 districts total</strong>,
          so this first split creates [3 districts] and [4 districts].
        </p>

        <FigureCard
          src="/figures/chapter2/alabama_round_1_2_regions.png"
          alt="Alabama split into 2 regions"
          paper-link="/papers/03_combined_recursive_bisection.pdf"
        >
          <strong>Round 1:</strong> Alabama split into 2 regions. Region A (blue) has ~2.15 million
          people and will eventually become 3 districts. Region B (orange) has ~2.88 million people
          and will become 4 districts.
        </FigureCard>

        <div class="grid md:grid-cols-2 gap-8 mt-12">
          <div class="card bg-blue-50">
            <h3 class="text-xl font-heading font-bold mb-3 text-schoolhouse-blue">Region A (Blue)</h3>
            <ul class="space-y-2 text-sm text-gray-700">
              <li><strong>Population:</strong> ~2.15 million (42.8%)</li>
              <li><strong>Census tracts:</strong> ~505 tracts</li>
              <li><strong>Future districts:</strong> 3</li>
              <li><strong>Target per district:</strong> ~718,700</li>
            </ul>
          </div>

          <div class="card bg-orange-50">
            <h3 class="text-xl font-heading font-bold mb-3 text-schoolhouse-orange">Region B (Orange)</h3>
            <ul class="space-y-2 text-sm text-gray-700">
              <li><strong>Population:</strong> ~2.88 million (57.2%)</li>
              <li><strong>Census tracts:</strong> ~676 tracts</li>
              <li><strong>Future districts:</strong> 4</li>
              <li><strong>Target per district:</strong> ~718,700</li>
            </ul>
          </div>
        </div>
      </div>
    </ScrollSection>

    <!-- The Edge Cut -->
    <ScrollSection bg-class="chapter-2-bg">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-2-accent">
          Understanding the Edge Cut
        </h2>

        <p class="text-story text-center mb-8">
          The <strong>edge cut</strong> is the number of edges (tract boundaries) that cross
          from Region A to Region B. METIS tries to minimize this to keep districts compact.
        </p>

        <div class="grid md:grid-cols-3 gap-6">
          <ConceptCard color="orange" icon="📊">
            <template #title>Edge Cut</template>
            <p class="text-sm">
              Alabama's first split has an edge cut of ~180 edges (out of ~3,200 total edges in
              the graph).
            </p>
          </ConceptCard>

          <ConceptCard color="orange" icon="🎯">
            <template #title>Why It Matters</template>
            <p class="text-sm">
              Fewer edges crossing = more compact regions. A low edge cut means the boundary
              follows natural geographic divisions.
            </p>
          </ConceptCard>

          <ConceptCard color="orange" icon="✅">
            <template #title>Is This Good?</template>
            <p class="text-sm">
              Yes! 180/3,200 = 5.6% edge cut rate. That's excellent. Most splits have 10-20%
              edge cut rates.
            </p>
          </ConceptCard>
        </div>
      </div>
    </ScrollSection>

    <!-- The Asymmetry Challenge -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-2-accent">
          The "7 Districts" Challenge
        </h2>

        <p class="text-story text-center mb-8">
          Here's where Alabama gets interesting! <strong>7 is an odd number</strong>, so we
          can't split it evenly. We need to split it as <strong>[3,4]</strong> or
          <strong>[4,3]</strong>.
        </p>

        <div class="card bg-gradient-orange-red text-white mb-8">
          <h3 class="text-2xl font-heading font-bold mb-4">Why [3,4] and not [3.5, 3.5]?</h3>
          <p class="text-lg leading-relaxed">
            Districts must be <strong>whole units</strong>! You can't have 3.5 congressional
            representatives. So for 7 total districts, we split into regions that will eventually
            become 3 districts and 4 districts.
          </p>
        </div>

        <div class="text-center">
          <p class="text-xl text-gray-700 mb-4">
            <strong>Population Ratio:</strong> 3:4 = 42.9% : 57.1%
          </p>
          <p class="text-story">
            That's why Region A has 42.8% of the population and Region B has 57.2%!
            The algorithm knew this from the beginning.
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- Interactive Demo Placeholder -->
    <ScrollSection bg-class="chapter-2-bg">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-2-accent">
          Try It Yourself! (Coming in Phase 4)
        </h2>

        <div class="card bg-white">
          <div class="bg-orange-100 rounded-lg p-12 text-center">
            <p class="text-xl font-heading text-gray-700 mb-4">
              🎮 Interactive Split Simulator
            </p>
            <p class="text-gray-600">
              In Phase 4, you'll be able to drag a dividing line across Alabama and see how
              population imbalance changes in real-time. Then click "Auto-Split" and watch
              METIS find the optimal cut!
            </p>
          </div>
        </div>
      </div>
    </ScrollSection>

    <!-- Key Takeaway -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-3xl mx-auto text-center">
        <div class="inline-block bg-white rounded-2xl shadow-2xl p-8 md:p-12 border-4 border-schoolhouse-orange animate-slide-up">
          <div class="text-7xl mb-6 animate-bounce-subtle">💡</div>
          <h2 class="text-5xl font-heading font-black mb-8 chapter-2-accent">
            Key Takeaway
          </h2>
          <p class="text-3xl font-heading leading-relaxed font-bold">
            You can split <span class="underline-orange bg-orange-100 px-2">any state into 2 perfectly balanced parts</span>
            using graph partitioning!
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- Cliffhanger -->
    <ScrollSection bg-class="bg-gradient-to-br from-orange-500 to-green-500">
      <div class="max-w-3xl mx-auto text-center text-white">
        <h2 class="text-4xl font-heading font-bold mb-6">
          Wait—Alabama needs 7 districts, not 2!
        </h2>
        <p class="text-2xl leading-relaxed">
          METIS can split a state in half. But what if we need 7 regions? Or 52?
          <strong>Here's where the magic happens...</strong>
        </p>
        <div class="mt-8 text-lg opacity-90">
          <p>The secret: keep splitting. Like a family tree. 🌳</p>
        </div>
      </div>
    </ScrollSection>

    <!-- Chapter Navigation -->
    <ChapterNavigation :current-chapter="2" />
  </div>
</template>

<script setup>
import Hero from '@/components/Hero.vue'
import ScrollSection from '@/components/ScrollSection.vue'
import FigureCard from '@/components/FigureCard.vue'
import ConceptCard from '@/components/ConceptCard.vue'
import SplitSimulator from '@/components/SplitSimulator.vue'
import ChapterNavigation from '@/components/ChapterNavigation.vue'
</script>
