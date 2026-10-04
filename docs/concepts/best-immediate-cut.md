# The argument for the best immediate cut

## Short version

Divide representation proportionally, keep both regions connected, and
introduce the least boundary compatible with the best achievable population
division. Resolve exact ties by a published ordering. Repeat the same rule at
each required split.

This is a proposed justification of the project's existing strict population
priority, not a new solver or a claim that every geographic value has a single
mathematical answer. Once the inputs and priorities are fixed, the rule defines
a unique best immediate cut. The research's construction and search methods
can then compete to find that same cut.

## Define proportional representation

Let a region have population P and require k = a + b future districts. The
recursive schedule fixes a = floor(k/2) and b = k - a. If the left child has
population p, the proportional target is aP/k. Define the integer deviation

\[
D(p) = |kp-aP|.
\]

This avoids rounding the target before comparing candidates. Because the
right child has population P-p, its corresponding numerator is
|k(P-p)-bP| = D(p). For fixed positive a and b, minimizing D also minimizes
the difference in population per future district:

\[
\left|\frac{p}{a}-\frac{P-p}{b}\right| = \frac{D(p)}{ab}.
\]

Thus population balance is tied directly to equal representation per seat.
The choice to give it absolute priority is a policy premise. Even a one-person
improvement outranks any boundary saving under this rule. A published tolerance
would define a different population policy and is not introduced here.

The current [objective implementation](../../crates/bisect-ilp/src/certified_split.rs)
stores the maximum and total of the two population numerators. Here they are
D and 2D, so those fields produce the same population ranking. They are retained
in the implementation and certificate format.

## Define the permitted cuts and their boundary cost

Fix the geographic units, populations, adjacency, island links, district-count
schedule and edge weights before searching. A permitted immediate cut assigns
each unit to one of two nonempty children, makes each child connected under the
published graph rules, and leaves each child at least as many units as its
required district count. Apply the published orientation rule as well.

For a permitted assignment x, let

\[
C(x)=\sum_{\{u,v\}:x_u\ne x_v}w_{uv}.
\]

With physical shared-boundary lengths as weights, C measures the boundary
introduced by this division. Administrative penalties instead make it a
weighted separation cost. Synthetic island links have their published graph
costs; they are not physical interfaces. Quantized integer weights define the
implemented objective exactly, but approximate physical length to the declared
precision.

Unit-count feasibility does not guarantee that descendants can complete their
schedule. Completion is checked as the procedure proceeds; it does not change
the parent's ranking. An empty permitted set must produce a declared failure.

## Why minimizing boundary has a geometric justification

Suppose the units form a fixed planar region and all physical shared interfaces
are accounted for consistently. Write L for the physical length of the new
interface. Each inherited exterior boundary belongs to one child, while the
new interface belongs to both children's perimeters. Therefore

\[
\operatorname{Per}(R_L)+\operatorname{Per}(R_R)
=\operatorname{Per}(R)+2L.
\]

Since the parent perimeter is fixed, minimizing L is exactly equivalent to
minimizing the children's combined perimeter. The argument includes existing
holes and disconnected physical components when their boundaries are accounted
for consistently. Graph connectivity through synthetic bridges remains a
separate convention.

This establishes a local geometric objective. It does not establish maximum
mean Polsby-Popper: population balance does not fix child areas, and a sum of
perimeters does not rank a sum of area-to-perimeter-squared ratios. It also does
not establish minimum final-plan perimeter, minimum county split count, or
optimal partisan outcomes. Administrative weights require their own rationale;
the physical perimeter identity does not select their coefficients.

## The selection rule and its characterization

Order permitted assignments by the tuple

\[
\bigl(D(x),\ C(x),\ K(x)\bigr),
\]

where K is the published canonical ordering of assignments after orientation.
Choose the lexicographically smallest tuple: first population, then boundary,
then canonical order.

The rule is characterized by three requirements. A selected cut must admit no
permitted population improvement. At that best population value, it must admit
no boundary improvement. At both values, it must admit no earlier canonical
assignment. These requirements are equivalent to minimizing the tuple; they
are not evidence that society must adopt the underlying priorities.

**Existence and uniqueness.** A finite unit set has finitely many assignments.
If its permitted set is nonempty, some assignment attains minimum D. Among
those assignments, some attains minimum C. A total canonical ordering then
selects exactly one assignment. Conversely, any assignment satisfying the
three requirements is that minimum: a smaller tuple would violate the first
requirement at which it differs.

Multiplying every boundary cost by the same positive constant preserves the
winner. In particular, a normalization depending only on the fixed a/b seat
ratio cannot change the winner at this node. Comparing different seat ratios
is a different decision. This helps distinguish the [ratio research](../reviews/2026-10-02-research-portfolio-review.md)
from the present fixed-schedule selection rule.

## What the research establishes

The [portfolio review](../reviews/2026-10-02-research-portfolio-review.md)
shows extensive investigation of weighting, ratio choice, alternative
constructions, sampling, local search, decomposition and exact optimization.
These experiments help evaluate premises and discover better candidates. They
do not collectively establish a universal winner across different objectives.

The [RI frontier](../experiments/scalable-certified/FRONTIER-REVIEW.md)
records connected candidates retaining the proved population floor, with
boundary cost improving from 102,659,356 to 43,047,238. This supports continued
search under the immediate-cut objective. The latter candidate remains an
incumbent, not an unrestricted boundary optimum.

The [national operational comparison](../experiments/nrs-v0.3-national-bakeoff-geometry-2020/README.md)
reports unfavorable geometric metrics against its official block-projected
comparator. It limits claims about that candidate engine's achieved quality;
it does not test every permitted cut or refute the definition above.

METIS, local improvement, MILP and SAT can serve the same selection rule if
they use the same instance, feasible set and objective. A method that finds a
better tuple has found a better immediate cut. A method that exhaustively
excludes all better tuples establishes the winner. A restricted-neighborhood
optimum establishes only the winner within that neighborhood.

## How to establish that the selected cut is best

The [certified procedure](certified-recursive-bisection.md) translates the
characterization into independently checked decisions: exclude a better
population value, exclude a smaller boundary at the population optimum, then
exclude an earlier canonical assignment at both optima. These exclusions must
cover the unrestricted permitted set, not only a heuristic candidate pool.

Discovery can use the fastest available methods. The proof checks the selected
result against the rule, independently of the path used to discover it. The
project has bounded exact evidence and a proved RI population floor; the first
complete State-scale boundary and canonical proof remains open.

For comparing existing solvers, use identical instances and report incumbent
tuples, unrestricted lower bounds, runtime, memory and proof-checking cost.
Keep timeouts and restricted bounds visible. The research does not yet identify
one solver as the most efficient across all regions.

## The public argument

> BISECT divides population in proportion to the districts each region will
> contain. Among connected cuts with the best achievable population division,
> it chooses the least weighted boundary, then resolves exact ties by a public
> ordering. Independent proof checking can establish that no permitted cut
> ranks ahead of the selected cut.

With physical length weights, the geographic rationale is minimum combined
child perimeter. With administrative weights, publish the additional preference
explicitly. Certification supports a best-cut claim only for nodes whose full
optimality evidence has passed verification. Final district checks and declared
failure handling remain necessary before claiming a usable completed plan.
