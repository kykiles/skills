# Навыки Matt Pocock

Источник: https://github.com/mattpocock/skills, коммит `b0618bc436ad893b3c5e84e55fba86586d34a404`.

Импортированы без изменения инструкций; вспомогательные файлы и `agents/openai.yaml` сохранены. Лицензия MIT — [mattpocock-skills.LICENSE](../licenses/mattpocock-skills.LICENSE). Обновления вручную, автоматической синхронизации нет.

Перед использованием инженерных навыков запустите `/setup-matt-pocock-skills` в целевом проекте. Некоторые навыки требуют инструментов конкретного клиента (например, Skill tool, субагентов или `claude --bg`); импорт в репозиторий не устанавливает их в клиент автоматически.

Категории `misc` и `in-progress` не входят в основной плагин автора. Экспериментальные навыки могут меняться или исчезать.

## Разработка

| Навык | Описание автора |
| --- | --- |
| [`ask-matt`](../ask-matt/SKILL.md) | Ask which skill or flow fits your situation. A router over the skills in this repo. |
| [`code-review`](../code-review/SKILL.md) | Review the changes since a fixed point (commit, branch, tag, or merge-base) along two axes: Standards (does the code follow this repo's documented coding standards?) and Spec (does the code match what the originating issue/spec asked for?). Runs both reviews in parallel sub-agents and reports them side by side. Use when the user wants to review a branch, a PR, work-in-progress changes, or asks to "review since X". |
| [`codebase-design`](../codebase-design/SKILL.md) | Shared vocabulary for designing deep modules. Use when the user wants to design or improve a module's interface, find deepening opportunities, decide where a seam goes, make code more testable or AI-navigable, or when another skill needs the deep-module vocabulary. |
| [`diagnosing-bugs`](../diagnosing-bugs/SKILL.md) | Diagnosis loop for hard bugs and performance regressions. Use when the user says "diagnose"/"debug this", or reports something broken/throwing/failing/slow. |
| [`domain-modeling`](../domain-modeling/SKILL.md) | Build and sharpen a project's domain model. Use when discussing codebase terminology, writing or editing a GLOSSARY.md, or recording or editing an ADR. |
| [`grill-with-docs`](../grill-with-docs/SKILL.md) | A relentless interview to sharpen a plan or design, which also creates docs (ADR's and glossary) as we go. |
| [`implement`](../implement/SKILL.md) | Implement a piece of work based on a spec or set of tickets. |
| [`implement-spec`](../implement-spec/SKILL.md) | Implement the result of /to-spec and /to-tickets in code. |
| [`improve-codebase-architecture`](../improve-codebase-architecture/SKILL.md) | Scan a codebase for deepening opportunities, present them as a visual HTML report, then grill through whichever one you pick. |
| [`pr`](../pr/SKILL.md) | Use when writing a PR body. |
| [`prototype`](../prototype/SKILL.md) | Build a throwaway prototype to answer a design question. Use when the user wants to sanity-check whether a state model or logic feels right, or explore what a UI should look like. |
| [`research`](../research/SKILL.md) | Investigate a question against high-trust primary sources and capture the findings as a Markdown file in the repo. Use when the user wants a topic researched, docs or API facts gathered, or reading legwork delegated to a background agent. |
| [`retro`](../retro/SKILL.md) | Conduct a retrospective on a coding session. |
| [`setup-matt-pocock-skills`](../setup-matt-pocock-skills/SKILL.md) | Configure this repo for the engineering skills: set up its issue tracker, triage label vocabulary, and domain doc layout. Run once before first use of the other engineering skills. |
| [`tdd`](../tdd/SKILL.md) | Test-driven development. Use when the user wants to build features or fix bugs test-first, mentions "red-green-refactor", or wants integration tests. |
| [`to-spec`](../to-spec/SKILL.md) | Turn the current conversation into a spec and publish it to the project issue tracker: no interview, just synthesis of what you've already discussed. |
| [`to-tickets`](../to-tickets/SKILL.md) | Break a plan, spec, or the current conversation into a set of tracer-bullet tickets, each declaring its blocking edges, published to the configured tracker (edges as text in one file per ticket locally, or native blocking links on a real tracker). |
| [`triage`](../triage/SKILL.md) | Move issues and external PRs through a state machine of triage roles, categorise, verify, grill if needed, and write agent-ready briefs. |
| [`wayfinder`](../wayfinder/SKILL.md) | Plan a huge chunk of work (more than one agent session can hold) as a shared map of decision tickets on your issue tracker, and resolve them one at a time until the way to the destination is clear. |
| [`wizard`](../wizard/SKILL.md) | Generate an interactive bash wizard that walks a human through steps only they can perform. Use when provisioning infrastructure, setting up credentials or CI secrets, walking an unfamiliar third-party dashboard, or running a one-off migration or cutover. Don't invoke this for steps the agent can perform itself. |

## Рабочий процесс

| Навык | Описание автора |
| --- | --- |
| [`grill-me`](../grill-me/SKILL.md) | A relentless interview to sharpen a plan or design. |
| [`grilling`](../grilling/SKILL.md) | Grill the user relentlessly about a plan, decision, or idea. Use when the user wants to stress-test their thinking, or uses any 'grill' trigger phrases. |
| [`handoff`](../handoff/SKILL.md) | Compact the current conversation into a handoff document for another agent to pick up. |
| [`teach`](../teach/SKILL.md) | Teach the user a new skill or concept, within this workspace. |
| [`to-questionnaire`](../to-questionnaire/SKILL.md) | Turn a decision you can't fully answer into a questionnaire for someone else to fill in. |
| [`wait-what`](../wait-what/SKILL.md) | Stop. That last message did not land: re-pitch it. |
| [`writing-for-agents`](../writing-for-agents/SKILL.md) | Writing documents for agents. Use when creating or editing skills, or modifying AGENTS.md or CLAUDE.md. |

## Дополнительные

| Навык | Описание автора |
| --- | --- |
| [`git-guardrails-claude-code`](../git-guardrails-claude-code/SKILL.md) | Set up Claude Code hooks to block dangerous git commands (push, reset --hard, clean, branch -D, etc.) before they execute. Use when user wants to prevent destructive git operations, add git safety hooks, or block git push/reset in Claude Code. |
| [`migrate-to-shoehorn`](../migrate-to-shoehorn/SKILL.md) | Migrate test files from `as` type assertions to @total-typescript/shoehorn. Use when user mentions shoehorn, wants to replace `as` in tests, or needs partial test data. |
| [`scaffold-exercises`](../scaffold-exercises/SKILL.md) | Create exercise directory structures with sections, problems, solutions, and explainers that pass linting. Use when user wants to scaffold exercises, create exercise stubs, or set up a new course section. |
| [`setup-pre-commit`](../setup-pre-commit/SKILL.md) | Set up Husky pre-commit hooks with lint-staged (Prettier), type checking, and tests in the current repo. Use when user wants to add pre-commit hooks, set up Husky, configure lint-staged, or add commit-time formatting/typechecking/testing. |

## Экспериментальные (beta)

| Навык | Описание автора |
| --- | --- |
| [`chief-of-staff`](../chief-of-staff/SKILL.md) | Pursue a long-running goal in a single session by co-ordinating subagents. |
| [`claude-handoff`](../claude-handoff/SKILL.md) | Hand the current conversation off to a fresh background agent that picks up the work immediately. |
| [`loop-me`](../loop-me/SKILL.md) | Grill me about specs for the workflows I want to build, within this workspace. |
| [`setup-ts-deep-modules`](../setup-ts-deep-modules/SKILL.md) | Wire dependency-cruiser into a TypeScript repo so each package is a deep module, with implementation hidden in subfolders and reachable only through its entry-point files. User-invoked. |
| [`writing-beats`](../writing-beats/SKILL.md) | Writing, exploit; assemble raw material into a journey of beats, grounding each term before a beat leans on it. |
| [`writing-fragments`](../writing-fragments/SKILL.md) | Writing, explore: mine raw fragments, no structure yet. |
| [`writing-shape`](../writing-shape/SKILL.md) | Writing, exploit: shape raw material into an article, paragraph by paragraph. |
