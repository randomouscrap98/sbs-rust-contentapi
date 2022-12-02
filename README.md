# sbs-rust-contentapi
The frontend for SBS that connects to a private contentapi instance

## About
SBS was original written as a basic PHP website, but it evolved over the years. It was also my first website, so there are lot of ancient
things in there that are extremely difficult to maintain, and it has dependencies on stuff that is REALLY hard to satisfy (even with Docker).

There's a history behind converting SBS to something new, but in the end we went with a basic database migration to a generic API, which
this SSR frontend will then consume. We're using SSR because it's just quicker to implement and easier to get right, and I need a 
minimum viable product that's quick and easy, because I assume the SBS community is permanently gone. I'm doing this to preserve the data
in a maintainable format; keep that in mind when viewing the final product.


I hope it works well for your needs!

## Running/Publishing
This is mostly in case I forget; I don't think anyone will be running the sbs frontend themselves!

There is a basic `publish.sh` file which by itself can't immediately publish the frontend, but which can be sourced
in another shell script after setting the appropriate variables. The script will tell you which variables you need 
to set (they're somewhat private information? Stuff I don't want scraped off github I guess, it's annoying). For 
instance, I currently have a `publish_oboy.sh` file which only sets some variables for publishing to oboy and then
sources (`. publish.sh`) the publish script. 

The publish script can also be used to temporarily run the frontend, which I use for debugging. Just pass "run" as
the first argument.

You can also set the release type, whether debug or release. This is unfortunately done in the publish.sh script
right now, but may be changed in the future to be something you set outside.

### IMPORTANT CAVEAT:
Because of glibc and that whole toolchain thing, this frontend SHOULD be built on the machine you're going to 
install it on. The servers tend to have a bit of an older glibc, especially compared to the crazy modern 
toolchains on development pc environments.
