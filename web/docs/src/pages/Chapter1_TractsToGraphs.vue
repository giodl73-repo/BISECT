<template>
  <div class="chapter-1">
    <!-- Hero -->
    <Hero :chapter="1" color="blue">
      <template #title>
        From Tracts<br />to Graphs
      </template>
      <template #subtitle>
        How census tracts become connected networks
      </template>
    </Hero>

    <!-- Story Hook -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <div class="text-center mb-12">
          <h2 class="text-4xl font-heading font-bold mb-6 chapter-1-accent">
            Imagine a State as a Giant Puzzle
          </h2>
          <p class="text-story">
            The United States Census divides the country into approximately <strong>73,000 tiny pieces</strong>
            called census tracts. Each tract is like a neighborhood—usually between 1,200 and 8,000 people.
          </p>
        </div>

        <FigureCard
          src="/figures/chapter1/alabama_tracts.png"
          alt="Census tracts in Alabama"
        >
          Alabama's 1,181 census tracts. Each colored region represents one tract—
          these are the building blocks we'll use to create 7 congressional districts.
        </FigureCard>
      </div>
    </ScrollSection>

    <!-- What Are Census Tracts? -->
    <ScrollSection bg-class="chapter-1-bg">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-1-accent">
          What Are Census Tracts?
        </h2>

        <div class="grid md:grid-cols-3 gap-6 mb-12">
          <ConceptCard color="blue" icon="🏘️">
            <template #title>Neighborhoods</template>
            <p>
              Census tracts are small, permanent geographic areas designed to be relatively
              homogeneous with respect to population characteristics, economic status, and living conditions.
            </p>
          </ConceptCard>

          <ConceptCard color="blue" icon="👥">
            <template #title>Population</template>
            <p>
              Each tract typically contains 1,200-8,000 people, with an optimal size of about 4,000.
              This makes them small enough to be meaningful but large enough to be statistically useful.
            </p>
          </ConceptCard>

          <ConceptCard color="blue" icon="🗺️">
            <template #title>Geography</template>
            <p>
              Tracts are designed to follow visible features like roads, rivers, and railroad tracks.
              They don't change much over time, making them perfect for long-term analysis.
            </p>
          </ConceptCard>
        </div>

        <div class="card bg-blue-50 border-l-4 border-schoolhouse-blue">
          <h3 class="text-2xl font-heading font-bold mb-3 chapter-1-accent">Key Insight</h3>
          <p class="text-story">
            Census tracts are the <strong>atomic units</strong> of redistricting. We never split a tract—
            each one stays whole and gets assigned to exactly one congressional district.
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- From Geography to Networks -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-1-accent">
          From Geography to Networks
        </h2>

        <p class="text-story text-center mb-12">
          Here's where the magic begins. Instead of thinking about tracts as <em>geographic shapes</em>,
          we can think of them as <strong>nodes in a network</strong>. Two tracts are connected if they
          share a border.
        </p>

        <div class="grid md:grid-cols-2 gap-8 mb-12">
          <div class="card">
            <h3 class="text-2xl font-heading font-bold mb-4 chapter-1-accent">Geographic View</h3>
            <div class="bg-blue-100 rounded-lg p-8 text-center">
              <p class="text-gray-600 italic">[Interactive map visualization will go here]</p>
              <p class="text-sm text-gray-500 mt-4">Click on Vermont to see its tracts</p>
            </div>
          </div>

          <div class="card">
            <h3 class="text-2xl font-heading font-bold mb-4 chapter-1-accent">Network View</h3>
            <div class="bg-purple-100 rounded-lg p-8 text-center">
              <p class="text-gray-600 italic">[Interactive graph visualization will go here]</p>
              <p class="text-sm text-gray-500 mt-4">Same tracts, different perspective</p>
            </div>
          </div>
        </div>

        <div class="card bg-gradient-blue-purple text-white">
          <h3 class="text-2xl font-heading font-bold mb-3">Why This Matters</h3>
          <p class="text-lg leading-relaxed">
            By converting geography into a <strong>graph</strong> (nodes + edges), we can use powerful
            algorithms from computer science to solve the redistricting problem. This is the foundation
            of everything that follows!
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- Building the Graph -->
    <ScrollSection bg-class="chapter-1-bg">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-1-accent">
          Building the Adjacency Graph
        </h2>

        <div class="mb-12">
          <p class="text-story text-center mb-8">
            Creating the graph is a three-step process:
          </p>

          <div class="grid md:grid-cols-3 gap-6">
            <ConceptCard color="blue" icon="1️⃣">
              <template #title>Load Tracts</template>
              <p>
                Read census tract boundaries from TIGER/Line shapefiles. Each tract has a unique
                identifier (GEOID) and a polygon geometry.
              </p>
            </ConceptCard>

            <ConceptCard color="blue" icon="2️⃣">
              <template #title>Find Neighbors</template>
              <p>
                For each tract, identify all tracts that share a boundary. Two polygons are neighbors
                if they touch along an edge (not just at a point).
              </p>
            </ConceptCard>

            <ConceptCard color="blue" icon="3️⃣">
              <template #title>Create Edges</template>
              <p>
                Build an edge list: each row represents a connection between two neighboring tracts.
                This is what we feed to the algorithm!
              </p>
            </ConceptCard>
          </div>
        </div>

        <FigureCard
          src="/figures/chapter1/adjacency_process.png"
          alt="Adjacency graph construction process"
          paper-link="/papers/03_combined_recursive_bisection.pdf"
        >
          The adjacency detection process. Tracts (left) are analyzed for shared boundaries (middle),
          resulting in an edge list (right) that defines the graph structure.
        </FigureCard>
      </div>
    </ScrollSection>

    <!-- Real Example: Alabama -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-1-accent">
          Real Example: Alabama
        </h2>

        <div class="grid md:grid-cols-2 gap-8 mb-8">
          <div>
            <h3 class="text-2xl font-heading font-bold mb-4">The Numbers</h3>
            <ul class="space-y-3 text-story">
              <li><strong>1,181 census tracts</strong> cover the entire state</li>
              <li><strong>~3,200 edges</strong> connect neighboring tracts</li>
              <li><strong>7 congressional districts</strong> to create</li>
              <li><strong>5.03 million people</strong> total population (2020 census)</li>
            </ul>
          </div>

          <div>
            <h3 class="text-2xl font-heading font-bold mb-4">The Challenge</h3>
            <p class="text-story mb-4">
              Here's where it gets interesting: <strong>7 districts can't be split evenly!</strong>
              This means we need asymmetric splits like [3,4] or [4,3]. You'll see this play out
              in Chapters 2 and 3.
            </p>
            <p class="text-sm text-gray-600 italic">
              Average degree: ~5.4 neighbors per tract<br />
              Target population per district: ~718,700 people
            </p>
          </div>
        </div>

        <div class="card bg-gradient-green-blue text-white">
          <h3 class="text-2xl font-heading font-bold mb-4">Follow Alabama's Journey</h3>
          <p class="text-lg mb-4">
            We'll use Alabama as our example throughout all 6 chapters, showing how 1,181 census
            tracts become 7 congressional districts through recursive bisection.
          </p>
          <div class="bg-white/20 rounded-lg p-8">
            <p class="text-sm mb-2">Coming in Chapter 2:</p>
            <p class="font-bold">How do we split Alabama's graph into 2 balanced regions?</p>
          </div>
        </div>
      </div>
    </ScrollSection>

    <!-- Key Takeaway -->
    <ScrollSection bg-class="chapter-1-bg">
      <div class="max-w-3xl mx-auto text-center">
        <div class="inline-block bg-white rounded-2xl shadow-xl p-8 md:p-12">
          <div class="text-6xl mb-6">💡</div>
          <h2 class="text-4xl font-heading font-black mb-6 chapter-1-accent">
            Key Takeaway
          </h2>
          <p class="text-2xl font-heading leading-relaxed">
            A state isn't just a shape—it's a <span class="underline-blue">network of connected neighborhoods</span>!
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- Next Chapter -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto text-center">
        <h2 class="text-4xl font-heading font-bold mb-6">What's Next?</h2>
        <p class="text-xl text-gray-700 mb-8">
          Now that we have a graph, how do we split it into two balanced parts?
          That's where METIS comes in...
        </p>
        <router-link to="/chapter-2" class="btn btn-secondary btn-lg">
          Continue to Chapter 2 →
        </router-link>
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
