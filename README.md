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

## Running locally
You will need the rust toolchain to build and run this frontend. You should be familiar with the rust ecosystem.
The frontend is a standard rust app, so basic `cargo` commands function as expected. The repo has no 
dependencies outside of what is required by rust and whatever cargo brings in automatically (such as the packages required to build the frontend).

This frontend connects to a [contentapi](https://github.com/randomouscrap98/contentapi) backend. Upon request, I can provide a `contentapi.tar.gz` 
archive which contains a prebuilt, pre-configured backend that has everything 
necessary to run SBS locally. Simply extract, run `run_local.sh`, and as long as it's running on the 
same machine as the frontend, you can run the frontend. All default settings are fine for the frontend, so a simple `cargo run` 
will suffice. You do not need dotnet to run this special backend. Please read the initial output of `run_local.sh` for more information

So to summarize:
* Download `contentapi.tar.gz` (link provided upon request)
* Extract to wherever
* Run `run_local.sh` from the extracted files (keep it running)
* On the same machine, go to the sbs frontend and do `cargo run`
* Visit `http://localhost:5011` to get to the sbs frontend
* You can continue to iterate on the sbs frontend while the backend is running in the background

## Publishing
This is mostly in case I forget; I don't think anyone will be publishing the sbs frontend for themselves!

There is a basic `publish.sh` file which by itself can't immediately publish the frontend, but which can be sourced
in another shell script after setting the appropriate variables. The script will tell you which variables you need 
to set (they're somewhat private information? Stuff I don't want scraped off github I guess, it's annoying). For 
instance, I currently have a `publish_oboy.sh` file which only sets some variables for publishing to oboy and then
sources (`. publish.sh`) the publish script. 

The publish script can also be used to temporarily run the frontend on the remote machine you published to, which I use for debugging. Just pass "run" as
the first argument.

You can also set the release type, whether debug or release. This is unfortunately done in the publish.sh script
right now, but may be changed in the future to be something you set outside.

### IMPORTANT CAVEAT:
Because of glibc and that whole toolchain thing, this frontend SHOULD be built on the machine you're going to 
install it on. The servers tend to have a bit of an older glibc, especially compared to the crazy modern 
toolchains on development pc environments.
