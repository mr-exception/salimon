# Notion project connection

Verified on 2026-09-08 using the connected Notion integration.

- Workspace: **Alireza Darbandi's Notion**
- Workspace ID: `66bfde99-3710-480d-880b-8fba0121a42f`
- Project: [Salimon](https://app.notion.com/p/801c9c427af24e9b8d57b07572ef4119)

These are durable references for agents, not an API configuration or background
sync service. Each session needs access through the authenticated Notion
integration. Building and running the client does not require Notion access.

## Entry points

| Source | Notion link / identifier |
| --- | --- |
| Documents | [Documents](https://app.notion.com/p/3d5b456853b981f18877f8cc4283a3cd) |
| Business specifications | [Business Specs](https://app.notion.com/p/3d5b456853b981e48833c6ebeb237892) |
| Task database | [Tasks](https://app.notion.com/p/e2dadc570fff48b4899def8b0ee70a27) |
| Task data source | `collection://7cf6230d-e528-4b78-8aeb-c8acfc0dbe71` |
| Ordered task view | `view://7c8708d3-012f-4ffa-9565-0928d151cfea` |
| Phase 0 scope | [Technical Feasibility Showcase](https://app.notion.com/p/3d5b456853b981db968dca1901a270a2) |
| Architecture | [Technical Architecture & AI Maintenance](https://app.notion.com/p/3d5b456853b981078a82c68207f4444e) |
| Product decisions | [Project Q&A](https://app.notion.com/p/3d5b456853b981888824e2cdfe1a5297) |
| Task 1 | [Create Phase 0 repository shell](https://app.notion.com/p/3d5b456853b981e485dfcc5d8dc86c9f) |
| Task 2 | [Bootstrap native macOS runtime and wgpu renderer](https://app.notion.com/p/3d5b456853b981e0b34ec086b885ddb7) |
| Task 3 | [Add Phase 0 diagnostics overlay](https://app.notion.com/p/3d5b456853b98136afd1dd612e9af67c) |
| Task 4 | [Implement large-scale coordinate and camera prototype](https://app.notion.com/p/3d5b456853b981258981dec8275426a0) |
| Task 5 | [Build compressed Solar System scene](https://app.notion.com/p/3d5b456853b981f98685d3b91e81340c) |
| Task 6 | [Implement scalable planet rendering](https://app.notion.com/p/3d5b456853b981eca132cff958c0a890) |
| Task 7 | [Create custom Phase 0 spaceship asset](https://app.notion.com/p/3d5b456853b981f18cc1d56d50570646) |

## Task lookup and updates

1. Fetch `self` to check the connected workspace and available tools. Prefer
   fetching the known links directly. When searching, use `ai_search` if available;
   otherwise use `search`, scoped to the Salimon page or task data source.
2. Fetch the task database schema before querying or updating properties. The
   verified schema has `Title`, `Order` (number), `Status` (select), `Description`,
   and `Acceptance Criteria`. Status options are `To Do`, `In Progress`, and `Done`.
   Identify a numbered task by `Order`, not title matching or search result order.
3. Read the full task and its linked specifications before implementation. Check
   for truncated or unsupported content before treating a fetch as complete.
4. Keep task status and implementation evidence on the existing task page. Record
   relevant commands/results, remaining limitations, and the commit identifier
   after committing. Preserve the task's requirements and unrelated page content.

The Salimon hub contains legacy notes about an entity/buffer project. Its current
game documentation and Business Specs establish the custom Rust, native macOS,
`wgpu` direction. Adjacent local projects are not requirements for this repository.
