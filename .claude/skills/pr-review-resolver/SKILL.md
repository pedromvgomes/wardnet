---
name: pr-review-resolver
description: |
  Analyze and resolve PR review comments — from GitHub Copilot, CodeRabbit, human reviewers, or any other source.
  Use this skill whenever the user wants to address PR review feedback, fix review comments, handle Copilot suggestions,
  respond to code review findings, or resolve PR conversations. Trigger on phrases like "address PR comments",
  "fix review feedback", "handle copilot review", "resolve PR findings", or when a PR URL is provided with
  intent to address its review comments. Also use when the user says "fix PR comments", "address review",
  or references open review threads they want to resolve.
---

# PR Review Resolver

Analyze PR review comments, present a structured assessment to the user, implement approved fixes, and close
the feedback loop by responding to reviewers.

## Workflow

### Phase 1: Gather Review Comments

1. **Identify the PR** — the user may provide a PR URL, a PR number, or you may need to detect it from
   the current branch. Use `gh pr view` or the GitHub MCP tools to find the PR.

2. **Fetch all review comments** — collect every unresolved review comment on the PR. For each comment, capture:
   - The reviewer (human or bot — e.g., `copilot`, `coderabbitai`, a team member's handle)
   - The file and line range the comment targets
   - The full comment text, including any suggested code changes
   - The conversation thread (replies, if any) for context

3. **Read the relevant code** — for each comment, read the file and surrounding context so you can
   form an informed opinion. Don't just rely on the diff — understand the broader context of the code.

### Phase 2: Analyze and Present Findings

For each review comment, produce a structured analysis:

```
### Finding [N]: [Short title summarizing the concern]
**Reviewer:** @handle
**File:** path/to/file.kt:L42-L50
**Comment:** [The reviewer's original comment]

**Analysis:**
[Your explanation of what the reviewer is flagging and why it matters or doesn't]

**Assessment:** Valid concern | Partially valid | Not applicable | Already addressed
[Justify your assessment — why you agree or disagree with the reviewer]

**Suggested fix:** (if valid)
[Concrete approach to resolve the concern, with enough detail for the user to evaluate]
```

Present **all findings at once** in a numbered list. The user can then respond with which ones to fix
(e.g., "fix all", "fix 1, 3, 5", "fix all except 2").

**Important considerations when analyzing:**
- Be honest in your assessment — don't rubber-stamp every comment as valid. Some automated reviewers
  produce false positives or flag things that are intentional design choices.
- When a comment is not valid, explain clearly why — the user needs to understand your reasoning
  to make a good decision, and you'll need to articulate this in the reply to the reviewer.
- When a comment is valid, think about the best fix — not just the quickest. Consider whether the
  reviewer's suggested change is the right approach or if there's a better alternative.
- Group related comments that touch the same concern — sometimes multiple comments are really about
  one underlying issue.

### Phase 3: Plan the Fixes

Once the user confirms which findings to address:

1. **Enter plan mode** — before writing any code, create a detailed implementation plan covering all
   approved fixes. The plan should include:
   - Each finding being addressed, with its number and title
   - The specific files and locations that will be modified
   - The concrete changes to be made (what code will be added, removed, or modified)
   - How fixes that interact with each other (e.g., two comments about the same function) will be
     coordinated
   - Any potential risks or side effects of the changes

2. **Present the plan for approval** — show the complete plan to the user and wait for explicit approval
   before proceeding to implementation. The user may request adjustments to the plan — iterate until
   they're satisfied.

   This checkpoint exists because the cost of implementing the wrong fix is much higher than the cost
   of reviewing a plan. The user knows the codebase context that you might not — let them catch issues
   before code is written.

3. **Do not proceed to implementation until the user approves the plan.** If the user requests changes
   to the plan, update it and present the revised version for approval.

### Phase 4: Implement Approved Fixes

Once the plan is approved:

1. **Implement the fixes** — make the code changes exactly as outlined in the approved plan. Follow the
   project's existing conventions and patterns. Read surrounding code to match style.

2. **Verify the changes** — run relevant tests or builds to make sure the fixes don't break anything.
   Use the project's standard test commands.

### Phase 5: Commit, Push, and Respond

1. **Commit** — create a single commit with all fixes. The commit message should:
   - Summarize what was fixed at a high level in the subject line
   - In the body, include a detailed report:
     - Which review findings were addressed (reference them by number)
     - What was changed for each
     - How each change addresses the reviewer's concern

   Example:
   ```
   fix: address PR review feedback

   Resolved the following review findings:

   - Finding 1 (null safety): Added null check in UserService.findById()
     before accessing the response body. This prevents the potential NPE
     flagged by @copilot when the upstream service returns 204.

   - Finding 3 (error handling): Replaced generic catch block with specific
     exception types in OrderProcessor. This ensures transient failures
     are retried while validation errors fail fast, as suggested by @reviewer.
   ```

2. **Push** — push the changes to the PR branch.

3. **Reply to reviewers** — for each addressed comment, post a reply explaining:
   - What was changed
   - How the change addresses the concern
   - Ask if the reviewer has any further feedback

   For comments that were intentionally **not** fixed (user decided they're not applicable),
   reply with a polite explanation of why the team chose not to address it.

4. **Request re-review** — request a new review from the reviewers who left comments,
   so they can verify the fixes. Use `gh pr edit` or the GitHub API to re-request reviews.

## Edge Cases

- **No unresolved comments** — if the PR has no open review comments, tell the user and stop.
- **Comments on deleted lines** — if a comment references code that no longer exists (e.g., from
  a previous revision), note this in the analysis and suggest marking it as resolved.
- **Conflicting reviewer opinions** — if two reviewers disagree, present both perspectives and
  let the user decide.
- **Large PRs** — if there are many comments (>15), consider grouping them by file or theme
  to make the analysis easier to digest.

## Tools

This skill relies on GitHub CLI (`gh`) and/or the GitHub MCP tools for:
- Fetching PR details and review comments
- Posting reply comments
- Requesting re-reviews
- Pushing commits
