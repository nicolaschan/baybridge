default: run

watch:
  npx tailwindcss -i css/input.css -o dist/output.css --watch

build-css:
  npm install
  npx tailwindcss -i css/input.css -o dist/output.css

build: build-css
  cargo build --release

run: build-css
  cargo run -- serve
