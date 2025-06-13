# TODO: Move this logic into Cast
dx bundle --platform web --release
sha=$(git rev-parse --verify HEAD)
year=$(date +%Y)
month=$(date +%m)
day=$(date +%d)
counter=1
# TODO: Pull version from Cargo.toml
# TODO: Increment build counter
zip -vr "./artifacts/0.1.0+$year.$month.$day.$counter.$sha.zip" ../target/dx/web/release/web/public/ -x "*.DS_Store"