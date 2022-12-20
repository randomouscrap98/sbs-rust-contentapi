use common::*;
use common::layout::*;
use maud::html;

pub fn render(data: MainLayoutData) -> String {
    layout(&data, html!{
        section {
            h1 { "About SmileBASIC Source"}
            p { 
                "We're a simple fan site dedicated to the various programming applications created by "
                a href="https://smileboom.com/en/" { "SmileBoom"} " which let you program in a dialect of BASIC "
                "on your Nintendo handhelds! Originally released on the DSi as "
                a href="https://en.wikipedia.org/wiki/Petit_Computer" { "Petit Computer" } ", it got a sequel "
                "on the Nintendo 3DS before finally coming to the Nintendo Switch! The language is a modified "
                "version of BASIC called " b { "SmileBASIC"} ", and it lets you do a lot of fun stuff! "
            }
            p { 
                "We were most active during the 3DS era, so most of what you'll see here is for SmileBASIC"
                "on the Nintendo 3DS. " 
            }
            h3 { "Privacy" }
            p { 
                r#"We DO NOT track you in any meaningful way. We do not sell any bit of your information. We do not collect
                usage data. We do not show ads, so there's no tracking cookies."# 
            }
            p { 
                r#"We save one cookie which represents your login session. No personal information is stored in your cookie 
                other than your userId. You are free to delete it at any point (you'll just be logged out). I don't have the 
                resources to implement GDPR stuff, and I believe it's mostly for ad tracking cookies anyway? Since we don't
                collect your data to sell, or track you for monetary purposes, I think I just have to tell you what data we 
                store about you and how you can request or delete it."# 
            }
            p { 
                r#"This is a social website. As such, there is obvious data we store for public retrieval, such as pages,
                comments, forum threads, and forum posts. We also store direct messages, but those are only accessible by the 
                parties involved, NO other user. We store a username you pick and your email, along with 
                a salted password hash from which you cannot reproduce the original password (standard practice). Your email 
                is not visible to anyone but you, and we only use it for password recovery and account verification. We store 
                various settings you may set for the website. We also store images you upload, which you can use as your avatar or as 
                screenshots or images for your programs and tutorials. The data is all stored on a private server which is kept 
                up to date with the latest security patches. We follow industry standard security practices for sensitive data, and
                all connections are forced to use HTTPS. There are no insecure hops between you and the server, barring a man-in-the-middle
                attack."# 
            }
            p { r#"You can request a dump of all your user data, but it might take a bit, I'm just one guy. The format will be a basic
                SQL dump for all the text data, and a zip archive of all images you've uploaded. You can also request a deletion of
                all your personal information, although it will make me very sad, as we'll lose valuable SBS history. Deletion is 
                permanent and cannot be undone. All images will be wiped from the server, and your user data will be purged. Pages
                and threads which you created will be turned into stubs and the original data purged, but the stub will contain
                OTHER people's comments and posts, as that data is theirs and not yours. But again, the timeframe for deletion may
                be a while, as I'm just one guy."#
            }
            h3 { "Technical info" }
            p { "All code for SmileBASIC source is open source and available on github:" }
            table {
                tr { td { b { "Backend:" }} td { a href="https://github.com/randomouscrap98/contentapi" { "Contentapi" } " (by me)" } }
                tr { td { b { "Frontend:" }} td { a href="https://github.com/randomouscrap98/sbs-rust-contentapi" { "Custom rust frontend" } " (by me)" } }
            }
            p { "SmileBASIC source uses the following technologies:" }
            table {
                tr { td { a href="https://www.sqlite.org/index.html"{"SqLite:"}} td {"Database engine" } }
                tr { td { a href="https://dotnet.microsoft.com/en-us/"{"Dotnet:"}} td {"Basis for Contentapi" } }
                tr { td { a href="https://dotnet.microsoft.com/en-us/apps/aspnet"{"ASP.NET:"}} td {"Web API for Contentapi"} }
                tr { td { a href="https://imagemagick.org/index.php"{"Image Magick:"}} td {"Image manipulation (thumbnails etc)"} }
                tr { td { a href="https://github.com/DapperLib/Dapper"{"Dapper:"}} td {"Basic ORM for Contentapi"} }
                tr { td { a href="https://www.rust-lang.org"{"Rust:"}} td {"Basis for this frontend"} }
                //Don't forget to change this to wrap
                tr { td { a href="https://github.com/SergioBenitez/Rocket"{"Rocket:"}} td {"Server-side web framework for this frontend"} }
                tr { td { a href="https://aws.amazon.com"{"Amazon AWS:"}} td { "Hosting (EC2) and image/backup storage (S3)"} }
                tr { td { a href="https://domains.google"{"Google:"}} td { "Our DNS and email provider"} }
            }
            h3 { "Contact info" }
            p { r#"This website is run by just me! If you need help with the website or have any website-related issues, 
                you can contact me over email at smilebasicsource@gmail.com."# }
        }
    }).into_string()
}