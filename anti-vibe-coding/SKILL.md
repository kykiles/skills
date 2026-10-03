---
name: anti-vibe-coding
description: >-
  Direct and audit AI-assisted Rust development using bounded specifications,
  test-first workflows, code generation guardrails, automated verification,
  and a review checklist. Use when writing, modifying, or reviewing Rust code
  with AI, checking AI-generated Rust changes, or controlling architectural
  drift, unsafe code, panic handling, arithmetic, and dependency risks.
---

# Skill: anti-vibe-coding

## System Identity & Core Directive
You are **Anti-Vibe-Coding (Rust Edition)** — a specialized skill and control framework for directing, constraining, and auditing Large Language Model (LLM) agents (e.g., Claude Code, Cursor, Copilot, Windsurf) during Rust software development.

### Core Problem Solved
"Vibe coding" — relying on AI conversational velocity without strict structural validation — leads to "AI Slop": code that compiles and passes happy-path tests, but introduces silent architectural debt, logic bugs, unhandled panics, security vulnerabilities (OWASP Top 10 for LLMs), and supply chain risks (Slopsquatting). In Rust, while the borrow checker eliminates safe memory corruption, AI models frequently introduce:
- Silent integer overflow in release builds (when `overflow-checks` are disabled).
- Denial-of-Service (DoS) vectors via unhandled `.unwrap()` / `.expect()` panics.
- Unsound `unsafe` blocks lacking invariant verification (`// SAFETY:`).
- Hallucinated or vulnerable `Cargo.toml` dependencies (Slopsquatting).
- Circular test validation (AI writing both flawed tests and flawed implementations).
- FFI boundary crashes due to unwinding panics across `extern "C"` boundaries.

---

## Phase 1: Pre-Generation & Context Engineering (Spec & Test-First)

### 1. Spec-Driven & Manifest-Driven Development
Before generating implementation code, the AI agent MUST operate under bounded contracts:
- **Project Rulefiles**: Maintain a mandatory `CLAUDE.md`, `.cursorrules`, or `AGENTS.md` specifying architecture, MSRV (Minimum Supported Rust Version), and formatting rules.
- **Explicit Acceptance Criteria**: Prompts must specify single-user stories with clear boundary conditions, non-goals, and input validation requirements.
- **Architectural Manifest**: Declare allowed modules, public API surfaces, and modified file boundaries upfront. Reject multi-layer drift.

### 2. Test-First Prompting (TDD Protocol)
To prevent "circular validation" (AI writing tests that mirror its own flawed assumptions):
- **Human/Spec Test Definition**: Define expected behaviors, edge cases, and failure modes as tests *before* requesting feature code.
- **Red-Green-Refactor Loop**:
  1. AI generates or receives failing tests (`cargo test` fails).
  2. AI writes minimal code to pass tests (`cargo test` passes).
  3. AI refactors for idiomatic Rust while maintaining green tests.
- **Property-Based Testing**: Require `proptest` or `quickcheck` for critical data transformations to test thousands of adversarial inputs.

### 3. Supply Chain & Slopsquatting Protection
AI models hallucinate non-existent package names ~19.7% of the time.
- **Strict Dependency Allowlist**: Prohibit AI from adding unapproved crates to `Cargo.toml`.
- **Package Verification**: Verify publisher identity, download metrics, and creation date on `crates.io` before accepting any new dependency suggestion.
- **Lockfile Enforcement**: Commit `Cargo.lock` and run `cargo build --locked` in CI.
- **Automated Dependency Audit**: Enforce `cargo-deny` and `cargo-audit` on every change.

---

## Phase 2: Rust Code Generation Guardrails

When generating Rust code, the AI agent MUST strictly adhere to the following 5 engineering pillars:

```
+-----------------------------------------------------------------------------------+
|                           ANTI-VIBE-CODING RUST PILLARS                           |
+--------------------------+--------------------------+-----------------------------+
| 1. Zero Unsound Unsafe   | 2. Fail-Secure Errors    | 3. Type-Driven Design       |
| 4. Explicit Arithmetic   | 5. Deterministic Clean   |                             |
+--------------------------+--------------------------+-----------------------------+
```

### Pillar 1: Unsafe Code Hygiene
- **Forbidden Unsafe**: Apply `#![forbid(unsafe_code)]` in all workspace crates by default.
- **Isolated Boundary**: If `unsafe` is unavoidable (FFI, low-level primitives), isolate it in a minimal, private submodule.
- **Mandatory Safety Documentation**: Every `unsafe` block MUST be preceded by a `// SAFETY:` comment explicitly detailing the invariant upheld.
- **Safe API Wrapper**: Encapsulate `unsafe` operations inside safe abstractions.

### Pillar 2: Fail-Secure Error Handling
- **No `unwrap()` / `expect()` in Production**: Never use `.unwrap()` or `.expect()` on external inputs, network paths, or library code. Turn error conditions into `Result<T, E>`.
- **Crate Separation**:
  - **Libraries**: Use `thiserror` to define typed, matchable error enums. Never leak opaque error types.
  - **Applications/Binaries**: Use `anyhow` for top-level error propagation and `.context()` breadcrumbs.
- **Clippy Enforcement**: Configure `[lints.clippy]` in `Cargo.toml`:
  ```toml
  [lints.clippy]
  unwrap_used = "warn"
  expect_used = "warn"
  ```

### Pillar 3: Type-Driven Design ("Parse, Don't Validate")
- **Newtypes for Domain Identifiers**: Wrap primitives to prevent argument swapping (e.g., `struct UserId(u64)` vs `struct OrderId(u64)`).
- **Parse at Boundaries**: Convert untrusted strings/bytes into validated, strongly-typed representations immediately upon ingress.
- **Enums over Booleans**: Replace multi-boolean flags with explicit, exhaustive `enum` types.
- **Typestate Pattern**: Use generic markers (`Request<Unauthenticated>` -> `Request<Authenticated>`) to enforce state machine invariants at compile time.

### Pillar 4: Integer Overflow & Defensive Limits
- **Release Profile Integrity**: Enable overflow checks in `Cargo.toml`:
  ```toml
  [profile.release]
  overflow-checks = true
  ```
- **Explicit Arithmetic**: Use `.checked_add()`, `.saturating_mul()`, or `.wrapping_sub()` when handling external sizes, offsets, or array capacities. Use `TryFrom` instead of `as` truncation.
- **Resource Limits**: Cap all vector allocations, request body sizes, and recursion depths before touching allocators.

### Pillar 5: FFI & Panic Safety
- **Panic Boundary Isolation**: Never allow Rust panics to unwind across `extern "C"` FFI boundaries. Wrap all FFI entry points in `std::panic::catch_unwind`.
- **Null & Lifetime Checks**: Validate raw pointers for null/alignment and convert C strings safely via `CStr::from_ptr`.

---

## Phase 3: Automated Verification Pipeline

AI-generated Rust code MUST pass a 4-step automated verification loop before human review:

```bash
# Step 1: Formatting & Style Verification
cargo fmt --all -- --check

# Step 2: Strict Linter Check
cargo clippy --all-targets --all-features -- -D warnings

# Step 3: Test Suite & Property Verification
cargo test --locked

# Step 4: Supply Chain & Secret Scanning
cargo audit
cargo deny check
gitleaks detect --staged
```

### Advanced Safety Verification (Where Applicable)
- **Undefined Behavior Check**: Run `cargo +nightly miri test` for any crate containing `unsafe` code.
- **Fuzz Testing**: Run `cargo fuzz` on external parsers and deserializers.

---

## Phase 4: Anti-Vibe Code Reviewer Checklist

When reviewing AI-generated Rust Pull Requests, human reviewers or secondary reviewer sub-agents MUST verify the following checklist:

| Check Item | Requirement | Pass/Fail |
| :--- | :--- | :---: |
| **1. Unsafe Hygiene** | No `unsafe` without `#![allow(unsafe_code)]` exception + `// SAFETY:` comment. | [ ] |
| **2. Panic Hygiene** | Zero `.unwrap()` / `.expect()` in non-test paths. | [ ] |
| **3. Overflow Safety** | Untrusted arithmetic uses `checked_` / `saturating_` / `TryFrom`. | [ ] |
| **4. Dependency Audit** | All new crates in `Cargo.toml` verified on `crates.io` and checked with `cargo-deny`. | [ ] |
| **5. Secrets & Logging** | No hardcoded API keys/tokens. Sensitive fields wrapped in `SecretString` / `zeroize`. | [ ] |
| **6. Architectural Drift** | Code follows domain separation, no duplicated helper functions or orphan patterns. | [ ] |
| **7. Test Quality** | Tests cover failure modes, edge cases, and invalid inputs, not just happy paths. | [ ] |

---

## Example Prompt Template for Secure AI Rust Coding

```markdown
Role: Senior Rust Systems Engineer (Anti-Vibe-Coding Mode)

Task: Implement [Feature Description]

Constraints:
1. Follow "Parse, Don't Validate". Model inputs using strongly-typed domain primitives / Newtypes.
2. Error Handling: Use `thiserror` (for lib) or `anyhow` (for app). NO `.unwrap()` or `.expect()`.
3. Unsafe: `#![forbid(unsafe_code)]` unless explicitly requested. If required, isolate and document `// SAFETY:`.
4. Dependencies: Do NOT add new external crates without listing them and stating justification.
5. Verification: Provide unit tests covering invalid inputs, bounds violations, and expected error variants.
```
