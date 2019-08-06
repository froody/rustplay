in get_filenames:

cargo build --release
./target/release/browser <DIRNAME> paths-out.bin path-components.txt

in root:
cargo build --release
./target/release/hello_world path-components.txt paths-out.bin > report.txt
