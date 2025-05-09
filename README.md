[![GitHub Release](https://img.shields.io/github/v/release/alexwine36/serpeo?filter=serpeo*&style=flat)](https://github.com/alexwine36/serpeo/releases) [![Build](https://github.com/alexwine36/serpeo/actions/workflows/build.yml/badge.svg?event=release)](https://github.com/alexwine36/serpeo/actions/workflows/build.yml)



# Serpeo

An application to run SEO checks on a website efficiently




## Run Locally

### Requirements
- Rustup
- Pnpm

Clone the project

```bash
  git clone git@github.com:alexwine36/serpeo.git
```

Go to the project directory

```bash
  cd serpeo
```

Install dependencies

```bash
  pnpm install
```

Start the server

```bash
  pnpm tauri:dev
```





## Running Tests

To run tests, run the following command

```bash
  pnpm test
```

To run just rust tests

```bash
pnpm rust-test
# or
cargo test
```

To run just js tests

```bash
pnpm js-test
```


## Tech Stack

**Client:** React, Vite, TailwindCSS, ShadCN

**Server:** Rust, Tauri


## Contributing

Contributions are always welcome!

See `contributing.md` for ways to get started.

Please adhere to this project's `code of conduct`.

### Generate new plugin
#### Page Plugin
```bash
pnpm turbo gen plugin
```

#### Site Plugin
```bash
pnpm turbo gen site-plugin
```

## Issues

If you have any issues, please add them [here](https://github.com/alexwine36/serpeo/issues?q=sort%3Aupdated-desc+is%3Aissue+is%3Aopen)

