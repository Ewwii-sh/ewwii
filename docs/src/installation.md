# Installation

The first step of using Ewwii is installing it. You would need to have the following prerequesties installed on your system to build/install ewwii.

**Prerequesties:**

-   rustc
-   cargo

Rather than with your system package manager,
I **strongly** recommend installing it using [rustup](https://rustup.rs/).

Additionally, eww requires some dynamic libraries to be available on your system.
The exact names of the packages that provide these may differ depending on your distribution.
The following list of package names should work for arch linux:

<details>
<summary><strong>Packages (click here)</strong></summary>

-   gtk3 (libgdk-3, libgtk-3)
-   gtk-layer-shell (only on Wayland)
-   pango (libpango)
-   gdk-pixbuf2 (libgdk_pixbuf-2)
-   libdbusmenu-gtk3
-   cairo (libcairo, libcairo-gobject)
-   glib2 (libgio, libglib-2, libgobject-2)
-   gcc-libs (libgcc)
-   glibc

</details>

> **Note** that you will most likely need the -devel variants of your distro's packages to be able to compile ewwii.

## Building

Once you have the prerequisites ready, you're ready to install and build ewwii.

First clone the repo:

```bash
git clone https://github.com/byson94/ewwii
```

```bash
cd ewwii
```

Then build:

```bash
cargo build --release --no-default-features --features x11
```

**NOTE:**
When you're on Wayland, build with:

```bash
cargo build --release --no-default-features --features=wayland
```

## Running ewwii

Once you've built it you can now run it by entering:

```bash
cd target/release
```

Then make the Eww binary executable:

```bash
chmod +x ./ewwii
```

Then to run it, enter:

```bash
./ewwii daemon
./ewwii open <window_name>
```

## Installing via package managers

If you don't want to go through the _very_ tedious task of cloning and building ewwii, you can install it using Cargo (Rust crate manager).

You can run the following command to install ewwii from cargo:

```bash
cargo install --git https://github.com/byson94/ewwii
```
