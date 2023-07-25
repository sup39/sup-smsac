set -e

version=$(grep -Po '(?<=version = ").*(?=")' Cargo.toml | head -n1)
outDir=out/sup-smsac-$version
#rm -rf "$outDir"
mkdir -p "$outDir"

cargo build --release
cargo-about generate about.hbs -o www/LICENSE.html
cp ./target/release/sup-smsac.exe "$outDir/"
cp -r www res README.md LICENSE.txt CHANGELOG.md "$outDir/"
