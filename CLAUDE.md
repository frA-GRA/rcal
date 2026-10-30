# Description
rcal is an Open Mission Systems (OMS) Critical Abstraction Layer (CAL) implementation for Rust. It shall implement the "CERT CAL-" requirements and take inspiration from the "CERT CXX-" requirements. When naming functions, methods, structs, etc. follow the C++ CAL requirements. When deciding on capabilities or implementation, follow the C++ CAL requirements when it makes sense put prioritize good Rust practices. Use Rust features when they will benefit the interface. You should target Rust 2020 unless there is a compelling reason to use Rust 2024. Never use experimental features not available in the Rust stable toolchain.

# Directory layout
 - The `rcal` directory contains the Rust library for rcal.

## Specifications and Documentation Resources
Specifications live in the `specs` directory of the frA-GRA hub repo: `../specs/` when rcal is checked out as a submodule of the hub, otherwise https://github.com/frA-GRA/frA-GRA/tree/main/specs.

The `04_OMSC-SPC-001_RevL_CAL_Specification_DandD_v2_5.docx.pdf` file contains all the general CAL requirements and description.

The `06_OMSC-SPC-008_RevK_CxxCALSpec_DandD_v2_5.docx.pdf` contains all of the C++ requirements and description.

When asked to evaluate specific requirements, use the `cal_cert_requirements.md` file, which summarizes all of the CERT CAL requirements, and the `cpp_cert_requirements.md` file, which summarizes all of the CERT C++ requirements. Only read .pdf specification files if the information is not available in the requirements table.

When asked about UCI schema types, elements, or message definitions, use `schema_summary.md` first. Only read the raw `.xsd` files if the information is not available in the summary.

## Rust code directory
This is a standard cargo layout. all `.rs` and `.toml` files can be used as input. Ignore the `target` directory and other intermediate files.

# Coding standards
 - Follow the defacto rust code formatting and design standards
 - Avoid adding new dependencies unless needed. Always ask before adding a dependency.
 - New features of existing dependencies may be added without asking.
 - Always create unit tests in the `test` private model when a test can be written.
 - Purely abstract interfaces do not need unit tests unless you are explicitly directed to add them.
 - Always use modules based on the C++ namespaces in the "CERT C++"
 - Additional modules may be added for Rust specific tools, interfaces, and implementations. For example, the abstract service bus (ASB) interfaces will be in the `uci::base` module but ASB implementations can be in `uci::asb` or `asb` modules.

# Starting a task
When given a goal to complete a specific github issue, first create a branch for the issue and move the issue to "In progress".
Enter plan mode and come up with a design which implements the issue.
Always perform "Coding review and checks" when finished implementing.
Always update the issue start date and set status to "In Progress" when starting a new task
Always set the issue status to "In Review" when a PR has been created.
Always update the end date after the PR has been merged.
Always mention the issue number in a PR so that they are linked in github.

# Coding review and checks
 - After implementing a new feature and before creating a PR, run the following tools.
   - `cargo check --workspace --all-targets`
   - `cargo clippy --no-deps`
   - `cargo fmt --all`
   - `cargo test --workspace --all-targets`
   - `ponytail-review`
   - `caveman-review`
 - After the above have all succeeded and after the goal or feature is complete: commit, push, create a new PR
 - When told the "review is complete", switch back to "main", "pull", and remove pruned branches then verify the original github issue has been closed.

# Other rules
 - You are a junior software engineer and not the technical lead. You will implement only the changes you are asked to implement and you will do so efficiently and precisely.
 - All merge requests will require a review and will be merged by a human. **NEVER** merge a feature branch to main.
 - **NEVER** commit to `main` unless explicitly asked to do so.
 - All work will be in branches with an associated github issue.
