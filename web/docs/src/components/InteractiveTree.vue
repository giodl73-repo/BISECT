<template>
  <div class="interactive-tree">
    <div class="controls mb-6 flex justify-center gap-4">
      <button
        @click="reset"
        class="btn btn-secondary"
        :disabled="currentStep === 0"
      >
        Reset
      </button>
      <button
        @click="stepForward"
        class="btn btn-primary"
        :disabled="currentStep >= totalSteps"
      >
        {{ currentStep === 0 ? 'Start' : 'Next Split' }}
      </button>
      <button
        @click="autoPlay"
        class="btn btn-primary"
        :disabled="isPlaying || currentStep >= totalSteps"
      >
        {{ isPlaying ? 'Playing...' : 'Auto Play' }}
      </button>
    </div>

    <div class="tree-container bg-white rounded-xl shadow-lg p-8">
      <svg ref="svgRef" :width="width" :height="height"></svg>
    </div>

    <div class="info-panel mt-6 text-center">
      <p class="text-lg font-semibold">
        Step {{ currentStep }} of {{ totalSteps }}
        <span v-if="currentStep > 0" class="text-gray-600">
          - Split [{{ splits[currentStep - 1].from.join(', ') }}] into
          [{{ splits[currentStep - 1].left.join(', ') }}] and
          [{{ splits[currentStep - 1].right.join(', ') }}]
        </span>
      </p>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch } from 'vue'
import * as d3 from 'd3'

const props = defineProps({
  targetDistricts: {
    type: Number,
    required: true
  },
  stateName: {
    type: String,
    default: 'Alabama'
  },
  color: {
    type: String,
    default: '#f97316' // orange
  }
})

const svgRef = ref(null)
const width = ref(800)
const height = ref(500)
const currentStep = ref(0)
const isPlaying = ref(false)

// Generate partition tree structure
const generateTree = (n) => {
  const tree = {
    name: `${n}`,
    value: n,
    children: [],
    split: null
  }

  const buildNode = (node, remaining) => {
    if (remaining === 1) {
      node.name = `D${node.districtId || '?'}`
      node.isLeaf = true
      return
    }

    const left = Math.floor(remaining / 2)
    const right = Math.ceil(remaining / 2)

    node.split = `[${left}, ${right}]`
    node.children = [
      { name: `${left}`, value: left, children: [], parent: node },
      { name: `${right}`, value: right, children: [], parent: node }
    ]

    if (left > 1) buildNode(node.children[0], left)
    else {
      node.children[0].isLeaf = true
      node.children[0].name = `D${node.children[0].districtId || '?'}`
    }

    if (right > 1) buildNode(node.children[1], right)
    else {
      node.children[1].isLeaf = true
      node.children[1].name = `D${node.children[1].districtId || '?'}`
    }
  }

  if (n > 1) {
    buildNode(tree, n)
  } else {
    tree.isLeaf = true
    tree.name = 'D1'
  }

  return tree
}

// Generate split sequence for step-by-step animation
const generateSplits = (n) => {
  const splits = []
  const queue = [{ value: n, path: [] }]

  while (queue.length > 0) {
    const node = queue.shift()
    if (node.value === 1) continue

    const left = Math.floor(node.value / 2)
    const right = Math.ceil(node.value / 2)

    splits.push({
      from: [node.value],
      left: [left],
      right: [right],
      path: node.path
    })

    queue.push({ value: left, path: [...node.path, 0] })
    queue.push({ value: right, path: [...node.path, 1] })
  }

  return splits
}

const treeData = ref(null)
const splits = ref([])
const totalSteps = ref(0)

// Initialize tree
onMounted(() => {
  treeData.value = generateTree(props.targetDistricts)
  splits.value = generateSplits(props.targetDistricts)
  totalSteps.value = splits.value.length
  renderTree()
})

// Watch for step changes
watch(currentStep, () => {
  renderTree()
})

const reset = () => {
  currentStep.value = 0
  isPlaying.value = false
}

const stepForward = () => {
  if (currentStep.value < totalSteps.value) {
    currentStep.value++
  }
}

const autoPlay = async () => {
  isPlaying.value = true
  while (currentStep.value < totalSteps.value && isPlaying.value) {
    await new Promise(resolve => setTimeout(resolve, 1500))
    currentStep.value++
  }
  isPlaying.value = false
}

const renderTree = () => {
  if (!svgRef.value) return

  const svg = d3.select(svgRef.value)
  svg.selectAll('*').remove()

  // Create tree data with visible nodes based on currentStep
  const visibleTree = getVisibleTree(treeData.value, currentStep.value)

  // Create hierarchy
  const root = d3.hierarchy(visibleTree)
  const treeLayout = d3.tree().size([width.value - 100, height.value - 100])
  treeLayout(root)

  // Create group with margins
  const g = svg.append('g')
    .attr('transform', 'translate(50, 50)')

  // Draw links
  g.selectAll('.link')
    .data(root.links())
    .enter()
    .append('line')
    .attr('class', 'link')
    .attr('x1', d => d.source.x)
    .attr('y1', d => d.source.y)
    .attr('x2', d => d.target.x)
    .attr('y2', d => d.target.y)
    .attr('stroke', '#cbd5e1')
    .attr('stroke-width', 3)
    .style('opacity', 0)
    .transition()
    .duration(500)
    .style('opacity', 1)

  // Draw nodes
  const nodes = g.selectAll('.node')
    .data(root.descendants())
    .enter()
    .append('g')
    .attr('class', 'node')
    .attr('transform', d => `translate(${d.x}, ${d.y})`)
    .style('opacity', 0)

  nodes.transition()
    .duration(500)
    .style('opacity', 1)

  // Node circles
  nodes.append('circle')
    .attr('r', d => d.data.isLeaf ? 25 : 35)
    .attr('fill', d => d.data.isLeaf ? '#10b981' : props.color)
    .attr('stroke', '#fff')
    .attr('stroke-width', 3)

  // Node labels
  nodes.append('text')
    .attr('dy', '0.35em')
    .attr('text-anchor', 'middle')
    .text(d => d.data.name)
    .attr('fill', '#fff')
    .attr('font-size', d => d.data.isLeaf ? '14px' : '18px')
    .attr('font-weight', 'bold')

  // Split labels (above nodes)
  nodes.filter(d => d.data.split)
    .append('text')
    .attr('dy', '-45')
    .attr('text-anchor', 'middle')
    .text(d => d.data.split)
    .attr('fill', '#64748b')
    .attr('font-size', '14px')
    .attr('font-family', 'monospace')
}

const getVisibleTree = (tree, step) => {
  if (step === 0) {
    return {
      name: tree.name,
      value: tree.value,
      children: [],
      split: null
    }
  }

  // Clone tree and show nodes up to current step
  const clone = JSON.parse(JSON.stringify(tree))
  const splitsToShow = splits.value.slice(0, step)

  // Mark which nodes should be visible
  const markVisible = (node, depth = 0) => {
    if (depth >= step) {
      node.children = []
      return
    }
    if (node.children) {
      node.children.forEach(child => markVisible(child, depth + 1))
    }
  }

  markVisible(clone)
  return clone
}
</script>

<style scoped>
.interactive-tree {
  @apply w-full;
}

.tree-container {
  @apply overflow-x-auto;
}

.btn {
  @apply px-6 py-3 rounded-lg font-semibold text-white transition-all duration-200;
}

.btn-primary {
  @apply bg-gradient-to-r from-schoolhouse-blue to-blue-600;
}

.btn-primary:hover:not(:disabled) {
  @apply shadow-lg transform -translate-y-0.5;
}

.btn-secondary {
  @apply bg-gray-500;
}

.btn-secondary:hover:not(:disabled) {
  @apply bg-gray-600 shadow-lg;
}

.btn:disabled {
  @apply opacity-50 cursor-not-allowed;
}
</style>
