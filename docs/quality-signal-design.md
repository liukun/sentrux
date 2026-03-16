# Sentrux Quality Signal — Design Documentation

## The Problem

In the AI agent era, code quality degrades through iterative generation. AI agents write code at machine speed, each commit adding entropy to the system. Without structural feedback, the codebase reaches a point where the AI agent can no longer make progress — it's stuck in its own mess.

The traditional approach (human code review) doesn't scale to machine-speed generation. We need an automated sensor that creates a feedback loop: measure structure, signal quality, guide the agent toward genuine architectural improvement.

## The Philosophy

### Why not just test suites?

Tests verify BEHAVIOR (does it produce the right output). They don't verify STRUCTURE (is it organized well). An AI agent can pass all tests while producing spaghetti architecture — tests say "it works" but the codebase is unmaintainable.

### Why not letter grades (A-F)?

Letter grades create artificial boundaries. Score 0.79 = B, score 0.81 = A. The AI agent games the boundary instead of genuinely improving. With continuous scores, every 0.01 improvement matters equally. No boundary to game. The AI converges naturally when improvements become marginal.

### Why not 20 proxy metrics?

Proxy metrics (coupling ratio, dead code percentage, function length, etc.) measure SYMPTOMS, not ROOT CAUSES. An AI agent can game individual proxies — add fake imports to boost cohesion, make everything public to eliminate "dead code," split functions superficially to reduce length. The metrics improve but the code doesn't.

Root cause metrics measure FUNDAMENTAL STRUCTURAL PROPERTIES that can only be improved through genuine architectural change.

### The Cybernetics Foundation

From Norbert Wiener (1948) and 钱学森 (Tsien Hsue-shen, engineering cybernetics):

```
sensor → signal → controller → actuator → system → sensor

Sentrux (sensor) → quality_signal → AI agent (controller) → code changes (actuator) → codebase (system) → Sentrux
```

For this loop to converge to genuine improvement, the signal must be:
- **Monotone**: genuine improvement always increases the signal
- **Smooth**: small changes produce small signal changes (no discontinuities)
- **Ungameable**: improving the signal requires improving the actual system
- **Observable**: computable from available data (static analysis)

## The 5 Root Cause Metrics

A codebase is a directed graph G = (V, E) where V = files (with properties like size and complexity) and E = dependencies (imports, calls).

From graph theory and information theory, this graph has exactly 5 independent structural properties:

### 1. Modularity (Newman's Q)

**Theory**: Newman 2004, graph community detection.

**What it measures**: How well the dependency graph decomposes into independent clusters (modules). Compares actual intra-module edge density against a random graph with the same degree sequence.

**Formula**:
```
Q = (1/m) * Σ [A_ij - k_out_i * k_in_j / m] * δ(c_i, c_j)

A_ij = 1 if edge from i to j
k_out_i = out-degree of node i
k_in_j = in-degree of node j
m = total edges
δ(c_i, c_j) = 1 if same module
```

**Range**: [-0.5, 1.0]. Q > 0.3 = significant modular structure.

**Normalization**: score = (Q + 0.5) / 1.5 → [0, 1]

**Why it's ungameable**: Q measures actual vs random edge distribution. Adding useless edges moves the graph CLOSER to random, which DECREASES Q. Only genuine modular restructuring improves Q.

**Language-fair**: Works on ANY graph. Uses both import edges and call edges. Swift projects with 0 imports but 6 call edges still get a meaningful Q value.

**Replaces**: coupling + cohesion + god files + hotspots (all symptoms of low Q).

### 2. Acyclicity

**Theory**: Martin 2003, Acyclic Dependencies Principle (ADP).

**What it measures**: Absence of circular dependencies. Cycles make build order undefined, change propagation unpredictable, and testing difficult.

**Computation**: Tarjan's SCC algorithm counts strongly connected components with >1 member.

**Range**: [0, ∞) cycle count.

**Normalization**: score = 1 / (1 + cycle_count) → (0, 1]. Sigmoid because count is unbounded.

**Why it's fundamental**: A cycle means A depends on B depends on A — neither can be understood or tested independently. This is a structural impossibility, not a style preference.

### 3. Depth

**Theory**: Lakos 1996, levelization and layered architecture.

**What it measures**: Longest dependency chain in the DAG. Deep chains mean a change at the bottom propagates through many layers.

**Computation**: Iterative longest-path DFS from entry points or root nodes.

**Range**: [0, ∞) levels.

**Normalization**: score = 1 / (1 + depth / 8) → (0, 1]. Sigmoid with midpoint 8 (depth 8 = score 0.5).

**Why it's independent from Q**: A graph can have perfect modularity (high Q) but still have a chain of 20 modules depending on each other sequentially. Depth measures this orthogonal property.

### 4. Equality (Gini Coefficient)

**Theory**: Gini 1912, originally from economics (wealth inequality).

**What it measures**: How evenly complexity is distributed across functions. A codebase where one god function has CC=200 and all others have CC=2 has high Gini (concentrated). A codebase where all functions have CC=5-10 has low Gini (distributed).

**Formula**:
```
Sort values ascending.
G = Σ (2i - n - 1) * x_i / (n * Σ x_i)
```

**Range**: [0, 1]. G=0 perfectly equal, G=1 one element has everything.

**Normalization**: score = 1 - G → [0, 1]. Direct invert.

**Why it matters**: God files/functions are the #1 source of AI agent confusion. When 40% of complexity is in one file, the AI agent can't reason about it effectively. Distributed complexity is manageable complexity.

**Why not Shannon entropy**: We initially included entropy but removed it. Entropy measures distribution uniformity (high entropy = uniform = all files same size). But: (a) entropy of file sizes gives confusing direction (high = good contradicts thermodynamic intuition), (b) Gini already captures the same concern (concentration), (c) keeping both would double-count this dimension. Gini is more intuitive and better at detecting outliers.

### 5. Redundancy

**Theory**: Kolmogorov complexity — the gap between actual code and minimum equivalent code.

**What it measures**: Fraction of code that is unnecessary. Combines dead functions (unreferenced by any call site) and duplicate functions (identical body hashes).

**Formula**: R = (dead_count + duplicate_count) / total_functions

**Range**: [0, 1].

**Normalization**: score = 1 - R → [0, 1]. Direct invert.

**Why it's fundamental**: Every line of dead or duplicate code is structural waste — it increases the search space for the AI agent without contributing to behavior. Removing it always improves the codebase.

## Quality Signal Computation

### Step 1: Compute 5 raw values

```
modularity_q       = Newman's Q from import + call graph
cycle_count         = Tarjan's SCC count
max_depth           = longest path in DAG
complexity_gini     = Gini coefficient of per-function CC
redundancy_ratio    = (dead + duplicate) / total functions
```

### Step 2: Normalize to [0, 1]

```
modularity  = (Q + 0.5) / 1.5          bounded [-0.5, 1] → [0, 1]
acyclicity  = 1 / (1 + cycles)          unbounded → sigmoid
depth       = 1 / (1 + max_depth / 8)   unbounded → sigmoid
equality    = 1 - gini                   bounded [0, 1] → [1, 0]
redundancy  = 1 - ratio                  bounded [0, 1] → [1, 0]
```

3 of 5 metrics have ZERO arbitrary parameters (direct normalization).
2 of 5 need a sigmoid midpoint: cycles midpoint = 1, depth midpoint = 8.

### Step 3: Geometric mean

```
quality_signal = (modularity * acyclicity * depth * equality * redundancy) ^ (1/5)
```

**Why geometric mean** (Nash Social Welfare theorem, 1950):

The geometric mean is the UNIQUE aggregation function satisfying:
1. **Pareto optimality**: if all scores improve, signal improves
2. **Symmetry**: all root causes are equally important
3. **Independence**: irrelevant dimensions don't affect the result

Practically: gaming one metric while tanking another cannot increase the geometric mean. The ONLY way to significantly raise it is to improve ALL factors. This forces the AI agent toward genuine architectural improvement (Approach 2) instead of metric gaming (Approach 1).

### Step 4: Display

GUI shows integer on 0-10000 scale: `Quality 7342` with pixel block progress bar and continuous red→yellow→green color gradient. No letter grades. No percentages. No arbitrary boundaries. Every +1 is one point of real improvement.

MCP returns integer 0-10000 for AI agent: `"quality_signal": 7342`

## For AI Agents (MCP Interface)

### MCP Response

```json
{
  "quality_signal": 7342,
  "bottleneck": "modularity",
  "root_causes": {
    "modularity":  {"score": 6700, "raw": 0.45},
    "acyclicity":  {"score": 10000, "raw": 0},
    "depth":       {"score": 10000, "raw": 4},
    "equality":    {"score": 6100, "raw": 0.35},
    "redundancy":  {"score": 8800, "raw": 0.12}
  },
  "diagnostics": {
    "modularity": {
      "god_files": [{"path": "...", "fan_out": 18}],
      "hotspot_files": [{"path": "...", "fan_in": 25}],
      "most_unstable": [{"path": "...", "instability": 0.92}]
    },
    "equality": {
      "complex_functions": [{"file": "...", "func": "...", "cc": 75}],
      "large_files": [{"path": "...", "lines": 1200}]
    },
    "redundancy": {
      "dead_functions": [{"file": "...", "func": "...", "lines": 45}],
      "duplicate_groups": [...]
    }
  }
}
```

The AI agent maximizes `quality_signal`. Each root cause score tells it WHERE the biggest improvement opportunity is (lowest score = biggest drag on geometric mean). Diagnostics tell the AI agent exactly WHICH files/functions to fix for each root cause.

## Convergence Behavior

With continuous scores and geometric mean:

```
AI agent iteration 1: signal = 0.58 → finds equality is lowest (0.35)
  → refactors god function (CC=75) into 3 smaller functions
  → equality improves to 0.52, signal = 0.63

AI agent iteration 2: signal = 0.63 → finds modularity is lowest (0.55)
  → extracts shared interface, reduces cross-module deps
  → modularity improves to 0.65, signal = 0.69

AI agent iteration 3: signal = 0.69 → finds redundancy is lowest (0.72)
  → removes 8 dead functions
  → redundancy improves to 0.85, signal = 0.74

...diminishing returns...

AI agent iteration N: signal = 0.81 → each change improves by < 0.005
  → natural convergence. Code is as good as static analysis can detect.
```

No artificial stopping point (no "you got an A, stop"). The AI converges naturally when the marginal improvement per change approaches zero — exactly like gradient descent in machine learning.

## Why 5 and Not More?

A directed graph with attributed nodes has exactly these independent structural properties:

| Dimension | What it captures | Edge or Node? |
|---|---|---|
| Modularity Q | Do edges cluster into modules? | Edge |
| Acyclicity | Are there circular edges? | Edge |
| Depth | How deep are edge chains? | Edge |
| Equality (Gini) | Are node properties concentrated? | Node |
| Redundancy | Are there unnecessary nodes? | Node |

3 edge properties + 2 node properties = 5 total.

Adding more would either: (a) be redundant with an existing dimension (entropy overlaps with Gini), or (b) measure something not derivable from static graph analysis (domain correctness, runtime behavior).

## Cross-Language Fairness

Root cause metrics work on the dependency GRAPH, not on language syntax:

| Language | Import syntax | Q works? | Why? |
|---|---|---|---|
| Java | explicit imports | Yes | Edges from imports |
| TypeScript | explicit imports | Yes | Edges from imports |
| Swift | implicit module scope | Yes | Edges from CALL graph |
| Go | explicit package imports | Yes | Edges from imports |
| Rust | mod + use | Yes | Edges from imports |

Newman's Q uses whatever edges exist (import OR call). It measures graph STRUCTURE, not language CONVENTION.

The absolute score may differ across languages (a well-written Swift project might score 75% while a well-written Java project scores 85%), but the TREND (delta over time) is comparable. For the AI agent feedback loop, only the trend matters — the agent optimizes within a single project.

## Theoretical Foundation Summary

| Theory | Year | What it provides |
|---|---|---|
| Cybernetics (Wiener) | 1948 | Feedback loop architecture |
| Systems Engineering (钱学森) | 1954 | Decomposition into independent subsystems |
| Information Theory (Shannon) | 1948 | Measurement bounds, entropy |
| Kolmogorov Complexity | 1963 | Theoretical ground truth (uncomputable) |
| Graph Modularity (Newman) | 2004 | Modularity Q metric |
| Nash Bargaining (Nash) | 1950 | Geometric mean as optimal aggregation |
| Gini Coefficient | 1912 | Inequality measurement |
| Tarjan's Algorithm | 1972 | Cycle detection |
| Lakos Levelization | 1996 | Dependency depth |
| Lyapunov Stability | 1892 | Convergence guarantee for feedback systems |

The quality signal is not an arbitrary score. It is the best computable approximation of the Kolmogorov complexity gap, decomposed into 5 independent graph-theoretic dimensions, aggregated via the Nash-optimal geometric mean, designed for stable convergence in a cybernetic feedback loop.
