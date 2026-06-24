---
name: architecture
description: How the major pieces of this project connect and flow. Load when working on system design, integrations, or understanding how components interact.
triggers:
  - "architecture"
  - "system design"
  - "how does X connect to Y"
  - "integration"
  - "flow"
edges:
  - target: context/stack.md
    condition: when specific technology details are needed
  - target: context/decisions.md
    condition: when understanding why the architecture is structured this way
  - target: patterns/device-api.md
    condition: when implementing or debugging the load/publish flow between web UI and device
last_updated: 2026-06-23
---

# Architecture

## System Overview
Two pieces talk over HTTP using a JSON page-state contract:

Web UI (Vite + Vue + shadcn-vue) — on load, GETs the current page state (JSON) from the
device → user edits pages (sidebar of page thumbnails, an editing toolbar, and a timeline
showing the order pages loop through) → user clicks **Publish** → the new page state is
POSTed to the device server → the device saves it and refreshes the display. Image uploads
are sent as separate POST requests.

Where the web UI is hosted (served from the device, or hosted elsewhere) is [TO BE DETERMINED].

Once the page-state contract and endpoints are defined, this protocol likely earns its own
`context/protocol.md` — for now it is small enough to live here.

## Key Components
- **Web UI** — Vue app for editing pages. Sidebar shows page thumbnails, a toolbar holds editing tools, a timeline shows the order pages loop through. Talks to the device over HTTP.
- **Device server** — runs on the noticeboard device. Serves current page state on GET, accepts published state and image uploads on POST, and refreshes the display. [TO BE DETERMINED — language (Python assumed) and framework]
- **Page state** — the JSON document describing all pages and their loop order. The contract between web UI and device. [TO BE DETERMINED — schema/shape]

## External Dependencies
- **The noticeboard device** — target hardware is an Orange Pi Zero 2W. Hosts the device server and the physical display.
- **Persistence on the device** — how page state is stored. [TO BE DETERMINED — database/format not decided]
- **Image storage on the device** — uploaded via separate POST requests. [TO BE DETERMINED — location/format]

## What Does NOT Exist Here
- No user accounts and no authentication — assumes a trusted private network where anyone can GET/POST.
- No cloud service — the device is the source of truth; there is no central backend in between (unless the web UI is later hosted separately, still [TO BE DETERMINED]).
- No display rendering in the web UI — the device owns drawing the noticeboard; the web UI only edits and publishes state.
