# Totsugeki web application

Web version of the desktop prototype application.

## Usage

Build tailwindcss (read totsugeki) following `totsugeki-native-app`
instructions.

### Prerequisites

```bash
cargo install dioxus-cli
```

#### Start a `dev-server` for the project:

```bash
dioxus serve
```

or package this project:

```bash
dioxus build --release
```

## Project Structure

```
.project
- public # save the assets you want include in your project.
- src # put your code
- - utils # save some public function
- - components # save some custom components
```
