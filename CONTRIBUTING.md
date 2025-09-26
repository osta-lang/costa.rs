# Contributing to Osta

Thank you for your interest in contributing to Osta! We appreciate your efforts to help build a powerful systems
programming language. Whether you're fixing bugs, improving documentation, or proposing new features, all contributions
are welcome.

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are
expected to uphold this code. Reports of unacceptable behavior can be directed to the project maintainers.

## Getting Started

To contribute, follow these steps:

1. **Fork the repository**: Create your own fork of the [osta-lang/costa.rs](https://github.com/osta-lang/costa.rs)
   repository.
2. **Clone your fork**:
    ```bash
    git clone https://github.com/your-username/costa.rs.git
    cd costa.rs
    ```
3. **Create a branch**: Use a descriptive name for your feature (`feat/branch-name`) or bugfix branch (
   `fix/branch-name`).
    ```bash
    git checkout -b feat/your-feature
    ```
4. **Install dependencies**: Ensure you have Rust installed (we target the latest stable version). Run:
    ```bash
    cargo build
    ```
5. **Make your changes**: Implement your contribution, ensuring it aligns with the project's goals and style.
6. **Test your changes**: Run the test suite:
    ```bash
    cargo test
    ```
7. **Commit your changes**: Use clear, concise commit messages following conventional commit style (e.g.
   `feat(typing): add procedural types caching`).
    ```bash
    git add .
    git commit -m "feat(module): brief description of your feature"
    ```
8. **Push to your fork**:
    ```bash
    git push origin feat/your-feature
    ```
9. **Open a pull request**: Create a PR against the `dev` branch of the main `osta-lang/costa.rs` repository. Include a
   description of your changes, why they are needed, and any relevant issue numbers.
10. **Participate in the review**: Respond to feedback and make any requested changes.
11. **Celebrate**: Once your PR is merged, celebrate your contribution to Osta!

## Reporting Bugs

- Before reporting a bug, please search the [issue tracker](https://github.com/osta-lang/costa.rs/issues) to see if it
  has already been reported.
    - If it has, add any additional information you have to the existing issue.
    - If it hasn't, create a new issue with:
        - A clear, descriptive title.
        - Steps to reproduce the bug.
        - Expected vs. actual behavior.
        - Environment details (OS, Rust version, etc.).
        - Any relevant logs or screenshots.
        - If possible, a minimal code snippet that reproduces the issue.

## Suggesting Features

- Check the [project roadmap](https://github.com/osta-lang/costa.rs/milestones)
  and [discussions](https://github.com/osta-lang/costa.rs/discussions) for ongoing proposals.
- Open a new discussion or issue labeled `enhancement` with:
    - A clear description of the feature.
    - Motivation and use cases.
    - Proposed syntax or implementation ideas, referencing the [Osta documentation](https://docs.osta-lang.dev) if
      applicable.
- For syntax changes, provide rationale tied to the language's goals (e.g., type system power, C compatibility) and
  examples.

## Development Guidelines

- **Code Style**: Follow the Rust's idiomatic style. Run `cargo fmt` before committing.
- **Testing**: Write tests for new features and bug fixes. Aim for high test coverage. Ensure all tests pass before
  submitting a PR.
- **Documentation**: Update or add documentation for new features or changes. Use clear, concise language.
- **Commit Messages**: Use the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) format for commit
  messages.
- **Early Stage**: Note that Osta is in early development; expect frequent changes. Contributions should be flexible to
  adapt to evolving design decisions.

## Pull Request Guidelines

- Check that your PR targets the `dev` branch.
- Ensure your PR passes all CI checks (build, tests, linting).
- Provide a detailed description of your changes and their purpose.
- Link to any related issues or discussions.
- Keep PRs focused; avoid mixing unrelated changes.
- Expect feedback and iteration—maintainers may request changes or clarifications.
- Once reviewed and approved, your changes will be merged.

## Questions?

If you have questions about contributing, open a discussion or reach out via issues.  
We look forward to your contributions! 🚀
  