#!/bin/bash

set -e

# do cleanup on exit. if debugging, comment out line 9
cleanup() {
  rm -r ./tmp/
}
trap cleanup EXIT

# TODO: take in file glob as command line argument
PROFILES=tests/small/add.bril

RUNMODES=("nothing" "rvsdg-optimize")

# create temporary directory structure necessary for bench runs
mkdir -p ./tmp/bench

# bench will benchmark a single bril file, outputting its contents to ./tmp/bench/<profile_name>.json
bench() {
    profile="../../../$1"

    # strip the file path down to just the file name
    # TODO: profile name is not unique, generate a unique output path (it will be aggregated anyway)
    profile_file=$(basename -- $profile)
    profile_name="${profile_file%.*}"

    mkdir ./tmp/bench/"$profile_name"
    pushd ./tmp/bench/"$profile_name"

		# loop over RUNMODES and generate a profile for each, leaving it in the profile_name directory
    for mode in ${RUNMODES[@]}
		do
		  echo "profiling $mode"

			out="${mode}.json"
    	pwd
			# generate the instruction count profile by interp'ing
			cargo run --release $profile --interp --profile-out="./$out" --run-mode "$mode"
			
	    # $out now contains a key value of total_dyn_inst: value, so use read to get the key/value
      # TODO: this is kind of a yaml sort of format so maybe yq would be good in the future
			IFS=': ' read KEY VAL <<< $(cat $out)

			# generate the profile and overrite the $out file
			hyperfine --warmup 2 --export-json "$out" "cargo run --release $profile --interp --run-mode $mode"

			# overwwrite outfile with json version of profile data, annotate with profile name.
    	# we also combine both instruction count and hyperfine json output into a single object
    	# to make things super easy
			printf '{"%s": "%s", "hyperfine": %s}' \
      	$KEY $VAL "$(cat "$out")" > $out
    done

    popd
}

for p in $PROFILES
do
  bench $p
done

# aggregate all profile data into a single JSON array
python3 infra/aggregate.py > nightly/data/profile.json
