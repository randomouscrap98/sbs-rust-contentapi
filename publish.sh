# set -e

# Stuff you may want to change
NAME="sbs-rust-contentapi"
PUBLISHDIR="publish"
EXTRAS="LICENSE README.md static settings.toml $INSTALLEXTRAS"

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

# Everything published is release now, since we can use musl
BUILDTARGET="x86_64-unknown-linux-musl"
BUILDTYPE="release"
BUILDPARAM="--release --features \"perf\" --target=${BUILDTARGET}"

# # This is ridiculous. cargo is... ugh (see the dual variables)
# if [ "$INSTALLCOMPILE" = "release" ]; then
#    BUILDTYPE="release"
#    BUILDPARAM="--release --features \"perf\""
# else
#    BUILDTYPE="debug"
#    BUILDPARAM=""
# fi

# Calculated stuff
INSTALLDIR="${INSTALLBASE}/${NAME}"
LOGIN="${INSTALLUSER}@${INSTALLHOST}"
FULLENDPOINT="${LOGIN}:${INSTALLDIR}"


# Before we do anything, we need to install the musl target. It may 
# already be installed
rustup target add ${BUILDTARGET}

# Now, we build for the target.
cargo build ${BUILDPARAM}

# Next, we clear out the publish folder (always fresh) and recreate it
rm -rf "$PUBLISHDIR"
mkdir -p "$PUBLISHDIR"
cp -r $EXTRAS "$PUBLISHDIR"
cp "target/${BUILDTARGET}/${BUILDTYPE}/${NAME}" "$PUBLISHDIR"

# RSYNC options:
# h : human readable (always need)
# v : verbose (also always)
# z : compress (why not?)
# a : archive (recursive AND preserve as much metadata as possible, generally what you want)

# Copy everything except 'target', which is a LOT of data...
# echo "Copying release to ${FULLENDPOINT}"
# rsync -avhz -e "ssh -p ${INSTALLPORT}" ./ --exclude 'target' --exclude '.git' \
#    --exclude 'contentapi_copy' ${FULLENDPOINT}
#rsync -avhz -e "ssh -p ${INSTALLPORT}" ./ --exclude 'target' --exclude '.git' ${FULLENDPOINT}

# We have to build ON the server itself because glibc (I don't want to use docker)
# echo "Building ${NAME} on remote server ${INSTALLHOST}"
# BUILDCMD="cargo build ${BUILDPARAM}"
# if [ "$INSTALLCLEAN" = "true" ]; then
#    BUILDCMD="cargo clean;${BUILDCMD}"
# fi
# echo "* Build command: ${BUILDCMD}"
# SSHCMD=". /home/${INSTALLUSER}/.cargo/env; cd ${INSTALLDIR}; ${BUILDCMD}"

# if [ "$1" = "run" ]
# then
#    PRODUCT="./target/${BUILDTYPE}/${NAME}"
#    # If choosing a profile, set it first before calling the product
#    if [ -n "$INSTALLPROFILE" ]; then
#       PRODUCT="${PRODUCT} ${INSTALLPROFILE}"
#    fi
#    echo "ALSO Running ${PRODUCT}"
#    SSHCMD="${SSHCMD} && echo \"Running ${NAME}...\" && ${PRODUCT}"
# fi

# ssh -t -p ${INSTALLPORT} ${LOGIN} "${SSHCMD}"

# If we said to run, let's go ahead and do that remotely now just for fun
echo "All done!"
