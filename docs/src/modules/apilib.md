# env

`Namespace: global/std/env`

<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> get_current_dir </h2>

```rust,ignore
fn get_current_dir() -> String
```

<div>
<div class="tab">
<button group="get_current_dir" id="link-get_current_dir-Description"  class="tablinks active" 
    onclick="openTab(event, 'get_current_dir', 'Description')">
Description
</button>
<button group="get_current_dir" id="link-get_current_dir-Returns"  class="tablinks" 
    onclick="openTab(event, 'get_current_dir', 'Returns')">
Returns
</button>
<button group="get_current_dir" id="link-get_current_dir-Example"  class="tablinks" 
    onclick="openTab(event, 'get_current_dir', 'Example')">
Example
</button>
</div>

<div group="get_current_dir" id="get_current_dir-Description" class="tabcontent"  style="display: block;" >
Get the current working directory.
</div>
<div group="get_current_dir" id="get_current_dir-Returns" class="tabcontent"  style="display: none;" >

This function returns the current working directory as a `String`. If there is an error
(e.g., if the path cannot be retrieved), it returns a `Result::Err` with the error message.

</div>
<div group="get_current_dir" id="get_current_dir-Example" class="tabcontent"  style="display: none;" >

```
import "std::env" as env;

// Get the current working directory
let current_dir = env::get_current_dir();
print(current_dir); // output: /home/username/project
```

</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> get_env </h2>

```rust,ignore
fn get_env(var: String) -> String
```

<div>
<div class="tab">
<button group="get_env" id="link-get_env-Description"  class="tablinks active" 
    onclick="openTab(event, 'get_env', 'Description')">
Description
</button>
<button group="get_env" id="link-get_env-Arguments"  class="tablinks" 
    onclick="openTab(event, 'get_env', 'Arguments')">
Arguments
</button>
<button group="get_env" id="link-get_env-Returns"  class="tablinks" 
    onclick="openTab(event, 'get_env', 'Returns')">
Returns
</button>
<button group="get_env" id="link-get_env-Example"  class="tablinks" 
    onclick="openTab(event, 'get_env', 'Example')">
Example
</button>
</div>

<div group="get_env" id="get_env-Description" class="tabcontent"  style="display: block;" >
Get the value of an environment variable.
</div>
<div group="get_env" id="get_env-Arguments" class="tabcontent"  style="display: none;" >

-   `var` - The name of the environment variable to retrieve.
</div>
<div group="get_env" id="get_env-Returns" class="tabcontent"  style="display: none;" >

This function returns the value of the environment variable as a `String`.
If the variable is not found or there is an error, it returns a `Result::Err` with the error message.

</div>
<div group="get_env" id="get_env-Example" class="tabcontent"  style="display: none;" >

```
import "std::env" as env;

// Get the value of the "HOME" environment variable
let home_dir = env::get_env("HOME");
print(home_dir); // output: /home/username
```

</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> get_home_dir </h2>

```rust,ignore
fn get_home_dir() -> String
```

<div>
<div class="tab">
<button group="get_home_dir" id="link-get_home_dir-Description"  class="tablinks active" 
    onclick="openTab(event, 'get_home_dir', 'Description')">
Description
</button>
<button group="get_home_dir" id="link-get_home_dir-Returns"  class="tablinks" 
    onclick="openTab(event, 'get_home_dir', 'Returns')">
Returns
</button>
<button group="get_home_dir" id="link-get_home_dir-Example"  class="tablinks" 
    onclick="openTab(event, 'get_home_dir', 'Example')">
Example
</button>
</div>

<div group="get_home_dir" id="get_home_dir-Description" class="tabcontent"  style="display: block;" >
Get the path to the home directory.
</div>
<div group="get_home_dir" id="get_home_dir-Returns" class="tabcontent"  style="display: none;" >

This function returns the value of the "HOME" environment variable as a `String`.
If the variable is not found or there is an error, it returns a `Result::Err` with the error message.

</div>
<div group="get_home_dir" id="get_home_dir-Example" class="tabcontent"  style="display: none;" >

```
import "std::env" as env;

// Get the home directory
let home_dir = env::get_home_dir();
print(home_dir); // output: /home/username
```

</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> get_username </h2>

```rust,ignore
fn get_username() -> String
```

<div>
<div class="tab">
<button group="get_username" id="link-get_username-Description"  class="tablinks active" 
    onclick="openTab(event, 'get_username', 'Description')">
Description
</button>
<button group="get_username" id="link-get_username-Returns"  class="tablinks" 
    onclick="openTab(event, 'get_username', 'Returns')">
Returns
</button>
<button group="get_username" id="link-get_username-Example"  class="tablinks" 
    onclick="openTab(event, 'get_username', 'Example')">
Example
</button>
</div>

<div group="get_username" id="get_username-Description" class="tabcontent"  style="display: block;" >
Get the current username.
</div>
<div group="get_username" id="get_username-Returns" class="tabcontent"  style="display: none;" >

This function returns the value of the "USER" environment variable as a `String`.
If the variable is not found or there is an error, it returns a `Result::Err` with the error message.

</div>
<div group="get_username" id="get_username-Example" class="tabcontent"  style="display: none;" >

```
import "std::env" as env;

// Get the username of the current user
let username = env::get_username();
print(username); // output: username
```

</div>

</div>
</div>
</br>
<div style='box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2); padding: 15px; border-radius: 5px; border: 1px solid var(--theme-hover)'>
    <h2 class="func-name"> <code>fn</code> set_env </h2>

```rust,ignore
fn set_env(var: String, value: String)
```

<div>
<div class="tab">
<button group="set_env" id="link-set_env-Description"  class="tablinks active" 
    onclick="openTab(event, 'set_env', 'Description')">
Description
</button>
<button group="set_env" id="link-set_env-Arguments"  class="tablinks" 
    onclick="openTab(event, 'set_env', 'Arguments')">
Arguments
</button>
<button group="set_env" id="link-set_env-Returns"  class="tablinks" 
    onclick="openTab(event, 'set_env', 'Returns')">
Returns
</button>
<button group="set_env" id="link-set_env-Example"  class="tablinks" 
    onclick="openTab(event, 'set_env', 'Example')">
Example
</button>
</div>

<div group="set_env" id="set_env-Description" class="tabcontent"  style="display: block;" >
Set the value of an environment variable.
</div>
<div group="set_env" id="set_env-Arguments" class="tabcontent"  style="display: none;" >

-   `var` - The name of the environment variable to set.
-   `value` - The value to assign to the environment variable.
</div>
<div group="set_env" id="set_env-Returns" class="tabcontent"  style="display: none;" >

This function does not return a value.

</div>
<div group="set_env" id="set_env-Example" class="tabcontent"  style="display: none;" >

```
import "std::env" as env;

// Set the value of the "MY_VAR" environment variable
env::set_env("MY_VAR", "SomeValue");
```

</div>

</div>
</div>
</br>
