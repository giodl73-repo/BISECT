# Expert Reviewer Database

> **AI Simulation Disclosure**: This database supports an AI-simulated peer review
> system. The named researchers are **not** actual reviewers of this work. Their
> names, affiliations, and expertise areas are used to construct AI personas that
> emulate the *perspective and priorities* each researcher is known for, based on
> their published work, public talks, and documented research philosophy. No
> endorsement, affiliation, or participation by these individuals is implied.
> All reviews generated from these personas are synthetic outputs produced by a
> large language model (Claude, Anthropic). See the project README for methodology.

A comprehensive pool of expert reviewers for AI-simulated paper reviews across all research modules. Select reviewers based on paper topic and contribution type.

**Scope**: Global - shared across all modules in this monorepo.
**Per-module subsets**: See `{module}/REVIEWERS.md` for module-specific selections.

---

## Reviewer Categories

### Category: Systems & Infrastructure
*For papers on: distributed systems, state management, fault tolerance, scalability*

| Name | Affiliation | Expertise | Key Question |
|------|-------------|-----------|--------------|
| **Matei Zaharia** | Databricks/Stanford | Spark, MLflow, data systems | How does this scale? What's the failure model? |
| **Tri Dao** | Princeton/Together AI | FlashAttention, efficiency, systems | What's the compute/memory tradeoff? |
| **Ion Stoica** | Berkeley/Databricks | RAY, distributed systems | How does this compose with existing infra? |
| **Joseph Gonzalez** | Berkeley | GraphX, distributed ML | What's the communication pattern? |
| **Reynold Xin** | Databricks | Spark SQL, query optimization | Can this be expressed declaratively? |

### Category: Compilers & PL Theory
*For papers on: code generation, DSLs, formal methods, type systems*

| Name | Affiliation | Expertise | Key Question |
|------|-------------|-----------|--------------|
| **Chris Lattner** | Modular/LLVM | LLVM, Swift, MLIR, Mojo | Where's the IR? What optimizations? |
| **Nada Amin** | Harvard | PL theory, Scala, staging | What are the formal semantics? |
| **Jonathan Ragan-Kelley** | MIT | Halide, compilers, performance | Is there a search over the design space? |
| **Emery Berger** | UMass | Performance, memory, profiling | What's the overhead? |
| **Armando Solar-Lezama** | MIT | Program synthesis, Sketch | Can this be synthesized rather than templated? |

### Category: AI Agents & Orchestration
*For papers on: LLM agents, multi-agent systems, workflow orchestration*

| Name | Affiliation | Expertise | Key Question |
|------|-------------|-----------|--------------|
| **Harrison Chase** | LangChain | LangChain, LangGraph, agents | How does this integrate with existing frameworks? |
| **Jerry Liu** | LlamaIndex | RAG, retrieval, agents | What's the overhead vs autonomous execution? |
| **Omar Khattab** | Stanford/Databricks | DSPy, prompting, optimization | Can prompts be optimized automatically? |
| **Shunyu Yao** | Princeton | ReAct, Tree of Thoughts | How does this structure reasoning? |
| **Noah Shinn** | Princeton | Reflexion, self-improvement | Does the system learn from mistakes? |

### Category: Prompting & LLM Capabilities
*For papers on: prompt engineering, chain-of-thought, LLM reasoning*

| Name | Affiliation | Expertise | Key Question |
|------|-------------|-----------|--------------|
| **Jason Wei** | OpenAI | Chain-of-thought, emergent abilities | What prompting strategy? Error analysis? |
| **Denny Zhou** | Google DeepMind | Least-to-most, self-consistency | Is there structured decomposition? |
| **Sewon Min** | Berkeley/AI2 | In-context learning, few-shot | How does this compare to few-shot? |
| **Xiang Lisa Li** | Stanford | Prefix tuning, prompt tuning | Why not tune prompts? |

### Category: Human-AI Interaction
*For papers on: HITL systems, human oversight, decision support*

| Name | Affiliation | Expertise | Key Question |
|------|-------------|-----------|--------------|
| **Ben Shneiderman** | UMD | HCAI, human agency, oversight | Does this preserve meaningful human control? |
| **Michael Bernstein** | Stanford | Crowdsourcing, human computation | How does this compare to crowd workflows? |
| **Ece Kamar** | Microsoft Research | Complementarity, deferral | Should blocking be confidence-based? |
| **Saleema Amershi** | Microsoft Research | Interactive ML, HITL | Does the system learn from feedback? |
| **Krzysztof Gajos** | Harvard | Adaptive interfaces, personalization | Does this adapt to user behavior? |
| **Jeffrey Heer** | UW | Visualization, human-data interaction | How are decisions presented? |

### Category: ML Systems & Efficiency
*For papers on: training efficiency, inference optimization, model serving*

| Name | Affiliation | Expertise | Key Question |
|------|-------------|-----------|--------------|
| **Song Han** | MIT | Pruning, quantization, efficiency | What's the latency? Memory footprint? |
| **Tianqi Chen** | CMU | TVM, MLC, model optimization | Can this be compiled/optimized? |
| **Ce Zhang** | ETH/Together | Data-centric AI, systems | How does data quality affect this? |
| **Dan Fu** | Stanford | Efficient architectures, state space | What's the sequence length scaling? |

### Category: ML Research / Learning
*For papers on: learning algorithms, representations, generalization*

| Name | Affiliation | Expertise | Key Question |
|------|-------------|-----------|--------------|
| **Chuang Gan** | MIT-IBM Watson | Vision-language, multimodal | Is there a learning component? |
| **Pieter Abbeel** | Berkeley/Covariant | Robotics, RL, imitation | How does this relate to reward specification? |
| **Chelsea Finn** | Stanford | Meta-learning, robotics | Can this generalize to new tasks? |
| **Percy Liang** | Stanford | HELM, benchmarks, foundations | How was this evaluated? |
| **Ludwig Schmidt** | UW | Distribution shift, robustness | Does this work out-of-distribution? |

### Category: Software Engineering & DevOps
*For papers on: code review, testing, CI/CD, developer tools*

| Name | Affiliation | Expertise | Key Question |
|------|-------------|-----------|--------------|
| **Shreya Shankar** | Berkeley | ML ops, observability, pipelines | How do you debug this? Test it? |
| **Sarah Bird** | Microsoft/Responsible AI | ML ops, responsible AI | What are the failure modes? |
| **Nadia Polikarpova** | UCSD | Verification, synthesis, types | Can correctness be verified? |
| **Sumit Gulwani** | Microsoft Research | Program synthesis, FlashFill | Can this be synthesized from examples? |

### Category: NLP & Information Retrieval
*For papers on: text generation, retrieval, QA systems*

| Name | Affiliation | Expertise | Key Question |
|------|-------------|-----------|--------------|
| **Danqi Chen** | Princeton | Dense retrieval, QA | How does retrieval quality affect this? |
| **Mike Lewis** | Meta AI | RAG, BART, generation | What's the retrieval-generation tradeoff? |
| **Kenton Lee** | Google | QA, retrieval, entity linking | How does this handle ambiguity? |
| **Luke Zettlemoyer** | UW/Meta | Semantic parsing, NLU | Is there structure in the output? |

### Category: Security & Safety
*For papers on: AI safety, alignment, security*

| Name | Affiliation | Expertise | Key Question |
|------|-------------|-----------|--------------|
| **Dan Hendrycks** | Center for AI Safety | Benchmarks, safety evaluation | What could go wrong? Edge cases? |
| **Jacob Steinhardt** | Berkeley | Robustness, adversarial | What are the adversarial failure modes? |
| **Nicholas Carlini** | Google DeepMind | Adversarial ML, extraction | Can this be attacked? |
| **Florian Tramer** | ETH | Privacy, security, ML | What information leaks? |

---

## Conference-Specific Selection Guides

### MLSys
Focus: Systems + efficiency + scalability
**Recommended reviewers**: Matei Zaharia, Tri Dao, Ion Stoica, Song Han, Tianqi Chen, Shreya Shankar

### NeurIPS (Systems Track)
Focus: Novel ML systems with learning component
**Recommended reviewers**: Matei Zaharia, Pieter Abbeel, Chelsea Finn, Ce Zhang, Omar Khattab

### ICML
Focus: Learning algorithms, theoretical grounding
**Recommended reviewers**: Percy Liang, Chuang Gan, Chelsea Finn, Denny Zhou, Ludwig Schmidt

### ACL/EMNLP
Focus: NLP, text generation, language understanding
**Recommended reviewers**: Danqi Chen, Mike Lewis, Luke Zettlemoyer, Sewon Min, Omar Khattab

### CHI/UIST
Focus: Human-AI interaction, interfaces
**Recommended reviewers**: Ben Shneiderman, Michael Bernstein, Ece Kamar, Saleema Amershi, Jeffrey Heer

### PLDI/OOPSLA
Focus: Compilers, languages, verification
**Recommended reviewers**: Chris Lattner, Nada Amin, Jonathan Ragan-Kelley, Armando Solar-Lezama, Emery Berger

### OSDI/SOSP
Focus: Operating systems, distributed systems
**Recommended reviewers**: Ion Stoica, Matei Zaharia, Joseph Gonzalez, Reynold Xin

### ICLR
Focus: Representations, deep learning methods
**Recommended reviewers**: Percy Liang, Chelsea Finn, Ludwig Schmidt, Dan Fu, Chuang Gan

---

## Paper Type to Reviewer Selection

### Type: AI Workflow/Orchestration Paper
**Must have**: Harrison Chase, Jerry Liu, Omar Khattab
**Add for systems angle**: Matei Zaharia, Tri Dao
**Add for HCI angle**: Michael Bernstein, Ece Kamar

### Type: Code Generation/DSL Paper
**Must have**: Chris Lattner, Nada Amin, Jonathan Ragan-Kelley
**Add for AI angle**: Omar Khattab, Harrison Chase
**Add for testing**: Shreya Shankar

### Type: Human-AI Collaboration Paper
**Must have**: Ben Shneiderman, Michael Bernstein, Ece Kamar, Saleema Amershi
**Add for agents**: Harrison Chase, Jerry Liu
**Add for prompting**: Jason Wei

### Type: Efficiency/Systems Paper
**Must have**: Song Han, Tri Dao, Tianqi Chen
**Add for distributed**: Matei Zaharia, Ion Stoica
**Add for serving**: Ce Zhang

### Type: Prompting/Reasoning Paper
**Must have**: Jason Wei, Denny Zhou, Omar Khattab
**Add for evaluation**: Percy Liang
**Add for learning**: Chuang Gan, Pieter Abbeel

### Type: State Management/Checkpointing Paper
**Must have**: Matei Zaharia, Tri Dao, Ion Stoica
**Add for correctness**: Nada Amin
**Add for ops**: Shreya Shankar

---

## Reviewer Expertise Tags

Quick reference for expertise matching:

| Tag | Reviewers |
|-----|-----------|
| `distributed-systems` | Matei Zaharia, Ion Stoica, Joseph Gonzalez |
| `compilers` | Chris Lattner, Jonathan Ragan-Kelley, Tianqi Chen |
| `pl-theory` | Nada Amin, Armando Solar-Lezama |
| `agents` | Harrison Chase, Jerry Liu, Shunyu Yao |
| `prompting` | Jason Wei, Denny Zhou, Omar Khattab |
| `human-ai` | Ben Shneiderman, Michael Bernstein, Ece Kamar |
| `efficiency` | Song Han, Tri Dao, Dan Fu |
| `ml-ops` | Shreya Shankar, Sarah Bird |
| `retrieval` | Jerry Liu, Danqi Chen, Mike Lewis |
| `safety` | Dan Hendrycks, Jacob Steinhardt |
| `learning` | Pieter Abbeel, Chelsea Finn, Chuang Gan |
| `evaluation` | Percy Liang, Ludwig Schmidt |

---

## Adding New Reviewers

When adding reviewers from new papers or citations:

1. **Identify expertise category** from their top papers
2. **Find their "key question"** - what do they always ask?
3. **Add affiliation** - current as of paper date
4. **Tag with expertise** for quick lookup
5. **Note conference preference** if known

Template:
```markdown
| **Name** | Affiliation | Expertise | Key Question |
```

---

*Last updated: February 2026*
*Total reviewers: 45+*
