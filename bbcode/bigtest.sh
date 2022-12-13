# This is how you'd run the performance tests! They have output and stuff!
cargo test --release --features "bigtest perf" -- --nocapture --test-threads=1

# you can remove 'perf' to see how it compares. I find it to be about 4 times
# faster, so it seems worth it for production

# Some notes about performance tests:
# -----------------------------------
#
# So the 10000 test will just work, and it'll time how long it takes to do
# 10000 smallish posts. The performance here isn't really indicative of
# anything important, just there for fun
#
# The IMPORTANT test actually reads files from the filesystem very VERY BIG
# bbcode parsing and equality check.
#
# Create a directory 'bigtests'. The system will run the bbcode parser against
# each file in this directory, and compare against the SAME named file in
# 'bigtests/parsed'. So, you might have this:
#
# /bigtests
# |-test1.txt
# |-test2.txt
# |-/parsed
#   |-test1.txt
#   |-test2.txt
#
# The parsed files need to be exactly how the bbcode should look after parsing,
# each and every character (including whitespace and newlines).

