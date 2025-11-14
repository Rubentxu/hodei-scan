### **Universal Glossary of hodei-scan**

#### A

*   **Adapter (Level 1 Extractor):** A type of extractor whose only job is to execute a third-party analysis tool (e.g., `Ruff`, `ESLint`) and **translate** its report into Atomic Facts in `hodei-scan` IR format. It's the fastest way to add rule coverage for a new language or tool.
*   **Differential Analysis:** The process of analyzing only the files that have changed between two points in time (e.g., between two commits), using a cache to avoid re-analyzing code that hasn't been modified.
*   **Data-Flow Analysis (DFA):** A deep analysis technique that models how data (values) flows through the variables and functions of a program. It's the foundation of Taint Analysis. In our analogy, it's the "pipe map".
*   **Arena Allocator:** A memory management strategy that allocates large blocks of memory at once and then distributes small pieces from that block. It's much faster than requesting memory from the operating system for each small object. We use it to optimize the creation of the `SemanticModel`.
*   **AST (Abstract Syntax Tree):** A tree-shaped representation of the grammatical structure of source code. It's the basic "3D blueprint" of a code file.

#### B

*   **Governance Backend (`hodei-server`):** The centralized and stateful server component of the platform. It acts as the "Central Archive" and "Strategic Center", managing the central cache, policies, analysis history, and dashboards.
*   **Baselining:** The process of marking a set of existing `Findings` at a point in time (e.g., in the `main` branch) as "accepted technical debt". This allows future analyses in feature branches to only fail for **new** problems introduced.

#### C

*   **Central Cache:** A store of analysis results (Partial IRs) managed by `hodei-server`. It allows sharing analysis work across the entire team and CI/CD pipelines.
*   **Hybrid Cache:** The strategy of the `hodei-scan` CLI that combines a local cache (on the user's machine, for instant speed) and the central cache (for sharing work). The order is: `Local -> Central -> Execute`.
*   **Local Cache:** A store of Partial IRs on the user's local machine (e.g., in `~/.cache/hodei`), used to speed up repeated executions of the same analysis.