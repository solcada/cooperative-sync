# Positioning: Architecture vs Ownership Model

## Core distinction
Distributed system design and cooperative ownership are separate choices.
- Architecture answers: "How does the product technically operate?"
- Ownership answers: "Who governs it and who benefits economically?"

A product can be cloud-centralized in v1 and still be genuinely cooperative in governance and outcomes.

## 1) Product architecture (what we build)

### v1: Cloud-first backup service
- One-way backup from client folder to managed cloud storage.
- Central control plane for auth, metadata, restore jobs, and billing.
- Object storage for file data; relational database for metadata.

### Why this is first
- Fastest path to reliable customer value.
- Best supportability and operational predictability.
- Strongest early unit economics (low complexity, clear margins).

### v1.5: Bring-your-own-node (optional, trusted mode)
- Users can contribute personal nodes for their own data (or invited group).
- Positioned as power-user/family/team feature, not default path.
- Maintains cloud fallback for reliability and restore speed.

### v2+: Federation experiments
- Optional multi-pool or community storage participation.
- Only after reliability, abuse controls, and economics are proven.

## 2) Cooperative ownership/governance (how we run it)

### Principles
- Member ownership and transparent governance.
- Clear policy on pricing, reserves, and reinvestment.
- Benefit sharing tied to long-term sustainability, not growth at all costs.

### Governance mechanisms (draft)
- Member voting rights on major policy changes.
- Public metrics reporting (reliability, pricing changes, reserve policy).
- Defined process for electing/rotating board or stewardship group.

### Economic model (draft)
- Service priced for healthy operations and reliability.
- Surplus allocation policy: reserves, member dividends/credits, reinvestment.
- Optional contributor credits for verified node participation in later phases.

## 3) How to communicate this externally

### Short message
"Cloud-simple backup, cooperatively owned."

### Expanded message
- We start with the most reliable and affordable architecture.
- We operate with cooperative governance and transparent economics.
- We add decentralization features only where they improve member value without harming reliability.

## 4) Decision guardrails

Use these checks before adopting distributed/federated features:
- Reliability: does this improve or degrade restore success and time-to-recovery?
- Cost: does this lower total cost of service at equal reliability?
- Support: can non-expert users understand and recover from failure modes?
- Security: can we maintain strong end-to-end privacy and abuse controls?
- Governance fit: does it reinforce cooperative goals or add technical complexity without member benefit?

## 5) Immediate next steps
1. Ship v1 cloud backup + restore with strong reliability SLAs.
2. Publish cooperative governance and surplus-allocation draft.
3. Define BYO node alpha criteria (trusted users only, explicit constraints).
4. Re-evaluate federation after measurable v1 traction and support data.
