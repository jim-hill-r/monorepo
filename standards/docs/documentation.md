- All projects MUST include a README.md. The README.md should include the following sections:
    - {Name of the project} section which includes a short description (executive summary) of the goal of the project, including why it was named that.
- All projects MUST include a CONTRIBUTING.md. The CONTRIBUTING.md should include the following sections:
    - A `Getting Started` section which describes how to install the project's toolchain, how to build, and how to test.

## Pull Request Standards

### UI Changes Documentation

All pull requests with UI changes MUST include visual documentation.

**What constitutes a UI change:** Any change that affects what users see or how they interact with the application, including:
- Changes to visual appearance (colors, layout, styling, fonts)
- Changes to user interface components (buttons, forms, menus, navigation)
- Changes to displayed content or text (including error messages, tooltips, help text)
- Changes to animations or transitions
- Changes to responsive behavior or mobile layouts
- Changes to accessibility attributes that affect user experience (ARIA labels, roles)

**Screenshot Requirements:**

- **Screenshots**: Always include screenshots of UI changes in the PR description
  - Take full-page screenshots showing the before and after states when possible
  - Capture screenshots that clearly demonstrate the visual changes
  - Use the playwright browser tools to take screenshots when the dev server is running

**Exceptions:** Backend-only changes, configuration changes, or changes to non-visual code do not require screenshots.