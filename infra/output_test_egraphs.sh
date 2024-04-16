# TODO: take in file glob as command line argument
PROFILES=(tests/passing/**/*.bril)

# exit script if serialized directory already exists
if [ -d "./serialized" ]; then
  echo "serialized already exists, exiting"
  exit 1
fi

# make serialized directory
mkdir -p ./serialized

# bench will benchmark a single bril file, outputting hyperfine contents to ./tmp/bench/<PROFILE_NAME>.json
# and will output the number of instructions it executed to ./tmp/bench/<PROFILE_NAME>.profile
bench() {
    echo "json for $1"
    # store just the file name to a variable
    PROFILE_FILE=$(basename -- "$1")
    cargo run --release "$1" --run-mode serialize > ./serialized/"$PROFILE_FILE".json
}

for p in "${PROFILES[@]}"
do
  bench "$p"
done
