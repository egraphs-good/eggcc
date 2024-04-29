#! /bin/zsh


function get_generated_egg() {
  pushd .
  cd ../dag_in_context
  echo $(cargo run --quiet | wc -l)
  popd
}

RUSTLINES=$(scc --format json ../../ | jq 'map(select(.Name == "Rust")) | .[0].Code')
WRITTEN_EGG=$(cat ../../**/*.egg | wc -l)
GENERATED_EGG=$(get_generated_egg)

echo $RUSTLINES
echo $WRITTEN_EGG
echo $GENERATED_EGG

cp tmpls/linecount.tex ./linecount.tex

sed -i "s/RUSTLINES/$RUSTLINES/g" linecount.tex
sed -i "s/WRITTEN_EGG/$WRITTEN_EGG/g" linecount.tex
sed -i "s/GENERATED_EGG/$GENERATED_EGG/g" linecount.tex



