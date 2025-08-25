# Modules

Modules undoubtedly are one of the most powerful features in Rhai. They provide infinite extensibility to ewwii.

Every module follows the syntax:

```js
import "std::env" as env;
let home = env::get_home_dir(); // returns `$HOME` env var value
```

This allows you to write expressive, modular Rhai code with functions grouped logically under `std` or custom namespaces.
