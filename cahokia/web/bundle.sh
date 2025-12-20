# TODO: Move this logic into Cast
dx bundle --platform web --release
pwd=$(pwd)
sha=$(git rev-parse --verify HEAD)
year=$(date +%Y)
month=$(date +%m)
day=$(date +%d)
counter=1
dirty=$([[ -n $(git status -s) ]] && echo '-dirty')
filename="0.1.0+$year-$month-$day.$counter.$sha$dirty.zip"
# TODO: Pull version from Cargo.toml
# TODO: Increment build counter
cd ../target/dx/web/release/web/public
zip -r $filename . -x "*.DS_Store"
mv $filename $pwd/artifacts/$filename
cd $pwd