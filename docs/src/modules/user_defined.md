# User-Defined Modules

User-defined modules allow you to **organize your code** by splitting it into separate, reusable files. This makes large projects easier to maintain and understand.

## Exporting Items from a Module

In Rhai modules:

-   **Functions are automatically exported** by default. You do **not** need to use `export` for functions.
-   **Variables, constants, and objects must be exported manually** using the `export` keyword.

```rust,ignore
// File: ./foo/baz.rhai

/// A function that is automatically exported
fn greet() {
    return "Greetings!"
}

/// A private function, NOT exported automatically
private fn foo() {
    return "This function is hidden and not exported"
}

/// A variable
let PI = 3.14159;

// Export the variable explicitly
export PI;
```

**Tip:** Only variables, constants, and objects require the `export` keyword. Functions are always available unless marked `private`. [More info](https://rhai.rs/book/language/modules/export.html#export-functions)

## Importing a Module

You can import a module using the `import` keyword:

```rust,ignore
// File: ./ewwii.rhai

import "foo/baz" // just runs the script without importing it.
import "foo/baz" as example; // runs the script and imports it into example.

// Access exported items
print(example::greet()); // Greetings!
print(example::PI); // 3.14159
```

**Tip:** Always use the `as` keyword to import a script as a module with the name you desire.

## Notes

-   Functions are automatically exported unless explicitly marked `private`.
-   Variables, constants, and objects must be exported using the `export` keyword.
-   `as` keyword is important in an `import` statement if you want to import the variables and functions in a Rhai file.
