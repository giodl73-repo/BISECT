<template>
  <div class="home-page">
    <!-- Hero Section -->
    <section class="section-hero bg-gradient-schoolhouse relative overflow-hidden">
      <div class="container-custom text-center text-white relative z-10">
        <h1 class="text-hero mb-6 animate-fade-in text-shadow-strong">
          Algorithmic<br />Redistricting
        </h1>
        <p class="text-subhero mb-12 animate-slide-up text-white/90">
          A Visual Journey Through Fair Districts
        </p>

        <div class="animate-slide-up" style="animation-delay: 0.2s">
          <router-link to="/chapter-1" class="btn btn-lg bg-white text-schoolhouse-blue hover:bg-gray-100">
            Start the Journey →
          </router-link>
        </div>

        <!-- Animated Visual Placeholder -->
        <div class="mt-16 animate-fade-in" style="animation-delay: 0.4s">
          <div class="bg-white/10 backdrop-blur-sm rounded-2xl p-8 max-w-4xl mx-auto">
            <p class="text-lg text-white/80 italic">
              "How do we divide a state into equal districts without gerrymandering?<br />
              Let's find out together!"
            </p>
          </div>
        </div>
      </div>

      <!-- Scroll Indicator -->
      <div class="absolute bottom-8 left-1/2 transform -translate-x-1/2 animate-bounce-subtle">
        <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3" />
        </svg>
      </div>
    </section>

    <!-- What You'll Learn Section -->
    <ScrollSection bg-class="bg-white">
      <div class="text-center mb-16">
        <h2 class="text-5xl font-heading font-bold mb-4">What You'll Learn</h2>
        <p class="text-xl text-gray-600">
          Six chapters that make redistricting algorithms accessible and fun
        </p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
        <ConceptCard v-for="chapter in chapters" :key="chapter.number" :color="chapter.color" :icon="chapter.icon">
          <template #title>{{ chapter.title }}</template>
          <p>{{ chapter.description }}</p>
          <template #action>
            <router-link :to="`/chapter-${chapter.number}`" class="text-sm font-semibold hover:underline">
              Read Chapter {{ chapter.number }} →
            </router-link>
          </template>
        </ConceptCard>
      </div>
    </ScrollSection>

    <!-- The Challenge Section -->
    <ScrollSection bg-class="bg-gradient-to-br from-blue-50 to-purple-50">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-5xl font-heading font-bold mb-8 text-center">The Challenge</h2>

        <div class="grid md:grid-cols-2 gap-8 mb-12">
          <div class="card bg-red-50 border-l-4 border-red-500">
            <h3 class="text-2xl font-heading font-bold mb-3 text-red-600">The Problem</h3>
            <ul class="space-y-2 text-gray-700">
              <li>• 435 congressional districts across 50 states</li>
              <li>• Each district must have equal population</li>
              <li>• Traditional redistricting can be manipulated</li>
              <li>• Gerrymandering undermines fair representation</li>
            </ul>
          </div>

          <div class="card bg-green-50 border-l-4 border-green-500">
            <h3 class="text-2xl font-heading font-bold mb-3 text-green-600">The Solution</h3>
            <ul class="space-y-2 text-gray-700">
              <li>• Algorithmic approach using recursive bisection</li>
              <li>• Purely mathematical (no political data)</li>
              <li>• Creates compact, contiguous districts</li>
              <li>• Respects Voting Rights Act requirements</li>
            </ul>
          </div>
        </div>

        <div class="text-center">
          <p class="text-xl text-gray-700 italic">
            "Can computers create fairer districts than humans?<br />
            The answer might surprise you."
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- Key Results Section -->
    <ScrollSection bg-class="bg-white">
      <div class="text-center mb-16">
        <h2 class="text-5xl font-heading font-bold mb-4">Key Results</h2>
        <p class="text-xl text-gray-600">What this research discovered</p>
      </div>

      <div class="grid md:grid-cols-3 gap-8 max-w-5xl mx-auto">
        <div class="text-center">
          <div class="text-6xl font-black text-schoolhouse-purple mb-4">56%</div>
          <h3 class="text-2xl font-heading font-bold mb-2">More Compact</h3>
          <p class="text-gray-600">Districts improved through edge-weighting</p>
        </div>

        <div class="text-center">
          <div class="text-6xl font-black text-schoolhouse-green mb-4">42%</div>
          <h3 class="text-2xl font-heading font-bold mb-2">Critical Threshold</h3>
          <p class="text-gray-600">Minority population threshold for proportional representation</p>
        </div>

        <div class="text-center">
          <div class="text-6xl font-black text-schoolhouse-orange mb-4">137</div>
          <h3 class="text-2xl font-heading font-bold mb-2">MM Districts</h3>
          <p class="text-gray-600">Majority-minority districts achievable (vs 68 enacted)</p>
        </div>
      </div>
    </ScrollSection>

    <!-- Research Preview Section -->
    <ScrollSection bg-class="bg-gradient-to-br from-orange-50 to-red-50">
      <div class="max-w-4xl mx-auto text-center">
        <h2 class="text-5xl font-heading font-bold mb-8">Based on Real Research</h2>
        <p class="text-xl text-gray-700 mb-8 leading-relaxed">
          This educational journey is built on <strong>10 peer-reviewed research papers</strong>
          with over 210 figures, analyzing redistricting across all 50 states and 3 census years.
        </p>
        <router-link to="/research" class="btn btn-primary btn-lg">
          Explore the Research →
        </router-link>
      </div>
    </ScrollSection>

    <!-- Start Journey CTA -->
    <ScrollSection bg-class="bg-schoolhouse-gray-dark text-white">
      <div class="max-w-3xl mx-auto text-center">
        <h2 class="text-5xl font-heading font-bold mb-6">Ready to Begin?</h2>
        <p class="text-xl mb-8 text-gray-300 leading-relaxed">
          Start with Chapter 1 and discover how census tracts become connected networks.
          Each chapter builds on the last, creating a complete story of algorithmic redistricting.
        </p>
        <router-link to="/chapter-1" class="btn bg-white text-schoolhouse-blue hover:bg-gray-100 btn-lg">
          Start Chapter 1 →
        </router-link>
      </div>
    </ScrollSection>
  </div>
</template>

<script setup>
import ScrollSection from '@/components/ScrollSection.vue'
import ConceptCard from '@/components/ConceptCard.vue'

const chapters = [
  {
    number: 1,
    title: 'Tracts to Graphs',
    description: 'How census tracts become connected networks that we can algorithmically divide.',
    icon: '🗺️',
    color: 'blue',
  },
  {
    number: 2,
    title: 'Splitting in Two',
    description: 'Learn how to divide any region into 2 perfectly balanced parts using METIS.',
    icon: '✂️',
    color: 'orange',
  },
  {
    number: 3,
    title: 'Recursive Magic',
    description: 'Discover how recursion creates any number of districts through repeated splitting.',
    icon: '🌳',
    color: 'green',
  },
  {
    number: 4,
    title: 'Making it Compact',
    description: 'Edge-weighting makes districts 56% more compact and geographically sensible.',
    icon: '📐',
    color: 'purple',
  },
  {
    number: 5,
    title: 'Voting Rights Act',
    description: 'Ensuring minority communities have fair representation in districts.',
    icon: '🗳️',
    color: 'red',
  },
  {
    number: 6,
    title: 'Edge-Factor Solution',
    description: 'Balancing compactness with representation through innovative edge-factor weighting.',
    icon: '⚖️',
    color: 'yellow',
  },
]
</script>

<style scoped>
.btn-lg {
  @apply px-8 py-4 text-lg;
}
</style>
