# Optional: StaticScript

Before diving deeper into Ewwii and **Rhai** (Ewwii’s configuration language), check out **[StaticScript](https://ewwii-sh.github.io/staticscript/)**.

Rhai layouts are dynamic and can be verbose, which may feel overwhelming for users without experience in languages like Rust, C, or JavaScript. **StaticScript** simplifies Rhai by abstracting much of the complexity, adding stricter compilation, and providing clearer error messages. Many issues can be caught **at compile time**, making it more beginner-friendly.

**How it works:**
StaticScript compiles a custom language called `sts` into Rhai quickly and efficiently. For users creating **static-only widgets**, it can be a powerful way to simplify development.

> **Note:**
> StaticScript is **experimental**. It currently does **not** support most dynamic Rhai features, such as:
>
> -   Variables and updates
> -   Polling and listeners
> -   Functions and conditionals (`if/else`)
> -   Loops (`for`/`while`)
> -   Imports/exports
>
> While excellent for static widgets, it cannot yet match the full flexibility of raw Rhai.
