# Scrum Setup for panmesh

This document describes the Scrum development setup for the panmesh repository.

## GitHub Project

**Project**: [panmesh Scrum Board](https://github.com/users/tkoyama010/projects/3)

### Custom Fields

| Field | Type | Values |
|-------|------|--------|
| Status | Single Select | Todo, In Progress, In Review, Done |
| Story Points | Number | Fibonacci scale: 1, 2, 3, 5, 8, 13 |
| Priority | Single Select | High, Medium, Low |
| Issue Type | Single Select | User Story, Bug, Task, Spike, Epic |
| Sprint | Iteration | Configured via GitHub UI (see below) |

### Labels

| Label | Description | Color |
|-------|-------------|-------|
| `story` | User Story | #c2e0c6 |
| `task` | Technical task | #1d76db |
| `spike` | Research/investigation | #fef2c0 |
| `epic` | Large body of work | #3E4B9E |
| `priority:high` | Must be in current sprint | #b60205 |
| `priority:medium` | Should be in next sprint | #d93f0b |
| `priority:low` | Backlog candidate | #0e8a16 |

Plus GitHub defaults: `bug`, `documentation`, `enhancement`, `good first issue`, `help wanted`, `invalid`, `question`, `duplicate`, `wontfix`.

## Issue Templates

- **User Story**: `.github/ISSUE_TEMPLATE/user_story.yml` — Uses the "As a ... I want ... so that ..." format with acceptance criteria, story points, and priority.
- **Bug Report**: `.github/ISSUE_TEMPLATE/bug_report.yml` — Structured bug reporting with reproduction steps and environment info.

## PR Template

`.github/pull_request_template.md` — Includes summary, related issue, type of change, acceptance criteria checklist, testing checklist, and story points.

## Manual Setup Steps (Required)

The following steps cannot be done via the GitHub API and must be completed through the GitHub Projects UI:

### 1. Configure Sprint Iterations

1. Open the [project board](https://github.com/users/tkoyama010/projects/3)
2. Click on the **Sprint** field header
3. Add iteration periods (recommended: 2-week sprints):
   - Sprint 1: Start date = project start, Duration = 14 days
   - Sprint 2: Auto-continues after Sprint 1
   - Sprint 3: Auto-continues after Sprint 2
4. Add a "Backlog" (no date) iteration for unassigned items

### 2. Create Views

The project has one default view. Create the following views:

#### Backlog View
1. Click the view dropdown → **New view**
2. Name: **Backlog**
3. Layout: **Table**
4. Group by: **Priority** (or **Issue Type**)
5. Sort by: **Story Points** (ascending)
6. Filter: `Sprint = Backlog` (or no sprint assigned)

#### Sprint Board View
1. Click the view dropdown → **New view**
2. Name: **Sprint Board**
3. Layout: **Board**
4. Group by: **Status** (Todo, In Progress, In Review, Done)
5. Filter: `Sprint = <current sprint>`

#### Roadmap View
1. Click the view dropdown → **New view**
2. Name: **Roadmap**
3. Layout: **Roadmap**
4. Field for dates: **Sprint** (iterations)

## Scrum Workflow

1. **Backlog Refinement**: Create User Stories using the issue template. Assign labels (`story`, `priority:high`, etc.) and estimate Story Points.
2. **Sprint Planning**: Move selected stories to the current Sprint. Set Status to "Todo".
3. **Sprint Execution**: Move items to "In Progress" when work starts. Create a PR and move to "In Review" when the PR is opened.
4. **Sprint Review**: Demo completed work. Move "In Review" items to "Done" when PRs are merged.
5. **Sprint Retrospective**: Discuss what went well and what to improve.

## Story Point Estimation

Use the Fibonacci scale for relative estimation:

| Points | Complexity |
|--------|-----------|
| 1 | Trivial — a few lines, well-understood |
| 2 | Simple — small change, minimal risk |
| 3 | Moderate — some complexity, clear approach |
| 5 | Complex — significant work, some unknowns |
| 8 | Large — needs careful planning, multiple components |
| 13 | Very Large — consider splitting into smaller stories |
