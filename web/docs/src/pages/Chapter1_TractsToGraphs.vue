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

    <!-- Why Graphs? The Key Insight -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-1-accent">
          Why Turn Geography into Graphs?
        </h2>

        <p class="text-story text-center mb-8">
          Redistricting is a complex spatial problem. But here's the insight: With most programming problems,
          the first step is to <strong>solve the simplest version</strong>. For redistricting, that's splitting
          a region into two equal-population parts.
        </p>

        <div class="card bg-blue-50 border-l-4 border-schoolhouse-blue mb-12">
          <h3 class="text-2xl font-heading font-bold mb-3 chapter-1-accent">The Base Case</h3>
          <p class="text-story mb-4">
            Why start with the two-way split? Because it's the <strong>simplest redistricting problem</strong>
            that captures all the key challenges:
          </p>
          <ul class="space-y-2 text-story">
            <li>• Balancing population between two parts</li>
            <li>• Maintaining geographic compactness</li>
            <li>• Ensuring both parts stay connected (contiguity)</li>
          </ul>
          <p class="text-story mt-4">
            Once we master this building block, scaling up to 7, 8, or even 52 districts becomes straightforward
            through repetition (recursion—more on this in Chapter 3!).
          </p>
        </div>

        <div class="grid md:grid-cols-3 gap-6 mb-12">
          <ConceptCard color="blue" icon="🌐">
            <template #title>Preserves Geography</template>
            <p class="text-sm">
              Adjacent tracts in the real world remain connected in the graph. The network structure
              captures geographic relationships.
            </p>
          </ConceptCard>

          <ConceptCard color="blue" icon="🔗">
            <template #title>Ensures Contiguity</template>
            <p class="text-sm">
              Any partition must form connected components. If two tracts are in the same district,
              there must be a path of edges between them.
            </p>
          </ConceptCard>

          <ConceptCard color="blue" icon="⚡">
            <template #title>Enables Algorithms</template>
            <p class="text-sm">
              Graph partitioning algorithms from engineering and computer science can now solve
              our redistricting problem!
            </p>
          </ConceptCard>
        </div>
      </div>
    </ScrollSection>

    <!-- Real Example: Minneapolis -->
    <ScrollSection bg-class="chapter-1-bg">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-1-accent">
          Real Example: 12 Tracts in Minneapolis
        </h2>

        <p class="text-story text-center mb-12">
          Let's see this transformation with a real example: twelve census tracts from Minneapolis, Minnesota.
          These aren't hypothetical shapes—they're actual tract boundaries that follow streets, rivers,
          and neighborhoods.
        </p>

        <div class="card bg-white mb-12">
          <h3 class="text-2xl font-heading font-bold mb-6 text-center chapter-1-accent">
            The Transformation
          </h3>

          <div class="grid md:grid-cols-2 gap-8">
            <!-- Left: Geographic View -->
            <div>
              <h4 class="text-xl font-heading font-bold mb-4 text-center">Geographic View</h4>
              <div class="bg-blue-50 rounded-lg p-6">
                <div class="aspect-square bg-blue-100 rounded flex items-center justify-center mb-4">
                  <div class="text-gray-400 text-center">
                    <p class="text-sm">12 real census tracts</p>
                    <p class="text-xs mt-2">Hennepin County, Minnesota (2010)</p>
                  </div>
                </div>
                <p class="text-sm text-gray-700">
                  Each colored region is a census tract—small areas of 1,200-8,000 people.
                  Boundaries follow streets, rivers, and railroad tracks.
                </p>
              </div>
            </div>

            <!-- Right: Network View -->
            <div>
              <h4 class="text-xl font-heading font-bold mb-4 text-center">Network View</h4>
              <div class="bg-purple-50 rounded-lg p-6">
                <div class="aspect-square bg-purple-100 rounded flex items-center justify-center mb-4">
                  <div class="text-gray-400 text-center">
                    <p class="text-sm">Abstract graph</p>
                    <p class="text-xs mt-2">12 nodes, 19 edges</p>
                  </div>
                </div>
                <p class="text-sm text-gray-700">
                  Each tract becomes a <strong>vertex</strong> (node) sized by population.
                  Adjacent tracts connect via <strong>edges</strong>.
                </p>
              </div>
            </div>
          </div>
        </div>

        <div class="card bg-gradient-blue-purple text-white">
          <h3 class="text-2xl font-heading font-bold mb-4">What Just Happened?</h3>
          <div class="grid md:grid-cols-2 gap-6 text-lg leading-relaxed">
            <div>
              <p class="mb-2"><strong>Before:</strong></p>
              <p>Complex geographic shapes with irregular boundaries, different sizes,
              and spatial relationships that are hard to compute with.</p>
            </div>
            <div>
              <p class="mb-2"><strong>After:</strong></p>
              <p>Clean mathematical structure: nodes (tracts) and edges (borders). Now we can
              use powerful graph algorithms!</p>
            </div>
          </div>
        </div>
      </div>
    </ScrollSection>

    <!-- Building the Graph: Technical Details -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-1-accent">
          Building the Graph
        </h2>

        <p class="text-story text-center mb-12">
          How do we actually convert tracts into a graph? It's a three-step process:
        </p>

        <div class="grid md:grid-cols-3 gap-6 mb-12">
          <ConceptCard color="blue" icon="1️⃣">
            <template #title>Load Geometries</template>
            <p class="text-sm">
              Read census tract boundaries from TIGER/Line shapefiles. Each tract has a unique
              identifier (GEOID) and polygon geometry.
            </p>
          </ConceptCard>

          <ConceptCard color="blue" icon="2️⃣">
            <template #title>Detect Adjacency</template>
            <p class="text-sm">
              For each tract, find all neighbors that share a boundary. Two polygons are adjacent
              if they touch along an edge (not just at a single point).
            </p>
          </ConceptCard>

          <ConceptCard color="blue" icon="3️⃣">
            <template #title>Create Edge List</template>
            <p class="text-sm">
              Build the graph: each vertex represents a tract (with population size), each edge
              represents a shared border between neighbors.
            </p>
          </ConceptCard>
        </div>

        <div class="card bg-blue-50">
          <h3 class="text-xl font-heading font-bold mb-4 chapter-1-accent">
            The Result: A Graph Ready for Partitioning
          </h3>
          <p class="text-story mb-4">
            For our Minneapolis example with 12 tracts:
          </p>
          <ul class="space-y-2 text-story">
            <li>• <strong>12 vertices</strong>: One for each census tract</li>
            <li>• <strong>19 edges</strong>: Connections between neighboring tracts</li>
            <li>• <strong>Population data</strong>: Each vertex sized by tract population</li>
            <li>• <strong>Contiguity preserved</strong>: Connected in geography = connected in graph</li>
          </ul>
          <p class="text-story mt-4">
            This graph can now be fed to METIS (a graph partitioning algorithm) to split it into
            two balanced parts. We'll see that in Chapter 2!
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- Interactive Graph Builder -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-1-accent">
          🎨 Interactive: Watch Tracts Become a Graph
        </h2>

        <p class="text-story text-center mb-8">
          Click <strong>"Show Tracts"</strong> to see census tracts appear, then
          <strong>"Add Edges"</strong> to watch neighbors connect!
        </p>

        <GraphBuilder
          title="From Geography to Mathematics"
          description="Step through the transformation process"
          state-name="Alabama (Simplified)"
          :tract-count="30"
        />

        <div class="mt-8 grid md:grid-cols-2 gap-6">
          <div class="card bg-blue-50">
            <h3 class="text-lg font-heading font-bold mb-3 text-schoolhouse-blue">
              👀 What to Watch
            </h3>
            <ul class="text-sm space-y-2">
              <li>• Tracts appear as blue nodes</li>
              <li>• Edges connect between neighbors</li>
              <li>• Toggle between geographic and network views</li>
              <li>• Graph density shows avg connections per tract</li>
            </ul>
          </div>

          <div class="card bg-orange-50">
            <h3 class="text-lg font-heading font-bold mb-3 text-schoolhouse-orange">
              🔍 Key Insight
            </h3>
            <p class="text-sm">
              This is a simplified version with 30 tracts. Real Alabama has 1,181 tracts
              and ~3,200 edges! The algorithm works the same way regardless of size.
            </p>
          </div>
        </div>
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
import GraphBuilder from '@/components/GraphBuilder.vue'
</script>
