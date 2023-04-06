# set -e

# Stuff you may want to change
NAME="sbs-rust-contentapi"
PUBLISHDIR="publish"
EXTRAS="LICENSE README.md static settings.toml $INSTALLEXTRAS"
# NOTE: INSTALLEXTRAS is something expected to be set by a parent script

# Everything published is release now, since we can use musl
BUILDTARGET="x86_64-unknown-linux-musl"
BUILDTYPE="release"
BUILDPARAM="--release --features perf --target=${BUILDTARGET}"

# Check required variables
if [ -z "$INSTALLUSER" ]; then
   echo "MUST SET INSTALLUSER"
   exit 1
elif [ -z "$INSTALLHOST" ]; then
   echo "MUST SET INSTALLHOST"
   exit 1
elif [ -z "$INSTALLPORT" ]; then
   echo "MUST SET INSTALLPORT (default ssh = 22)"
   exit 1
elif [ -z "$INSTALLBASE" ]; then
   echo "MUST SET INSTALLBASE (don't include project name)"
   exit 1
fi

# Calculated stuff
INSTALLDIR="${INSTALLBASE}/${NAME}"
LOGIN="${INSTALLUSER}@${INSTALLHOST}"
FULLENDPOINT="${LOGIN}:${INSTALLDIR}"


# Before we do anything, we need to install the musl target. It may 
# already be installed
echo "Installing rust musl target for linux"
rustup target add ${BUILDTARGET}

# Now, we build for the target.
echo "Building $BUILDTARGET"
cargo build ${BUILDPARAM}

# Next, we clear out the publish folder (always fresh) and recreate it
echo "Prepping $PUBLISHDIR"
rm -rf "$PUBLISHDIR"
mkdir -p "$PUBLISHDIR"
cp -r $EXTRAS "$PUBLISHDIR"
cp "target/${BUILDTARGET}/${BUILDTYPE}/${NAME}" "$PUBLISHDIR"

# RSYNC options:
# h : human readable (always need)
# v : verbose (also always)
# z : compress (why not?)
# a : archive (recursive AND preserve as much metadata as possible, generally what you want)

echo "Copying to ${FULLENDPOINT}"
rsync -avhz -e "ssh -p ${INSTALLPORT}" "${PUBLISHDIR}/" ${FULLENDPOINT}

echo "All done!"
