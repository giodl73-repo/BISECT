# Demographic Similarity Districts — Plan

**Artifact Type**: Research Paper (Paper #16 - Experimental)
**Goal**: Create districts by clustering demographically similar census tracts instead of optimizing geography
**Estimated Effort**: 2-3 weeks
**Status**: Planned
**Source**: Enhancement E26

---

## Objective

Explore radical alternative: **What if districts grouped people by demographic characteristics instead of geographic proximity?**

**Research Question**: If representation is meant to reflect "communities of interest," could demographic similarity (age, race, income, education) define communities better than geography?

**Theoretical Motivation**: Geography is arbitrary for many interests (young professionals, retirees, racial minorities share concerns across space). Demographics may better capture shared political preferences.

---

## Research Questions

### RQ1: Demographic Clustering
**Can we create 435 demographically coherent districts?**

- Define similarity metric (Euclidean distance in demographic space)
- Cluster census tracts by demographics (k-means, hierarchical clustering)
- Constraint: Population balance (±0.5%)
- **Challenge**: Geographic contiguity impossible (similar tracts are dispersed)

### RQ2: Geographic Dispersion
**How dispersed would demographic districts be?**

- Hypothesis: Extreme dispersion (national-scale districts)
- Example: "Young urban professionals" district spanning all cities
- Example: "Rural retirees" district spanning all rural counties

### RQ3: Political Homogeneity
**Would demographic districts reduce political diversity within districts?**

- Hypothesis: Demographic clustering creates ideologically homogeneous districts
- Trade-off: Better representation of shared interests vs reduced deliberation

### RQ4: Constitutional Feasibility
**Could demographic districts satisfy constitutional requirements?**

- Geographic contiguity requirement (historically implied, not explicit)
- Likely unconstitutional (Gomillion v. Lightfoot 1960: bizarre shapes unconstitutional)
- BUT: Thought experiment reveals importance of geographic constraint

---

## Proposed Structure

### Abstract (150 words)
- Problem: Geographic districts assume shared interests within localities
- Question: What if we clustered by demographics instead of geography?
- Method: Apply k-means clustering to demographic similarity (age, race, income, education)
- Findings: (TBD) Demographic districts extremely dispersed but demographically coherent
- Contribution: Reveals role of geographic constraint in representation theory

### Section 1: Introduction (600 words)
- **Geographic assumption**: Congressional districts defined by geographic proximity
- **Question**: Is geography the right dimension for defining representation?
- **Alternative**: Demographic similarity (age, race, education, income)
- **This paper**: Thought experiment testing demographic clustering
- **Value**: Reveals what we lose/gain by enforcing geographic contiguity

### Section 2: Representation Theory (800 words)

#### 2.1: Communities of Interest
- Traditional view: Communities defined geographically (neighborhoods, cities, regions)
- Alternative view: Communities defined by shared characteristics (identity, class, interests)
- Example: LGBTQ community spans cities, racial minorities dispersed in suburbs

#### 2.2: Descriptive vs Substantive Representation
- **Descriptive**: Representative shares demographic traits with constituents (race, gender, age)
- **Substantive**: Representative advocates for constituent interests (policy preferences)
- Demographic districts maximize descriptive representation

#### 2.3: Prior Work on Non-Geographic Representation
- Functional representation: Represent occupations, not places (19th century proposals)
- Corporate voting: Shareholders vote proportionally (not geographic)
- Online communities: Reddit, Facebook groups (shared interests, not location)

### Section 3: Methodology (1,000 words)

#### 3.1: Demographic Variables
**Census tract characteristics** (2020):
- **Age**: % under 18, % 18-64, % 65+
- **Race**: % White, % Black, % Hispanic, % Asian, % Other
- **Income**: Median household income, % below poverty line
- **Education**: % high school, % bachelor's, % graduate degree
- **Housing**: % homeowners, % renters, median home value
- **Employment**: % employed, % unemployed, occupation distribution

**Total**: 15-20 demographic variables per tract

#### 3.2: Clustering Algorithm
**Step 1**: Normalize variables (z-scores)
**Step 2**: Compute pairwise tract similarity (Euclidean distance in demographic space)
**Step 3**: Apply k-means clustering (k=435 districts)
**Step 4**: Balance populations (iteratively reassign tracts to achieve ±0.5%)
**Step 5**: Analyze geographic dispersion

**Alternative algorithms**:
- Hierarchical clustering (reveals demographic hierarchy)
- Spectral clustering (handles non-linear similarities)
- DBSCAN (density-based, finds natural clusters)

#### 3.3: Metrics
- **Demographic coherence**: Within-cluster variance (low = homogeneous)
- **Geographic dispersion**: Mean pairwise distance between tracts in same district
- **Compactness**: Polsby-Popper (expected to be very low)
- **Political homogeneity**: Vote share variance within districts

### Section 4: Results (1,500 words)

#### 4.1: Cluster Characteristics
**Table 1**: Sample demographic districts

| District | Primary Characteristics | Geographic Span | Population | Coherence Score |
|----------|-------------------------|-----------------|------------|-----------------|
| 001 | Young urban professionals (25-40, college-educated, high-income) | National (all cities) | 769,000 | 0.92 |
| 002 | Rural retirees (65+, white, low-density) | National (all rural) | 769,000 | 0.89 |
| 003 | Black urban (majority-black, urban, moderate-income) | Southern + Northern cities | 769,000 | 0.87 |
| ... | | | | |

**Finding**: Demographic districts are nationally dispersed, highly homogeneous within clusters

#### 4.2: Geographic Dispersion
**Table 2**: Geographic statistics

| Metric | Demographic Districts | Geographic Districts |
|--------|----------------------|----------------------|
| Mean pairwise distance (miles) | TBD (very high) | TBD (low) |
| Contiguous districts | 0% | 100% |
| States spanned per district | TBD (many) | 1 (by definition) |

**Figure 1**: Map showing 3 sample demographic districts
- District 001 (young professionals): Tracts in NYC, SF, Seattle, Boston, DC, Chicago
- District 002 (rural retirees): Tracts in rural ME, WV, IA, MT
- District 003 (Black urban): Tracts in Atlanta, Detroit, Baltimore, Houston

**Visual**: Extreme dispersion, no geographic coherence

#### 4.3: Political Homogeneity
**Table 3**: Vote share variance

| System | Mean within-district variance | Competitive districts (45-55%) |
|--------|------------------------------|--------------------------------|
| Demographic | Very low (TBD) | 0 (all safe) |
| Geographic (algorithmic) | Moderate | 15-20% |

**Finding**: Demographic districts eliminate competitive elections (all homogeneous)

#### 4.4: Compactness Disaster
**Table 4**: Polsby-Popper scores

| System | Mean PP | Median PP |
|--------|---------|-----------|
| Demographic | ~0.001 (terrible) | ~0.001 |
| Geographic (algorithmic) | 0.461 | 0.456 |

**Interpretation**: Geographic constraint is essential for compact districts

### Section 5: Discussion (1,200 words)

#### 5.1: Constitutional Impossibility
**Geographic contiguity**:
- Historically required (implied in redistricting law)
- Gomillion v. Lightfoot (1960): Bizarre shapes unconstitutional
- Demographic districts would be struck down immediately

#### 5.2: Theoretical Implications
**What this reveals**:
- Geography is not arbitrary constraint—it's essential for:
  - Local accountability (representative can visit district)
  - Shared local concerns (schools, infrastructure, zoning)
  - Deliberation (competitive districts force representatives to balance views)

**Demographic clustering eliminates**:
- Local representation (no shared place)
- Competitive elections (all safe seats)
- Cross-demographic deliberation (homogeneous districts)

#### 5.3: Functional Representation Analogy
- 19th century proposals: Represent occupations (farmers, laborers, merchants)
- Similar logic: Group by shared interests, not location
- Rejected because: Place matters for governance (local roads, schools, infrastructure)

#### 5.4: Online Communities Parallel
- Modern problem: Many people's primary "communities" are online, not local
- Example: Reddit communities, Facebook groups, Twitter networks
- Question: Should representation follow digital communities?
- Answer (this paper): No—place still matters for governance

#### 5.5: Value of Thought Experiment
**What we learn**:
- Geographic constraint is not arbitrary—it serves critical functions
- Demographic representation maximizes descriptive at cost of local accountability
- Trade-offs between representation dimensions (descriptive vs geographic vs deliberative)

### Section 6: Conclusion (400 words)
- Summary: Demographic districts theoretically feasible but practically/constitutionally impossible
- Value: Reveals importance of geographic constraint
- Policy implication: Reforms should preserve geographic basis while improving other dimensions
- Future work: Hybrid systems (geographic with demographic weighting?)

---

## Figures (5 total)

**Figure 1: Sample Demographic Districts Map**
- 3 panels showing tracts in same demographic district
- Panel A: Young urban professionals (dots in all major cities)
- Panel B: Rural retirees (dots scattered across rural regions)
- Panel C: Black urban (Southern + Northern cities)

**Figure 2: Demographic Space Visualization**
- 2D projection (PCA or t-SNE) of census tracts in demographic space
- Color by cluster assignment
- Shows clean separation in demographic dimensions

**Figure 3: Dispersion vs Coherence**
- Scatter plot: Geographic dispersion (X) vs Demographic coherence (Y)
- Demographic districts: High dispersion, high coherence
- Geographic districts: Low dispersion, moderate coherence

**Figure 4: Political Homogeneity**
- Histogram of district-level vote shares
- Demographic districts: Bimodal (all safe R or safe D)
- Geographic districts: Normal distribution (competitive center)

**Figure 5: Compactness Comparison**
- Side-by-side maps: California
- Panel A: Geographic districts (compact)
- Panel B: Demographic districts (scattered tracts)
- Polsby-Popper scores annotated

---

## Target Venues

### Option 1: Perspectives on Politics (APSA)
**Why Perspectives?**
- Theoretical political science
- Representation theory focus
- Thought experiments and conceptual work
- Format: 8,000-10,000 words

**Fit**: Theoretical contribution to representation literature

### Option 2: American Journal of Political Science (AJPS)
**Why AJPS?**
- High-tier venue
- Quantitative theory
- Descriptive representation research
- Format: 10,000-12,000 words

**Fit**: Empirical test of theoretical question

### Option 3: Demography
**Why Demography?**
- Demographic clustering methodology
- Population representation
- Spatial demographics
- Format: 8,000 words

**Fit**: Demographic methods innovation

**Recommendation**: Submit to **Perspectives on Politics first** (best fit for thought experiment).

---

## Data Requirements

**Already Available**:
- Census tract demographics (2020 Census, ACS)
- Census tract geometries (TIGER/Line)
- Presidential vote by tract (estimated from precincts)

**Need to Compute**:
- Demographic similarity matrix (pairwise distances)
- k-means clustering (k=435)
- Geographic dispersion metrics
- Political homogeneity statistics

**Estimated Data Processing**: 1 week (clustering algorithm, dispersion analysis)

---

## Implementation Timeline

### Phase 1: Data Preparation (3-4 days)
- Extract demographic variables (15-20 per tract)
- Normalize and compute similarity matrix

### Phase 2: Clustering (3-4 days)
- Run k-means (k=435)
- Balance populations iteratively
- Validate demographic coherence

### Phase 3: Analysis (1 week)
- Compute geographic dispersion
- Analyze political homogeneity
- Generate comparison to geographic districts

### Phase 4: Writing (1 week)
- Draft all sections
- Representation theory section
- Constitutional analysis

### Phase 5: Review & Submission (3 days)
- Internal review
- Revise and submit

**Total: 2-3 weeks**

---

## Success Criteria

- [ ] 435 demographic districts created
- [ ] Population balance achieved (±0.5%)
- [ ] Demographic coherence quantified
- [ ] Geographic dispersion computed
- [ ] Political homogeneity analyzed
- [ ] All 5 figures generated
- [ ] Draft complete (8,000-10,000 words)

---

## Related Work Integration

**From Paper 01 (recursive-bisection)**:
- Provides geographic baseline for comparison

**From Paper 03 (vra-compliance)**:
- Discusses representation of minority communities (geographic vs demographic)

**Contrast**:
- Geographic: Optimizes space, accepts demographic heterogeneity
- Demographic: Optimizes demographics, accepts geographic dispersion

---

**Created**: 2026-02-08
**Panel Reference**: N/A (experimental)
**Related Enhancement**: E26 (Demographic Similarity)
**Risk Level**: Low (clearly theoretical, not policy proposal)
**Scientific Value**: High (reveals role of geographic constraint in representation)
