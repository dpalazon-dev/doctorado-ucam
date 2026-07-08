# Agent Runtime

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 2 — Specialized Implementation Model |
| **Normative status** | Canonical for cognitive execution runtime semantics |
| **Authoritative for** | Agent-task contracts, cognitive-operation lifecycle, planning and execution loop, context use, capability routing, model/tool invocation, verification, checkpoints, budgets, stopping conditions, human escalation, recovery and multi-agent execution patterns. |
| **Not authoritative for** | AI Operating Layer purpose, domain authority, component ownership, data-store selection, concrete model providers, prompt content, product-specific SDK behavior or deployment technology. |
| **Required reading** | `SYSTEM_ARCHITECTURE.md`, `SOFTWARE_ARCHITECTURE.md`, `COMPONENT_MODEL.md`, `DATA_ARCHITECTURE.md`, `AI_ARCHITECTURE.md`, `CONTEXT_MODEL.md`, `MEMORY_MODEL.md`, `EVENT_MODEL.md`, `SYSTEM_CAPABILITIES.md`, `DECISIONS.md`. |
| **Downstream documents** | `TECHNICAL_ARCHITECTURE.md`, `SPEC_CATALOG.md`, cognitive-operation Specs, capability/tool Specs, prompt and policy Specs, evaluation suites, runtime code and operational runbooks. |

> An agent is an ephemeral execution role inside the Cognitive Runtime. It is not a source of truth, a persistent identity or an authority boundary.

> **Status: Canonical · v1.1.** This document defines the provider-neutral cognitive execution harness used by ResearchOS.

---

## Purpose

The AI Architecture defines why and where intelligence participates in ResearchOS.

The Software Architecture defines the Cognitive Runtime as propositional and mediated by application contracts.

The Component Model defines the Agent Runtime Host as the component that executes bounded cognitive tasks.

This document specifies **how that runtime behaves**.

It answers:

- How is a cognitive task admitted and represented?
- How does the runtime interpret intent and construct a plan?
- How is context built and isolated?
- How are models and tools selected and invoked?
- How are intermediate results checked?
- What durable state is checkpointed?
- How are budgets, retries and stopping conditions enforced?
- When is human approval required?
- How do one or several specialized agent roles cooperate?
- How does execution recover after failure or restart?

It does not define a chatbot.

It defines a reliable execution harness that turns model and tool capabilities into traceable Artifacts and Proposals while preserving domain and human authority.

---

# Position in the Architecture

```text
AI Architecture
        ↓ defines cognitive responsibilities and authority
Component Model
        ↓ defines Agent Runtime Host and collaborating components
Data Architecture
        ↓ defines operational records and traceability
Agent Runtime
        ↓ defines execution semantics
Technical Architecture
        ↓ maps authority, cognition, IPC, model adapters and isolation to the local desktop runtime
Development Specifications
        ↓ define task-specific contracts, prompts, tools and acceptance tests
```

If this document conflicts with `AI_ARCHITECTURE.md` about authority or purpose, AI Architecture owns the rule.

If it conflicts with `SOFTWARE_ARCHITECTURE.md` about commands, proposals, transactions or effects, Software Architecture owns the rule.

## Current process realization

ADR-0009 and `TECHNICAL_ARCHITECTURE.md` split this logical runtime across two supervised processes without splitting authority:

- the authoritative Rust host owns admission, policy checks, budgets, durable scheduling, checkpoints, cancellation, tool authorization and commitment of accepted Proposals;
- the Python sidecar executes bounded cognitive and scientific steps and returns typed observations, Artifacts and Proposals;
- a model response, Python process or agent framework never becomes a second execution authority or canonical state owner;
- IPC failure is handled as a recoverable capability failure under the lifecycle defined here.

This section adopts the technical placement decision. The lifecycle and verification semantics remain provider- and language-neutral.

---

# Runtime Thesis

> The Agent Runtime is a bounded, durable and policy-controlled interpreter that constructs context, plans actions, invokes typed capabilities, verifies outputs and returns Artifacts or Proposals.

It never owns canonical state.

It never gains permission from model confidence.

It never treats provider conversation history as system memory.

It never converts generated text into validated Knowledge without the Application and Domain authority path.

---

# Core Definitions

## Cognitive Operation

A **Cognitive Operation** is the durable application-level record of one bounded reasoning objective.

Examples:

- extract candidate Knowledge from a Document;
- compare a set of papers;
- answer a question with evidence;
- propose a weekly plan;
- detect possible contradictions;
- draft a thesis section.

It has explicit state, budget, policy, context, evidence and result.

## Agent Task

An **Agent Task** is the normalized input given to the Agent Runtime.

It represents one objective inside a Cognitive Operation.

A Cognitive Operation may execute one or several Agent Tasks.

## Agent

An **agent** is an ephemeral runtime role executing an Agent Task under:

- an objective;
- a capability scope;
- a context package;
- a policy and budget;
- stopping conditions;
- an output contract.

An agent is not:

- a Core Entity;
- a long-lived service identity;
- a memory owner;
- a permission owner;
- an autonomous business authority;
- a persona that justifies a new component.

## Plan

A **Plan** is an explicit, inspectable sequence or graph of bounded steps.

A plan is operational state, not hidden model reasoning.

## Step

A **Step** is one executable unit that may:

- retrieve;
- invoke a model;
- call a capability/tool;
- transform structured data;
- verify a result;
- request approval;
- emit an Artifact or Proposal;
- branch, wait or stop.

## Observation

An **Observation** is the normalized result of a Step.

It is not automatically trusted.

## Artifact

An **Artifact** is a typed cognitive output.

It is transient by default and persists as a Document only through an explicit application transition.

## Proposal

A **Proposal** is a typed recommendation for a Command or Effect Intent.

It is never the mutation itself.

---

# Non-Negotiable Runtime Invariants

1. Canonical state is read through authorized ports and written only through Commands.
2. Context is constructed for the task and never inherited implicitly from a provider session.
3. Capability access is typed, policy-scoped and budgeted.
4. Models cannot invoke arbitrary code or external APIs outside the Capability Registry.
5. Every execution has an explicit maximum cost, time, step and retry budget.
6. Every loop has a stopping condition.
7. Intermediate output is untrusted until verified for its declared purpose.
8. Human approval is enforced outside the model.
9. Hidden chain-of-thought is neither requested as a system dependency nor persisted.
10. Durable progress is represented by checkpoints, actions, observations, evidence and concise rationale.
11. A process restart must not require replaying an entire provider conversation.
12. A failed cognitive operation cannot corrupt canonical state.
13. Provider substitution must not change the semantic runtime contract.
14. Parallelism cannot bypass budgets, authority or verification.
15. Multi-agent execution is an optional pattern, not the default architecture.
16. The least-complex execution pattern capable of satisfying the task contract must be selected.
17. Runtime traces may inform future improvements, but they cannot modify prompts, policies, routing rules or permissions autonomously.
18. Retrieved content and prior runtime experience remain untrusted until their provenance, trust class and intended use are verified.

---

# Progressive Execution Strategy

Research on successful agent harnesses consistently shows that orchestration quality matters more than simply selecting a larger model. ResearchOS adopts that lesson without assuming that every cognitive operation requires a planner, multiple agents or an open-ended loop.

The runtime follows a **complexity ladder**. It begins at the lowest adequate level and escalates only when a recorded insufficiency justifies the additional cost and risk.

| Level | Execution pattern | Appropriate when |
|---|---|---|
| 0 | Deterministic operation | The result can be computed, parsed, validated or retrieved without model reasoning |
| 1 | Single bounded model invocation | One structured inference can satisfy the output contract |
| 2 | Static or template pipeline | The task has known ordered stages |
| 3 | Plan–Act–Verify loop | The path depends on observations or tool results |
| 4 | Controlled fan-out, maker–checker or specialist handoff | Parallelism or independent review measurably improves quality |
| 5 | Debate, roundtable or broader multi-agent coordination | Experimental tasks where simpler patterns have failed under evaluation |

Rules:

- escalation must record the reason the previous level was insufficient;
- a model must not create orchestration complexity merely because it can;
- each level preserves the same authority, budget, checkpoint and verification contracts;
- Level 5 is never a proving-slice dependency;
- de-escalation is preferred when evaluation shows equivalent quality at lower cost or complexity.

---

# High-Level Runtime Topology

```text
Cognitive Operation Manager
        │ admits Agent Task
        ▼
┌──────────────────────────────────────────────────────────────────────┐
│                        Agent Runtime Host                            │
│                                                                      │
│ Intent Resolver → Planner → Context Binding → Step Executor          │
│      ↑              │              │                │                │
│      │              └──── Replan ←─┴── Observation ─┘                │
│      │                                      │                        │
│ Budget/Policy Guard                    Verification                  │
│      │                                      │                        │
│ Checkpoint Manager ← Execution State → Artifact/Proposal Builder     │
└───────────────┬─────────────────────┬────────────────────────────────┘
                │                     │
                ▼                     ▼
       Model Gateway           Capability / Tool Gateway
                │                     │
                └──────────┬──────────┘
                           ▼
                    Normalized Results
```

The Agent Runtime orchestrates these responsibilities. Concrete providers and tools remain adapters.

---

# Core Runtime Contracts

## AgentTask

```text
AgentTask
├── task_id
├── operation_id
├── task_type
├── objective
├── actor_id
├── initiator_type
├── subject_references
├── requested_output_type
├── capability_scope
├── data_access_scope
├── autonomy_mode
├── policy_snapshot
├── budget
├── deadline?
├── correlation_id
├── causation_id
└── contract_version
```

### Rules

- `objective` must be bounded and testable.
- `requested_output_type` must be known before execution.
- capability and data scopes default to deny.
- autonomy mode cannot be widened during execution.
- subject references identify canonical entities; they do not embed mutable aggregate objects.

## RuntimeBudget

```text
RuntimeBudget
├── max_wall_time
├── max_steps
├── max_model_invocations
├── max_tool_invocations
├── max_parallel_branches
├── max_input_tokens?
├── max_output_tokens?
├── max_cost?
├── max_retries_per_step
├── max_replans
└── allowed_provider_classes
```

A missing budget is an admission error, not unlimited permission.

## ResolvedIntent

```text
ResolvedIntent
├── intent_type
├── objective
├── success_criteria
├── output_contract
├── required_evidence
├── relevant_capabilities
├── approval_class
├── risk_class
├── ambiguity_notes
└── confidence
```

Intent confidence helps routing but does not grant authority.

## ExecutionPlan

```text
ExecutionPlan
├── plan_id
├── plan_version
├── objective
├── steps[]
├── dependencies
├── branch_conditions
├── verification_points
├── expected_outputs
├── stopping_conditions
├── budget_allocation
└── created_by
```

## PlanStep

```text
PlanStep
├── step_id
├── step_type
├── objective
├── input_references
├── capability_or_model_request
├── expected_output_schema
├── verification_policy
├── retry_policy
├── timeout
├── dependencies
├── side_effect_class
└── status
```

A PlanStep with an irreversible side effect cannot be executed directly by the Agent Runtime. It may only produce a proposed Effect Intent.

## ContextBinding

```text
ContextBinding
├── context_manifest_id
├── context_digest
├── source_versions
├── access_classification
├── expiry
├── builder_version
└── task_binding
```

## StepObservation

```text
StepObservation
├── step_id
├── source_type
├── normalized_result
├── result_digest
├── evidence_references
├── confidence?
├── warnings
├── timing_and_cost
├── provider_or_tool_identity
└── status
```

## VerificationResult

```text
VerificationResult
├── verification_id
├── subject_reference
├── policy_version
├── checks[]
├── passed
├── severity
├── evidence
├── limitations
├── recommended_action
└── verifier_identity
```

## RuntimeCheckpoint

```text
RuntimeCheckpoint
├── operation_id
├── task_id
├── plan_id_and_version
├── completed_steps
├── pending_steps
├── observations
├── budget_consumed
├── context_binding
├── active_leases?
├── waiting_reason?
├── created_at
└── checkpoint_version
```

The checkpoint contains only state required to resume and audit. It excludes hidden model reasoning.

## AgentResult

```text
AgentResult
├── operation_id
├── status
├── artifact?
├── proposal?
├── answer?
├── evidence
├── verification_result
├── limitations
├── cost_and_latency
├── trace_reference
└── next_action?
```

---

# Cognitive Operation Lifecycle

```text
Requested
    ↓
Admitted
    ↓
Intent Resolved
    ↓
Planned
    ↓
Context Bound
    ↓
Executing
    ↕
Verifying / Replanning
    ↓
Producing Result
    ↓
Completed
```

Alternative states:

```text
Waiting for Input
Waiting for Approval
Paused by Budget
Cancelled
Failed Recoverable
Failed Terminal
Escalated
```

## Requested

The Application or Process layer creates a Cognitive Operation request.

## Admitted

The runtime validates:

- task type;
- actor and policy;
- required output contract;
- capability scope;
- data classification;
- budget;
- runtime availability.

## Intent Resolved

The runtime removes ambiguity sufficient to execute.

If ambiguity changes meaning, authority or expected output, it requests human clarification or returns an explicit ambiguity result.

## Planned

The runtime creates the smallest plan adequate for the task.

Planning may be deterministic, model-assisted or hybrid.

## Context Bound

The Context Builder produces an authorized Context Manifest and transient Context Package.

The task is bound to the context digest and source versions.

## Executing

Steps run according to dependencies, budgets and policy.

## Verifying

Declared checks run at step and final boundaries.

## Replanning

Replanning is allowed only when:

- an observation invalidates an assumption;
- a tool/provider fails;
- evidence is insufficient;
- a verification check fails;
- a branch condition is reached.

Replanning cannot expand the original authority scope.

## Producing Result

Verified observations are transformed into a typed Artifact, Proposal or Answer.

## Completed

The result and trace are persisted. Any canonical mutation still requires the normal Application path.

---

# Admission Protocol

The Agent Runtime accepts work only through the Cognitive Operation Manager.

Admission performs:

1. schema validation;
2. actor and initiator validation;
3. capability-scope validation;
4. data classification and provider eligibility checks;
5. autonomy-policy evaluation;
6. output-contract resolution;
7. budget and deadline assignment;
8. duplicate/idempotency evaluation;
9. correlation and trace initialization;
10. durable operation creation.

## Admission Rejections

- unknown task type;
- unsupported output contract;
- unauthorized data scope;
- prohibited capability;
- missing budget;
- incompatible deadline;
- provider/tool unavailable with no allowed fallback;
- duplicate request with conflicting digest;
- task too broad to satisfy bounded-execution rules.

A broad task may be returned as a suggested decomposition rather than silently accepted.

---

# Intent Resolution

Intent resolution determines what successful completion means.

It may use:

- explicit task type;
- interaction metadata;
- selected entities;
- current workspace state;
- use-case mapping;
- deterministic rules;
- a bounded model classification step.

## Intent Resolution Output

The result must state:

- objective;
- success criteria;
- output type;
- evidence requirement;
- allowed capabilities;
- forbidden actions;
- human-review requirement;
- unresolved ambiguity.

## No Implicit Intent Expansion

A task to “summarize this paper” does not authorize:

- importing other papers;
- modifying Knowledge;
- sending a message;
- changing tasks;
- executing code;
- publishing the summary.

Additional useful actions are returned as proposals.

---

# Planning

## Planning Principle

Use the least complex plan that can succeed and be verified.

A deterministic pipeline is preferred when the task structure is known.

A model-generated plan is used when decomposition genuinely requires interpretation.

## Planning Modes

### Static Pipeline

A predefined sequence for stable tasks such as:

- parse document;
- extract structured metadata;
- validate schema;
- produce candidate records.

### Template Plan

A parameterized plan selected by task type.

### Dynamic Plan

A model-assisted plan for open-ended comparison, synthesis or investigation.

### Hybrid Plan

A fixed outer lifecycle with model-selected bounded inner steps.

Hybrid is the reference approach for ResearchOS.

## Plan Quality Checks

Before execution, the runtime verifies that:

- every step has one objective;
- inputs are available or producible;
- outputs are typed;
- dependencies are acyclic unless an explicit bounded loop exists;
- every loop has a limit;
- capability requests are authorized;
- external effects are proposals only;
- verification points exist for high-risk claims;
- the plan fits the budget;
- a terminal condition exists.

## Replanning Limits

Replanning is bounded by:

- maximum replan count;
- remaining budget;
- no repeated identical failed plan;
- explicit reason for change;
- checkpoint before material plan replacement.

---

# Context Binding

The Agent Runtime does not assemble context itself. It requests it from the Context Builder.

## Context Request

The request includes:

- task and resolved intent;
- subject references;
- required evidence classes;
- time horizon;
- relationship depth;
- data classification;
- provider eligibility;
- token/size budget;
- freshness and consistency requirement;
- exclusions.

## Context Use Rules

- the runtime consumes only the bound Context Package;
- additional retrieval is a capability call recorded as a new observation;
- source identifiers and versions remain attached to extracted claims;
- stale context triggers rebuild or explicit limitation;
- context from one task is not reused for another without a new binding;
- provider cache or chat history is never a substitute for a Context Manifest;
- sensitive context is minimized and expires.

## Context Isolation

Parallel tasks receive separate context bindings even when they share sources.

A tool or model sees only the subset required for its Step.

## Retrieval Escalation

The runtime requests retrieval through the Context Builder using a progressive evidence ladder. The ladder is a strategy, not a mandatory sequence for every task.

```text
Canonical references and structured filters
        ↓ if insufficient
Lexical retrieval
        ↓ if insufficient
Semantic/vector retrieval
        ↓ if insufficient
Relationship/graph traversal
        ↓ if policy permits and freshness requires it
Authorized external retrieval
```

Each escalation must preserve:

- source identity and version;
- data classification and actor scope;
- retrieval method and generation;
- sufficiency rationale;
- score or ranking evidence where applicable;
- the distinction between validated Knowledge, candidate Knowledge and raw source content.

Hybrid retrieval may combine several stages, but fusion must remain inspectable. A model-generated answer is never allowed to hide which retrieval channels supplied its evidence.

---

# Execution Loop

The canonical loop is:

```text
Plan
    ↓
Select next ready Step
    ↓
Authorize and bind inputs
    ↓
Act through model/capability/deterministic code
    ↓
Observe normalized result
    ↓
Verify
    ├── pass → persist checkpoint and continue
    ├── revise → retry or replan within budget
    ├── wait → input/approval/timer
    └── fail → recover, escalate or terminate
```

## Step Types

| Type | Purpose |
|---|---|
| Deterministic transform | parse, normalize, calculate, map, validate |
| Retrieval | query canonical/projection/external authorized sources |
| Model reasoning | classify, compare, synthesize, draft, critique |
| Capability/tool | execute a typed system or external operation |
| Verification | check schema, evidence, policy, computation or quality |
| Human gate | request clarification, review or approval |
| Coordination | fan-out, join, branch, wait, checkpoint |
| Result construction | build Artifact, Proposal or Answer |

## Step Atomicity

A Step is complete only when:

- the invocation result is recorded;
- the normalized observation is durable when recovery requires it;
- verification status is known or explicitly deferred;
- budget consumption is updated;
- the checkpoint is persisted.

## No Direct Canonical Write

A Step may prepare a Command payload or Proposal.

It may not call a canonical repository adapter.

---

# Capability Routing

The Capability Registry exposes stable system verbs and implementations.

## Capability Descriptor

```text
CapabilityDescriptor
├── capability_name
├── capability_version
├── input_schema
├── output_schema
├── effect_class
├── reversibility
├── required_permissions
├── allowed_data_classes
├── isolation_requirement
├── idempotency_support
├── cost/latency profile
├── implementation_candidates
└── verification policy
```

## Routing Inputs

The Tool Gateway selects an implementation using:

- required capability;
- input/output compatibility;
- policy and actor scope;
- data classification;
- runtime isolation;
- cost and latency budget;
- availability and health;
- experiment configuration;
- deterministic preference;
- prior failure in the same operation.

## Routing Rules

- deterministic tools are preferred over model inference for deterministic tasks;
- local/private tools are preferred when data policy forbids external processing;
- fallback must preserve output semantics;
- implementation identity is recorded;
- a fallback cannot widen permissions;
- arbitrary shell, HTTP or database access is not a generic capability.

---

# Model Runtime Interaction

The Agent Runtime invokes models only through the Model Gateway.

## Model Request

A normalized request contains:

- task/step objective;
- structured instructions and version;
- selected Context subset;
- output schema;
- available capability descriptions, if any;
- safety and data policy;
- temperature/determinism class where supported;
- token/time budget;
- correlation and trace identity.

## Model Result

The normalized result contains:

- structured output or parse failure;
- provider/model identity;
- finish reason;
- usage/cost metrics;
- safety/filter information;
- tool-call proposals;
- raw body reference or digest according to policy;
- latency and error class.

## Model Selection

Selection may consider:

- task class;
- context size;
- structured-output reliability;
- tool-use capability;
- data residency/privacy;
- latency;
- cost;
- evaluation results;
- current availability.

Model prestige is not a routing criterion.

## Provider State

Provider-managed conversation or thread state may be used as an optimization only if:

- the runtime can reconstruct the task without it;
- all authoritative execution state exists internally;
- provider state identity is recorded;
- loss of the provider thread is recoverable.

## Runtime Experience Boundary

Operation traces may reveal reusable execution knowledge such as:

- a parser or provider failure pattern;
- a routing decision that repeatedly underperforms;
- a prompt or schema that causes structured-output failures;
- a recovery sequence that consistently succeeds;
- cost and latency characteristics for a task class.

These observations are **runtime experience candidates**, not trusted memory and not self-modifying policy.

Promotion requires:

1. aggregation across sufficient executions;
2. removal of sensitive task content;
3. explicit evaluation against a versioned dataset;
4. human or governed approval;
5. a versioned change to instructions, routing policy, capability configuration or code.

The runtime may retrieve approved experience records to support routing or recovery. It may not rewrite its own prompts, permissions, stopping rules or capability registry during an operation.

---

# Tool Execution

## Tool Request Protocol

1. Validate capability and parameters.
2. Check actor/autonomy policy.
3. Check data/provider classification.
4. Check budget and rate limit.
5. Determine isolation requirement.
6. Assign idempotency key if applicable.
7. Execute through Tool Gateway.
8. Capture normalized result and evidence.
9. Verify declared postconditions.
10. Persist observation and checkpoint.

## Tool Result Classes

- success;
- partial success;
- transient failure;
- rate limited;
- invalid input;
- unauthorized;
- unsafe/prohibited;
- timeout with known no-effect;
- ambiguous effect;
- malformed/untrusted output.

## Untrusted Output

Tool output is untrusted data.

Browser pages, PDFs, emails and external text may contain prompt-injection attempts or malicious instructions.

The runtime treats them as evidence/content, not as privileged instructions.

---

# Verification Architecture

Verification is proportional to claim and effect risk.

## Verification Layers

### 1. Structural Verification

- schema validity;
- required fields;
- type and cardinality;
- reference resolution;
- no unknown commands/capabilities.

### 2. Deterministic Verification

- calculations;
- parser consistency;
- digest and identifier checks;
- domain preconditions;
- policy rules;
- output constraints.

### 3. Evidence Verification

- every material claim maps to a source;
- quoted text exists at the cited location;
- source version matches the Context Manifest;
- evidence actually supports the claim;
- contradictions and uncertainty are surfaced.

### 4. Cross-Source Verification

Used when a conclusion depends on multiple sources or external freshness.

### 5. Model-Based Critique

A separate bounded model review may identify omissions, inconsistencies or unsupported claims.

It is advisory unless paired with deterministic/evidence checks.

### 6. Human Verification

Required for:

- promotion to validated Knowledge when policy demands;
- irreversible or high-impact effects;
- scientific judgement beyond configured autonomy;
- unresolved conflicting evidence;
- low-confidence identity/entity resolution;
- output intended for publication without review.

## Verification Independence

Verification should be as independent from the producing mechanism as the task risk requires.

Preferred order:

1. deterministic checks;
2. direct source/evidence checks;
3. independent retrieval or cross-source comparison;
4. a separately instructed model-based critic;
5. human review.

A second call to the same model with the same evidence is not treated as independent proof. Self-consistency voting may estimate stability, but majority agreement does not establish truth. Provider diversity is optional and must be justified by measured quality, risk and cost rather than used as ritual redundancy.

## Verification Outcome

Verification may return:

- pass;
- pass with limitations;
- revise;
- insufficient evidence;
- conflicting evidence;
- policy rejection;
- human review required;
- terminal failure.

A verifier does not silently edit the subject. Revision creates a new output version.

---

# Artifact and Proposal Production

## Artifact Production

An Artifact includes:

- type;
- content/body reference;
- objective;
- source evidence;
- provenance;
- limitations;
- verification result;
- suggested persistence/use;
- producing operation.

Examples:

- summary;
- comparison table;
- answer;
- report draft;
- search strategy;
- plan draft.

## Proposal Production

A Proposal includes:

- intended Command or Effect type;
- exact subject/payload digest;
- reason;
- evidence;
- risk/reversibility;
- confidence;
- requested reviewer;
- expiry;
- changes from prior proposal version.

## Result Boundaries

- Artifact → may remain transient or be persisted as Document.
- Knowledge candidate → Proposal; acceptance issues a Command.
- Task/Project change → Proposal; acceptance issues a Command.
- External action → Effect Intent proposal; execution follows the effect protocol.
- Answer → Artifact with evidence; not Knowledge by default.

---

# Human Interaction and Approval

The Agent Runtime may pause for:

- missing task input;
- ambiguity clarification;
- capability authorization;
- proposal approval;
- conflicting evidence judgement;
- budget extension;
- external-effect confirmation.

## Waiting State

A waiting operation records:

- reason;
- exact requested input/decision;
- current result preview;
- subject digest;
- expiry/deadline;
- safe resume point;
- consequence of no response.

## Approval Resume

On approval:

- verify actor authority;
- verify proposal digest and expiry;
- record decision;
- issue the approved Command/Effect outside the model;
- observe resulting Event/Effect Result;
- update operation result.

The model does not interpret a conversational “yes” as approval unless the Interaction/Application layer resolves it into an explicit approval decision.

---

# Autonomy

Autonomy applies per behaviour, not per agent.

| Mode | Runtime behavior |
|---|---|
| Observe | may read and report only |
| Suggest | may produce Artifact or Proposal |
| Assist | may prepare exact action awaiting approval |
| Pre-authorized | may issue bounded commands/effects under recorded policy |
| Prohibited | cannot execute or propose the behaviour in the current context |

## Autonomy Rules

- no self-escalation;
- no inheritance from another task;
- no expansion through replanning;
- revocation takes effect at the next policy boundary and before any effect;
- pre-authorization has scope, expiry, budget and reversibility constraints;
- high-impact scientific or personal decisions remain human-owned.

---

# Durable Execution and Checkpointing

## Checkpoint Triggers

Checkpoint after:

- plan creation or material replan;
- context binding;
- each expensive or non-repeatable Step;
- capability/tool invocation;
- verification result;
- budget threshold;
- transition to waiting;
- branch fan-out/fan-in;
- result construction;
- failure or cancellation.

## Resume Protocol

1. Load Cognitive Operation and latest checkpoint.
2. Verify definition, policy and capability versions remain compatible.
3. Revalidate actor/autonomy state where required.
4. Verify Context sources are still valid or rebuild context.
5. Reconcile any in-flight ambiguous tool/effect invocation.
6. Reacquire leases.
7. Continue from the first incomplete safe Step.

## Definition Drift

If prompt, plan template, capability schema or runtime version changed incompatibly:

- resume with the recorded old version when available;
- migrate through an explicit operation;
- or terminate and restart with user-visible reason.

Silent reinterpretation of an old checkpoint is prohibited.

---

# Failure and Recovery

## Failure Classes

| Class | Example | Default response |
|---|---|---|
| Admission | unsupported task | reject without execution |
| Context | missing/unauthorized sources | rebuild, narrow or request input |
| Provider | outage/rate limit | retry/fallback within policy |
| Tool | parser/API failure | retry, alternate implementation or partial result |
| Structured output | invalid schema | repair attempt then retry/replan |
| Verification | unsupported claim | revise, gather evidence or return limitation |
| Budget | cost/step/time exhausted | pause, partial result or escalate |
| Policy | prohibited data/effect | terminate that path and audit |
| Concurrency | source changed | rebuild context or abandon stale proposal |
| Ambiguous effect | timeout after external call | reconcile before retry |
| Runtime | process crash | resume from checkpoint |
| Quality | repeated low-quality output | alternate strategy/model or human escalation |

## Retry Rules

A retry declares:

- failure class;
- changed condition or backoff;
- maximum attempts;
- idempotency strategy;
- remaining budget;
- escalation path.

Repeating the same prompt/tool call without changed conditions is not a recovery strategy.

## Partial Results

A partial result is acceptable when:

- the output contract permits it;
- missing sections are explicit;
- evidence and limitations are preserved;
- no unsupported completeness claim is made;
- the user/process can decide whether to continue.

---

# Budgets and Stopping Conditions

## Budget Enforcement

Budget is checked:

- at admission;
- before each model/tool call;
- before fan-out;
- after each observation;
- before replan/retry;
- before result construction.

## Stopping Conditions

An operation stops when one of these is true:

- success criteria met and verified;
- terminal output contract satisfied with declared limitations;
- human decision required;
- budget exhausted;
- deadline reached;
- no authorized next action exists;
- repeated strategy failure threshold reached;
- policy violation;
- cancellation requested;
- external ambiguity requires manual reconciliation.

## Loop Detection

The runtime detects:

- repeated equivalent plan;
- repeated identical capability call and result;
- no-progress cycles;
- oscillating revisions;
- recursive delegation;
- repeated unsupported claim.

Detected loops terminate or escalate; they do not consume the remaining budget indefinitely.

---

# Parallel and Multi-Agent Execution

## Default

One Agent Task executes in one coordinated runtime state.

Specialized roles are introduced only when they improve measurable output or isolation.

## Supported Patterns

### Sequential Pipeline

Planner → Retriever → Analyst → Verifier.

Use when dependencies are ordered and deterministic.

### Fan-Out / Fan-In

Independent branches analyze separate documents or perspectives, then a join Step combines verified outputs.

Use when work is genuinely parallel and budget permits.

### Maker–Checker

One role produces; another verifies against an explicit rubric and evidence.

Use for higher-risk synthesis or extraction.

### Specialist Handoff

A coordinator delegates one bounded subtask to a specialist role with narrower context and capabilities.

The specialist returns an Observation, not authority.

### Debate / Roundtable

Multiple model roles compare proposals under a moderator/verifier.

This is experimental and not part of the proving-slice baseline.

## Multi-Agent Rules

- one Cognitive Operation owns the global budget;
- each branch has a sub-budget and capability scope;
- contexts are isolated;
- agents do not communicate through hidden provider memory;
- inter-agent messages are typed observations or tasks;
- fan-in verifies source/evidence conflicts;
- one coordinator owns termination;
- no recursive unbounded delegation;
- persistent state remains in runtime records, not “agent memory.”

---

# Security and Trust

## Prompt Injection Defense

Untrusted content is delimited and classified as data.

The runtime:

- separates system instructions from retrieved content;
- never follows instructions found in documents, emails or webpages unless explicitly authorized as task data;
- minimizes exposed capability descriptions;
- requires tool authorization outside model output;
- validates every tool argument;
- filters secret and restricted data;
- records suspicious content indicators;
- may route high-risk content to isolated processing.

## Capability Security

- deny by default;
- least privilege per task and Step;
- scoped credentials;
- no generic unrestricted shell/HTTP/database tool;
- isolated execution for untrusted code or browser actions;
- output size and content limits;
- network and filesystem restrictions;
- explicit effect classification.

## Data Security

- context selection applies classification before relevance;
- provider eligibility is policy-controlled;
- prompts and logs are redacted according to classification;
- temporary files and context expire;
- model training/data-retention settings are treated as provider policy, not assumed.

## Memory and Retrieval Poisoning

Persistent memory and retrieval indexes are treated as attack surfaces.

The runtime must:

- retain provenance and trust class for every retrieved item;
- prevent raw model output or external content from becoming approved runtime experience automatically;
- isolate candidate/inferred relationships from validated relationships;
- detect instructions embedded in documents, emails or webpages and classify them as untrusted content;
- support revocation and re-indexing when a source is deleted, corrected or reclassified;
- avoid promoting repeated content merely because repetition increases retrieval frequency;
- record which memory or projection entries influenced an output.

---

# Observability and Traceability

Every operation propagates:

- operation ID;
- task ID;
- actor;
- correlation ID;
- causation ID;
- plan and step identity;
- context manifest;
- model/tool/capability identity;
- budget consumption;
- verification result;
- Artifact/Proposal identifiers;
- resulting Command/Event/Effect references.

## Required Metrics

- operation success/partial/failure rate;
- latency by task and Step type;
- model/tool invocation count;
- token and monetary cost;
- retry and replan count;
- context build latency and source count;
- structured-output failure;
- verification failure and unsupported-claim rate;
- human approval/rejection/edit rate;
- citation validity;
- budget exhaustion;
- provider/tool fallback rate;
- checkpoint recovery success.

## Trace Privacy

Trace does not require storing private hidden reasoning.

It stores observable decisions, actions, evidence, concise rationale and outcomes.

---

# Evaluation

The Agent Runtime is evaluated at three levels.

## Component Evaluation

- intent classification accuracy;
- plan validity;
- context source precision/recall;
- structured output compliance;
- tool routing correctness;
- verifier precision;
- recovery behavior.

## Task Evaluation

- success criteria;
- evidence coverage;
- factual correctness;
- citation validity;
- completeness;
- human edit distance;
- latency and cost;
- policy compliance.

## System Evaluation

- canonical-state safety;
- no unauthorized effects;
- recovery after restart;
- reproducibility from trace and source versions;
- provider replaceability;
- usefulness compared with simpler deterministic or RAG baselines.

Evaluation datasets and rubrics are versioned.

A model change is not accepted solely because subjective output appears better.

## Benchmark Strategy

The **ResearchOS internal evaluation suite** is the release authority because it measures the actual domain contracts, evidence requirements, approval rules and failure modes of the system.

External benchmarks may be used as diagnostics:

- GAIA for general multi-step tool use;
- AgentBench for environment-specific execution;
- τ-bench-style scenarios for policy compliance and state consistency;
- WebArena or OSWorld only when browser or operating-system control becomes an approved capability;
- retrieval-focused suites for context precision, recall, attribution and freshness.

They do not replace internal acceptance tests and are not architectural targets by themselves.

Versioned evaluation reports should include, where relevant:

- pass@1 and repeated-run stability/pass^k;
- policy-compliance rate;
- retrieval precision/recall and stale-retrieval rate;
- unsupported-claim and citation-validity rate;
- human acceptance, edit and rejection rate;
- latency, token use and monetary cost;
- recovery and checkpoint success;
- comparison with deterministic, single-call and simple-RAG baselines.

---

# Proving-Slice Runtime

The first Agent Runtime implementation supports two operation types.

## Operation A · Extract Knowledge Candidates

```text
DocumentProcessed event
    ↓
Create Cognitive Operation
    ↓
Resolve extraction intent from template
    ↓
Build context from one Document version
    ↓
Execute deterministic sectioning + model extraction
    ↓
Verify schema, source spans and candidate duplication
    ↓
Produce Knowledge candidate Proposals
    ↓
Human approves/rejects/edits
    ↓
Application issues CreateKnowledge / RelateKnowledge Commands
```

### Required output

Each candidate includes:

- type;
- concise statement;
- source Document/version/segment;
- evidence excerpt reference;
- confidence;
- relationship suggestions;
- limitations;
- duplicate/overlap result;
- verification status.

## Operation B · Answer with Evidence

```text
User question
    ↓
Resolve intent and evidence requirement
    ↓
Build domain-grounded context
    ↓
Retrieve lexical baseline; later vector/graph variants
    ↓
Generate structured answer
    ↓
Verify every material claim against source evidence
    ↓
Return Answer Artifact + Context Manifest + limitations
```

### Required comparison

Experiment 2 compares:

- simple lexical/RAG baseline;
- domain-grounded context;
- later hybrid retrieval if justified.

The runtime architecture must not bias evaluation by hiding different source sets or budgets.

---

# Initial Runtime Scope

## Required

- durable Cognitive Operation state;
- template/hybrid plan;
- Context Builder integration;
- one model adapter;
- deterministic parser/tool capability;
- typed structured outputs;
- checkpoint after each expensive Step;
- schema and evidence verification;
- Proposal/Artifact production;
- explicit human approval;
- budget and timeout;
- correlated telemetry;
- restart recovery.

## Deferred

- autonomous long-horizon research;
- open-ended web browsing;
- arbitrary code execution;
- debate agents;
- recursive delegation;
- self-modifying prompts/policies;
- automatic model training;
- generalized multi-agent marketplace;
- permanent agent identities;
- fully autonomous publication or communication;
- external effects beyond tightly bounded reversible actions.

---

# Specifications Required Before Development

Each cognitive operation specification must define:

1. operation identifier and version;
2. owning use case;
3. objective and success criteria;
4. allowed input/domain references;
5. output contract;
6. plan template or planning mode;
7. context request policy;
8. allowed capabilities/tools;
9. model requirements and provider eligibility;
10. verification rubric;
11. autonomy and approval mode;
12. budget and stopping conditions;
13. retry/recovery behavior;
14. telemetry and audit requirements;
15. evaluation dataset and acceptance thresholds;
16. resulting proposals/commands/effects;
17. privacy and retention behavior.

A prompt alone is not an operation specification.

---

# Conformance Criteria

An Agent Runtime implementation conforms only if:

1. every task is admitted through a durable Cognitive Operation;
2. every task has explicit output, policy, scope and budget;
3. context is bound through a traceable Context Manifest;
4. the runtime can resume without provider conversation state;
5. plans and actions are inspectable without storing hidden chain-of-thought;
6. capabilities are typed and deny-by-default;
7. models cannot call providers/tools outside gateways;
8. deterministic tools are preferred for deterministic work;
9. every loop, retry and replan is bounded;
10. high-risk claims and actions have declared verification;
11. cognitive output remains Artifact/Proposal until the Application/Domain path commits change;
12. approval is digest-bound and enforced outside the model;
13. failure cannot corrupt canonical state;
14. ambiguous external effects are reconciled before retry;
15. parallel branches share one controlled budget and termination owner;
16. traceability connects sources, context, steps, results and resulting Commands/Events;
17. sensitive data obeys provider/tool eligibility policy;
18. the proving-slice operations survive process restart;
19. provider substitution preserves the same runtime contracts;
20. evaluation can compare the runtime against a simpler baseline.

---

# Evolution Rules

A new agent role requires:

- a distinct bounded responsibility;
- narrower context or capability scope that improves safety or quality;
- measurable benefit over an existing pipeline;
- explicit input/output contract;
- budget and stopping conditions;
- evaluation evidence.

A new model provider requires an adapter and evaluation, not runtime redesign.

A new capability requires canonical capability vocabulary, policy, schema, effect class and verification.

A new orchestration pattern requires evidence that simpler sequential or hybrid execution is insufficient.

Runtime self-modification, persistent autonomous identities and unrestricted tool access are prohibited unless future architecture and ADRs explicitly replace these rules.

---

# Final Runtime Statement

The ResearchOS Agent Runtime is a controlled execution harness, not an autonomous authority.

It receives bounded tasks.

It resolves intent, builds explicit context, plans inspectable steps, invokes typed capabilities, records observations, verifies evidence and returns Artifacts or Proposals.

It survives failure through durable checkpoints.

It stops through budgets and policies.

It collaborates with humans through explicit approval.

It may use one model, several models or no model at all, but it always operates beneath the Domain and Application authority boundaries.

This runtime is how ResearchOS gains intelligence without surrendering coherence.
