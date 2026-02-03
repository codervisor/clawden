# ClawLab 0-1 Bootstrap Prompt

**Project:** ClawLab

**Objective:** An AI-powered "Browser/Computer Use" engine that automates web interactions to generate high-quality software product demos (videos/images).

**Core Tech Stack:**

* **Language:** TypeScript / Node.js
* **Automation:** Playwright (Headless/Headed)
* **Vision AI:** Mainstream LLMs (Claude, GPT, Gemini, etc.)
* **Video Engine:** Remotion (for rendering smooth UI motion)

**The Task:**
Act as a Lead Software Architect. Scaffolding the initial MVP for ClawLab with the following directory structure and core modules:

1. **`src/agent`**: A Vision-Agent wrapper that takes a "Goal" (e.g., "Show how to create a team"), takes screenshots via Playwright, and determines the next `click`, `type`, or `scroll` action.
2. **`src/recorder`**: A module to capture high-resolution frames and metadata (element coordinates, event logs) during the AI session.
3. **`src/renderer`**: A basic Remotion composition that takes the captured metadata and frames to produce a polished video with smooth cursor interpolation.
4. **`src/cli`**: A simple command-line interface to trigger the agent.

**Key Requirements for the Code:**

* **Action Smoothing:** Implement a logic to record "Bezier curve" mouse movements instead of teleporting to coordinates.
* **State Management:** The agent must verify if an action was successful (e.g., checking for a specific selector) before proceeding.
* **Error Handling:** Basic retry logic if the AI "hallucinates" a button that isn't there.

**Deliverable:**
Please provide the `package.json` dependencies and the core logic for the `ClawAgent.ts` and `BrowserManager.ts` files to get the project running from 0 to 1.
