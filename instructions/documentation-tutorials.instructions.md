---
applyTo: '**/docs/**/tutorials/*.md'
description: 'Instructions for creating tutorials in the documentation.'
---

# Tutorial Instructions for `docs/users/tutorials/**/*.md`

---

## **Purpose**
Tutorials guide users from start to finish, helping them achieve specific goals with your software. They should be **practical, progressive, and rewarding**, balancing granularity to avoid fatigue while ensuring a sense of accomplishment.

---

## **Structure**

### **Directory and File Naming**
- Tutorials are organized by **topic** (e.g., `Topic1/`, `Topic2/`).
- Each topic is divided into **sections** (e.g., `Section1/`, `Section2/`), which group related steps.
- Each section contains **sequential steps**, stored as separate Markdown files.
- **File naming format**:
  - Use **two-digit numbering** for sections (e.g., `01_section_name/`) and steps (e.g., `01_step_name.md`).
  - Use **lowercase, hyphen-separated names** for clarity.
  - Example:
    ```
    docs/users/tutorials/
    ├── Topic1/
    │   ├── 01-getting-started/
    │   │   ├── 01-installation.md
    │   │   ├── 02-basic-commands.md
    │   │   └── 03-checkpoint.md
    │   ├── 02-advanced-usage/
    │   │   ├── 01-configuration.md
    │   │   ├── 02-integration.md
    │   │   └── 03-checkpoint.md
    │   └── ...
    └── Topic2/
        ├── 01-introduction/
        │   ├── 01-setup.md
        │   ├── 02-first-steps.md
        │   └── 03-checkpoint.md
        └── ...
    ```

---

## **Content Guidelines**

---

### **1. Required Sections**
Every tutorial step **must** start with:

```markdown
---
**Concepts covered:**
- <List the abstract concepts or knowledge introduced in this step.>

**Skills you will practice:**
- <List the actionable skills or tasks the user will perform.>
---
```

- **Concepts covered**: Focus on **what the user will understand**.
- **Skills you will practice**: Focus on **what the user will do**.

---

### **2. Granularity and Grouping**
- **Sections** group related steps (e.g., "Getting Started," "Advanced Usage").
- **Steps** within a section should be **fine-grained but meaningful**—not too short (to avoid endless clicking) or too long (to avoid endless scrolling).
- Aim for **3–5 steps per section** to maintain engagement.

---

### **3. Step-by-Step Format**
- Use **numbered steps** for actions.
- Use **code blocks** for commands, configuration, or code snippets.
- Use **PlantUML diagrams** (stored in `docs/resources/diagrams/`) for visual guidance.
- **Avoid screenshots** (risk of deprecation).

---

### **4. Exercises**
- Place exercises **at the end of each step** to reinforce learning immediately.
- Hide solutions in collapsible tabs:
  ```markdown
  <details>
  <summary>Solution</summary>

  ```python
  # Solution code here
  ```
  </details>
  ```

---

### **5. Checkpoints**
- Add a **checkpoint file** (e.g., `03-checkpoint.md`) at the end of each section.
- Checkpoints should:
  - **Summarize** what the user has learned.
  - **Preview** what’s next.
  - **Encourage** the user to continue.
- Example:
  ```markdown
  # Checkpoint: Getting Started

  **What you’ve accomplished:**
  - Installed the software.
  - Ran your first commands.

  **Next up:**
  - Dive into advanced configuration.

  > Ready? Let’s go to [Advanced Usage](../02-advanced-usage/01-configuration.md)!
  ```

---

### **6. Tone and Language**
- Use an **active voice** and a **warm, encouraging tone**—like a university lecturer:
  - Be professional yet approachable.
  - Use phrases like:
    - *"Great job! You’ve just configured your first feature."*
    - *"Let’s move on to the next step—you’re doing well!"*
  - Avoid jargon; link to the [glossary](../../appendices/glossary.md) for technical terms.

---

### **7. Navigation**
- Link to **previous/next steps** at the top and bottom of each file:
  ```markdown
  > Previous: [Basic Commands](01-basic-commands.md)
  > Next: [Checkpoint](03-checkpoint.md)
  ```

---

### **8. Accessibility**
- Avoid using color alone to convey meaning; accompany color cues with text labels, icons, or patterns.
- Provide descriptive alt text for all diagrams and images, and include short captions where helpful.
- Structure content using semantic headings and lists so screen readers can navigate easily.
- Ensure code blocks and inline code use sufficient contrast; avoid low-contrast text and small font sizes.
- For collapsible solutions (`<details>`), include clear summary text and avoid relying on hover-only interactions.


---

## **Example Tutorial Step**

```markdown
# 01: Installation

---
**Concepts covered:**
- How the software is distributed and installed.
- System requirements and dependencies.

**Skills you will practice:**
- Installing the software using pip.
- Verifying the installation.
---

## Step 1: Install the Software
1. Open a terminal.
2. Run the following command:
   ```bash
   pip install my-software
   ```
3. Verify the installation:
   ```bash
   my-software --version
   ```

**Exercise:**
Try installing the software in a virtual environment.

<details>
<summary>Solution</summary>

```bash
python -m venv myenv
source myenv/bin/activate  # or `myenv\Scripts\activate` on Windows
pip install my-software
```
</details>

> Next: [Basic Commands](02-basic-commands.md)
```

---

## **Example Checkpoint**

```markdown
# Checkpoint: Getting Started

**What you’ve accomplished:**
- Installed the software.
- Ran your first commands.

**Next up:**
- Dive into advanced configuration.

> Ready? Let’s go to [Advanced Usage](../02-advanced-usage/01-configuration.md)!
```