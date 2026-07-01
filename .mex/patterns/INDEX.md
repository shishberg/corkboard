# Pattern Index

Lookup table for all pattern files in this directory. Check here before starting any task — if a pattern exists, follow it.

<!-- This file is populated during setup (Pass 2) and updated whenever patterns are added.
     Each row maps a pattern file (or section) to its trigger — when should the agent load it?

     Format — simple (one task per file):
     | `[<file>.md](<file>.md)` | One-line description of when to use this pattern |

     Format — anchored (multi-section file, one row per task):
     | `[<file>.md#task-first](<file>.md#task-first)` | When doing the first task |
     | `[<file>.md#task-second](<file>.md#task-second)` | When doing the second task |

     Example (from a Flask API project):
     | `add-api-client.md` | Adding a new external service integration |
     | `debug-pipeline.md` | Diagnosing failures in the request pipeline |
     | `crud-operations.md#task-add-endpoint` | Adding a new API route with validation |
     | `crud-operations.md#task-add-model` | Adding a new database model |

     Keep this table sorted alphabetically. One row per task (not per file).
     If you create a new pattern, add it here. If you delete one, remove it. -->

| Pattern | Use when |
|---------|----------|
| [add-page-component.md](add-page-component.md) | Adding a new noticeboard page type or editing widget in the Vue web UI |
| [deploy-to-orange-pi.md](deploy-to-orange-pi.md) | Flashing Armbian and getting the device server running headless on the Orange Pi Zero 2W |
| [device-api.md](device-api.md) | Loading or publishing page state between the web UI and the device (GET/POST, image upload) |
