<template>
  <div class="chapter-3">
    <!-- Hero -->
    <Hero :chapter="3" color="green">
      <template #title>
        The Recursive<br />Magic
      </template>
      <template #subtitle>
        Creating any number of districts through repeated splitting
      </template>
    </Hero>

    <!-- Story Hook -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <div class="text-center mb-12">
          <h2 class="text-4xl font-heading font-bold mb-6 chapter-3-accent">
            The Family Tree of Districts
          </h2>
          <p class="text-story">
            In Chapter 2, we learned how to split Alabama into <strong>2 regions</strong>.
            But Alabama needs <strong>7 districts</strong>! How do we get from 2 to 7?
          </p>
          <p class="text-story mt-4">
            The answer: <strong>Keep splitting!</strong> Just like a family tree grows through
            generations, we can split regions recursively until we have exactly the right number
            of districts.
          </p>
        </div>

        <div class="grid md:grid-cols-2 gap-8">
          <ConceptCard color="green" icon="🌳">
            <template #title>What is Recursion?</template>
            <p>
              Recursion means applying the same process <strong>repeatedly</strong>. Split a region
              into 2, then split those into 2 more, and keep going until you have the target
              number of districts.
            </p>
          </ConceptCard>

          <ConceptCard color="green" icon="🎯">
            <template #title>Why It Works</template>
            <p>
              Since we can split <em>any</em> region into 2 balanced parts (Chapter 2), we can
              keep splitting until we hit our target. It's like binary search for geography!
            </p>
          </ConceptCard>
        </div>
      </div>
    </ScrollSection>

    <!-- The Plan for 7 Districts -->
    <ScrollSection bg-class="chapter-3-bg">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-3-accent">
          Alabama's Splitting Strategy
        </h2>

        <p class="text-story text-center mb-8">
          Remember: <strong>7 is odd</strong>, so we need an asymmetric split. Here's the plan:
        </p>

        <div class="card bg-gradient-green-blue text-white mb-12">
          <h3 class="text-2xl font-heading font-bold mb-4">The Partition Tree: [3, 4]</h3>
          <div class="font-mono text-lg leading-relaxed">
            <pre class="overflow-x-auto">
                    Alabama (7 districts)
                           |
                     Split [3, 4]
                      /          \
            Region A (3)        Region B (4)
                |                    |
          Split [1, 2]         Split [2, 2]
           /        \           /        \
        D1       R-A2(2)      D4         D5
                   |
             Split [1, 1]
              /       \
            D2        D3
            </pre>
          </div>
          <p class="mt-4 text-lg">
            This tree shows <strong>3 rounds of splitting</strong> to create 7 final districts!
          </p>
        </div>

        <div class="grid md:grid-cols-3 gap-6">
          <ConceptCard color="green" icon="1️⃣">
            <template #title>Round 1</template>
            <p class="text-sm">
              Split Alabama into 2 regions: one with 3 districts worth of population (42.8%),
              one with 4 districts worth (57.2%).
            </p>
          </ConceptCard>

          <ConceptCard color="green" icon="2️⃣">
            <template #title>Round 2</template>
            <p class="text-sm">
              Split Region A [1, 2] and Region B [2, 2]. Now we have 4 regions total.
            </p>
          </ConceptCard>

          <ConceptCard color="green" icon="3️⃣">
            <template #title>Round 3</template>
            <p class="text-sm">
              Split the remaining region that has 2 districts worth of population [1, 1].
              Done! We now have exactly 7 districts.
            </p>
          </ConceptCard>
        </div>
      </div>
    </ScrollSection>

    <!-- Round-by-Round Visualization -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-3-accent">
          Alabama's Journey Continues: Round by Round
        </h2>

        <p class="text-story text-center mb-8">
          In Chapter 2, we split Alabama in half. Now watch as those 2 regions become 4, then 7
          final districts. Same algorithm, applied recursively!
        </p>

        <!-- Round 1 -->
        <div class="mb-16">
          <div class="flex items-center justify-center mb-6">
            <span class="inline-block px-6 py-2 bg-green-100 text-schoolhouse-green font-heading font-bold rounded-full">
              Round 1: 1 → 2 regions
            </span>
          </div>

          <FigureCard
            src="/figures/chapter3/alabama_round_1_2_regions.png"
            alt="Alabama Round 1: Split into 2 regions"
          >
            <strong>Round 1:</strong> Alabama split into Region A (blue, ~2.15M people, will become
            3 districts) and Region B (orange, ~2.88M people, will become 4 districts).
          </FigureCard>

          <div class="grid md:grid-cols-2 gap-6 mt-6">
            <div class="card bg-blue-50">
              <h4 class="font-heading font-bold mb-2">Region A</h4>
              <p class="text-sm">Target: 3 districts (42.8% population)</p>
              <p class="text-sm">Status: Needs further splitting [1, 2]</p>
            </div>
            <div class="card bg-orange-50">
              <h4 class="font-heading font-bold mb-2">Region B</h4>
              <p class="text-sm">Target: 4 districts (57.2% population)</p>
              <p class="text-sm">Status: Needs further splitting [2, 2]</p>
            </div>
          </div>
        </div>

        <!-- Round 2 -->
        <div class="mb-16">
          <div class="flex items-center justify-center mb-6">
            <span class="inline-block px-6 py-2 bg-green-100 text-schoolhouse-green font-heading font-bold rounded-full">
              Round 2: 2 → 4 regions
            </span>
          </div>

          <FigureCard
            src="/figures/chapter3/alabama_round_2_4_regions.png"
            alt="Alabama Round 2: Split into 4 regions"
          >
            <strong>Round 2:</strong> Both regions from Round 1 get split. Region A becomes
            1 district + 1 region (needing more split). Region B becomes 2 regions of 2 districts each.
          </FigureCard>

          <div class="grid md:grid-cols-2 lg:grid-cols-4 gap-4 mt-6">
            <div class="card bg-blue-100 text-sm">
              <h4 class="font-heading font-bold mb-1">District 1</h4>
              <p class="text-xs">Final! ~718K people</p>
            </div>
            <div class="card bg-blue-200 text-sm">
              <h4 class="font-heading font-bold mb-1">Region A2</h4>
              <p class="text-xs">Needs split [1, 1]</p>
            </div>
            <div class="card bg-orange-100 text-sm">
              <h4 class="font-heading font-bold mb-1">Region B1</h4>
              <p class="text-xs">Needs split [1, 1]</p>
            </div>
            <div class="card bg-orange-200 text-sm">
              <h4 class="font-heading font-bold mb-1">Region B2</h4>
              <p class="text-xs">Needs split [1, 1]</p>
            </div>
          </div>
        </div>

        <!-- Round 3 -->
        <div class="mb-12">
          <div class="flex items-center justify-center mb-6">
            <span class="inline-block px-6 py-2 bg-green-100 text-schoolhouse-green font-heading font-bold rounded-full">
              Round 3: 4 → 7 districts
            </span>
          </div>

          <FigureCard
            src="/figures/chapter3/alabama_round_3_7_districts.png"
            alt="Alabama Round 3: Final 7 districts"
          >
            <strong>Round 3 (Final):</strong> All remaining regions split into individual districts.
            Alabama now has exactly 7 districts, each with ~718,700 people (±0.5%).
          </FigureCard>

          <div class="card bg-gradient-green-blue text-white mt-6">
            <h3 class="text-xl font-heading font-bold mb-3">Mission Accomplished! 🎉</h3>
            <p>
              Through 3 rounds of recursive bisection, we've transformed Alabama's 1,181 tracts
              into 7 perfectly balanced, contiguous districts.
            </p>
          </div>
        </div>
      </div>
    </ScrollSection>

    <!-- The Algorithm in Detail -->
    <ScrollSection bg-class="chapter-3-bg">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-3-accent">
          How the Algorithm Decides When to Stop
        </h2>

        <p class="text-story text-center mb-12">
          The algorithm needs to know: "Should I split this region further, or is it a final district?"
        </p>

        <div class="grid md:grid-cols-2 gap-8 mb-12">
          <div class="card bg-green-50 border-l-4 border-schoolhouse-green">
            <h3 class="text-xl font-heading font-bold mb-3 chapter-3-accent">Base Case: Stop!</h3>
            <p class="text-sm mb-3">
              If a region has <strong>1 district worth of population</strong> (within ±0.5% of
              target), it becomes a final district. No more splitting!
            </p>
            <p class="text-xs text-gray-600">
              Example: District 1 has ~718,700 people, which is within 0.5% of the target. Done!
            </p>
          </div>

          <div class="card bg-purple-50 border-l-4 border-schoolhouse-purple">
            <h3 class="text-xl font-heading font-bold mb-3 text-schoolhouse-purple">Recursive Case: Keep Going!</h3>
            <p class="text-sm mb-3">
              If a region has <strong>2+ districts worth of population</strong>, split it using
              METIS and recursively process each half.
            </p>
            <p class="text-xs text-gray-600">
              Example: Region B has ~2.88M people (4 districts). Split it [2, 2] and recurse!
            </p>
          </div>
        </div>

        <div class="card bg-gradient-to-r from-green-500 to-blue-500 text-white">
          <h3 class="text-2xl font-heading font-bold mb-4">The Recursive Formula</h3>
          <div class="font-mono text-sm md:text-base leading-relaxed">
            <pre class="overflow-x-auto">
function partition(region, num_districts):
    if num_districts == 1:
        return [region]  # Base case: Final district!

    # Recursive case: Split and recurse
    split_ratio = [floor(num_districts/2), ceil(num_districts/2)]
    [region_A, region_B] = METIS_partition(region, split_ratio)

    districts_A = partition(region_A, split_ratio[0])
    districts_B = partition(region_B, split_ratio[1])

    return districts_A + districts_B
            </pre>
          </div>
        </div>
      </div>
    </ScrollSection>

    <!-- Why Recursion is Powerful -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-3-accent">
          Why Recursion is So Powerful
        </h2>

        <div class="grid md:grid-cols-3 gap-6 mb-12">
          <ConceptCard color="green" icon="♾️">
            <template #title>Works for Any Number</template>
            <p class="text-sm">
              7 districts? 53 districts (California)? 1 district (Montana)? The same recursive
              algorithm handles them all!
            </p>
          </ConceptCard>

          <ConceptCard color="green" icon="🚀">
            <template #title>Efficient</template>
            <p class="text-sm">
              The depth of recursion is log₂(num_districts). For Alabama's 7 districts, that's
              only 3 rounds. For California's 52 districts, it's just 6 rounds!
            </p>
          </ConceptCard>

          <ConceptCard color="green" icon="🎯">
            <template #title>Guaranteed Balance</template>
            <p class="text-sm">
              Since METIS guarantees balance at each split, the final districts are all within
              ±0.5% of the target population.
            </p>
          </ConceptCard>
        </div>

        <div class="card bg-blue-50 border-l-4 border-schoolhouse-blue">
          <h3 class="text-2xl font-heading font-bold mb-3 text-schoolhouse-blue">Real-World Performance</h3>
          <div class="grid md:grid-cols-3 gap-4 text-center mt-4">
            <div>
              <p class="text-3xl font-black text-schoolhouse-green">0.3s</p>
              <p class="text-sm text-gray-600">Alabama (7 districts)</p>
            </div>
            <div>
              <p class="text-3xl font-black text-schoolhouse-orange">2.4s</p>
              <p class="text-sm text-gray-600">Texas (38 districts)</p>
            </div>
            <div>
              <p class="text-3xl font-black text-schoolhouse-purple">4.1s</p>
              <p class="text-sm text-gray-600">California (52 districts)</p>
            </div>
          </div>
        </div>
      </div>
    </ScrollSection>

    <!-- The Odd Number Problem -->
    <ScrollSection bg-class="chapter-3-bg">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-3-accent">
          Handling Odd Numbers (The "7 Problem")
        </h2>

        <p class="text-story text-center mb-8">
          Alabama's 7 districts show a common challenge: <strong>odd numbers can't split evenly</strong>.
          Here's how we handle it:
        </p>

        <div class="grid md:grid-cols-2 gap-8 mb-8">
          <div class="card bg-white">
            <h3 class="text-xl font-heading font-bold mb-4 chapter-3-accent">Floor/Ceil Strategy</h3>
            <p class="text-sm mb-4">
              For N districts, split as [floor(N/2), ceil(N/2)]:
            </p>
            <ul class="space-y-2 text-sm">
              <li>• 7 → [3, 4]</li>
              <li>• 3 → [1, 2]</li>
              <li>• 2 → [1, 1]</li>
            </ul>
            <p class="text-xs text-gray-600 mt-4">
              This ensures both sides are as balanced as possible given the integer constraint.
            </p>
          </div>

          <div class="card bg-white">
            <h3 class="text-xl font-heading font-bold mb-4 chapter-3-accent">Population Adjustment</h3>
            <p class="text-sm mb-4">
              METIS gets told the exact population ratio:
            </p>
            <ul class="space-y-2 text-sm">
              <li>• [3, 4] → 42.857% : 57.143%</li>
              <li>• [1, 2] → 33.333% : 66.667%</li>
              <li>• [1, 1] → 50.000% : 50.000%</li>
            </ul>
            <p class="text-xs text-gray-600 mt-4">
              This is why Region A in Round 1 has 42.8% of Alabama's population—not 50%!
            </p>
          </div>
        </div>

        <div class="card bg-gradient-orange-red text-white">
          <h3 class="text-2xl font-heading font-bold mb-3">The Beauty of Asymmetry</h3>
          <p class="text-lg leading-relaxed">
            Odd numbers force <strong>asymmetric splits</strong>, which often create more
            <strong>geographically sensible</strong> districts! States aren't perfectly symmetric,
            so our algorithm adapts to their natural shape.
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- Interactive Tree Placeholder -->
    <!-- Interactive Recursion Tree -->
    <ScrollSection bg-class="bg-gradient-to-br from-green-50 to-blue-50">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-4xl font-heading font-bold mb-8 text-center chapter-3-accent">
          🌳 Interactive Recursion Tree
        </h2>

        <p class="text-story text-center mb-8">
          Click <strong>"Start"</strong> to watch Alabama's 7 districts emerge through recursive
          bisection. Each step shows one region splitting into two!
        </p>

        <InteractiveTree
          :target-districts="7"
          state-name="Alabama"
          color="#10b981"
        />

        <div class="mt-8 grid md:grid-cols-2 gap-6">
          <div class="card bg-green-50">
            <h3 class="text-lg font-heading font-bold mb-3 text-green-600">
              👀 Watch the Tree Grow
            </h3>
            <ul class="text-sm space-y-2">
              <li>• Round 1: [3, 4] split</li>
              <li>• Round 2: 4 regions</li>
              <li>• Round 3: 7 final districts!</li>
            </ul>
          </div>

          <div class="card bg-blue-50">
            <h3 class="text-lg font-heading font-bold mb-3 text-schoolhouse-blue">
              🎯 Key Insight
            </h3>
            <p class="text-sm">
              Green = done. Orange = needs more splitting.
              Tree grows until all leaves are districts!
            </p>
          </div>
        </div>
      </div>
    </ScrollSection>

    <!-- Odd vs Even: Alabama vs Colorado -->
    <ScrollSection bg-class="bg-white">
      <div class="max-w-6xl mx-auto">
        <div class="text-center mb-12">
          <h2 class="text-4xl font-heading font-bold mb-6 chapter-3-accent">
            Odd vs Even: Alabama (7) vs Colorado (8)
          </h2>
          <p class="text-story">
            Let's compare Alabama's <strong>asymmetric</strong> splits with Colorado's
            <strong>perfectly symmetric</strong> splits. Same algorithm, totally different trees!
          </p>
        </div>

        <!-- Side-by-Side Partition Trees -->
        <div class="grid md:grid-cols-2 gap-8 mb-12">
          <div class="card bg-orange-50 border-l-4 border-schoolhouse-orange">
            <h3 class="text-2xl font-heading font-bold mb-4 text-schoolhouse-orange">
              Alabama (7) - "The Messy One"
            </h3>
            <div class="font-mono text-sm leading-relaxed mb-4 bg-white p-4 rounded">
              <pre>
       7
       |
    [3, 4] ← Odd!
    /    \
  [1,2] [2,2]
   / \   / \
  D1 R  D4 D5
     |
   [1,1]
    / \
   D2 D3
              </pre>
            </div>
            <ul class="space-y-2 text-sm">
              <li><strong>Round 1:</strong> Asymmetric [3, 4]</li>
              <li><strong>Round 2:</strong> [1, 2] and [2, 2]</li>
              <li><strong>Round 3:</strong> Final splits</li>
              <li><strong>Depth:</strong> 3 rounds</li>
              <li class="text-schoolhouse-orange font-bold">Every split different!</li>
            </ul>
          </div>

          <div class="card bg-blue-50 border-l-4 border-schoolhouse-blue">
            <h3 class="text-2xl font-heading font-bold mb-4 text-schoolhouse-blue">
              Colorado (8) - "The Clean One"
            </h3>
            <div class="font-mono text-sm leading-relaxed mb-4 bg-white p-4 rounded">
              <pre>
        8
        |
     [4, 4] ← Even!
     /    \
  [2,2] [2,2]
   / \   / \
 [1,1][1,1][1,1][1,1]
  / \ / \ / \ / \
 D1 D2...    ...D8
              </pre>
            </div>
            <ul class="space-y-2 text-sm">
              <li><strong>Round 1:</strong> Perfectly symmetric [4, 4]</li>
              <li><strong>Round 2:</strong> All [2, 2]</li>
              <li><strong>Round 3:</strong> All [1, 1]</li>
              <li><strong>Depth:</strong> 3 rounds</li>
              <li class="text-schoolhouse-blue font-bold">Perfect binary tree!</li>
            </ul>
          </div>
        </div>

        <!-- Visual Comparison -->
        <div class="grid md:grid-cols-2 gap-8 mb-12">
          <div>
            <h3 class="text-xl font-heading font-bold mb-4 text-center">Alabama Final Districts</h3>
            <FigureCard
              src="/figures/chapter3/alabama_final.png"
              alt="Alabama's 7 districts"
            >
              Alabama's 7 districts created through asymmetric recursive bisection.
              Notice how the splits adapt to the odd number!
            </FigureCard>
          </div>

          <div>
            <h3 class="text-xl font-heading font-bold mb-4 text-center">Colorado Final Districts</h3>
            <FigureCard
              src="/figures/chapter3/colorado_final.png"
              alt="Colorado's 8 districts"
            >
              Colorado's 8 districts created through perfectly symmetric splits.
              Power of 2 creates a clean binary tree!
            </FigureCard>
          </div>
        </div>

        <!-- Key Insights -->
        <div class="grid md:grid-cols-3 gap-6 mb-8">
          <ConceptCard color="green" icon="🎯">
            <template #title>Same Algorithm</template>
            <p class="text-sm">
              Both states use the exact same recursive bisection algorithm. The algorithm
              automatically adapts to the target number of districts.
            </p>
          </ConceptCard>

          <ConceptCard color="green" icon="🌳">
            <template #title>Different Trees</template>
            <p class="text-sm">
              Alabama's odd number creates an <strong>unbalanced tree</strong>. Colorado's power
              of 2 creates a <strong>perfect binary tree</strong>. Both work perfectly!
            </p>
          </ConceptCard>

          <ConceptCard color="green" icon="⏱️">
            <template #title>Same Depth</template>
            <p class="text-sm">
              Both take 3 rounds! Alabama: log₂(7) ≈ 2.8 rounds up to 3.
              Colorado: log₂(8) = 3 exactly. Similar performance!
            </p>
          </ConceptCard>
        </div>

        <div class="card bg-gradient-to-r from-orange-500 via-green-500 to-blue-500 text-white">
          <h3 class="text-2xl font-heading font-bold mb-4">The Beauty of Adaptability</h3>
          <p class="text-lg leading-relaxed mb-4">
            The recursive bisection algorithm doesn't care if your number is odd, even, prime,
            or a power of 2. It <strong>automatically finds the optimal splitting strategy</strong>
            for any target number of districts.
          </p>
          <p class="text-lg leading-relaxed">
            Alabama (7) gets asymmetric splits that adapt to its odd constraint. Colorado (8)
            gets perfectly balanced splits that leverage its power-of-2 structure. Both create
            fair, balanced, contiguous districts!
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- Key Takeaway -->
    <ScrollSection bg-class="chapter-3-bg">
      <div class="max-w-3xl mx-auto text-center">
        <div class="inline-block bg-white rounded-2xl shadow-2xl p-8 md:p-12 border-4 border-schoolhouse-green animate-slide-up">
          <div class="text-7xl mb-6 animate-bounce-subtle">💡</div>
          <h2 class="text-5xl font-heading font-black mb-8 chapter-3-accent">
            Key Takeaway
          </h2>
          <p class="text-3xl font-heading leading-relaxed font-bold">
            Like a <span class="underline-green bg-green-100 px-2">family tree</span>, recursive bisection creates
            <span class="underline-green bg-green-100 px-2">any number of districts</span> through repeated splitting!
          </p>
        </div>
      </div>
    </ScrollSection>

    <!-- Cliffhanger -->
    <ScrollSection bg-class="bg-gradient-to-br from-green-500 to-purple-500">
      <div class="max-w-3xl mx-auto text-center text-white">
        <h2 class="text-4xl font-heading font-bold mb-6">
          But there's a problem...
        </h2>
        <p class="text-2xl leading-relaxed">
          These districts are balanced and connected. But some look like <strong>snakes</strong>
          winding across the state. How do we make them <em>compact</em>—round and tidy?
        </p>
        <div class="mt-8 text-lg opacity-90">
          <p>The fix: teach the algorithm about geography. 📐</p>
        </div>
      </div>
    </ScrollSection>

    <!-- Chapter Navigation -->
    <ChapterNavigation :current-chapter="3" />
  </div>
</template>

<script setup>
import Hero from '@/components/Hero.vue'
import ScrollSection from '@/components/ScrollSection.vue'
import FigureCard from '@/components/FigureCard.vue'
import ConceptCard from '@/components/ConceptCard.vue'
import InteractiveTree from '@/components/InteractiveTree.vue'
import ChapterNavigation from '@/components/ChapterNavigation.vue'
</script>
