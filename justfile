default: run

watch:
  tailwindcss -i css/input.css -o dist/output.css --watch

build-css:
  tailwindcss -i css/input.css -o dist/output.css

build: build-css
  cargo build --release

run: build-css
  cargo run -- serve
