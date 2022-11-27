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
