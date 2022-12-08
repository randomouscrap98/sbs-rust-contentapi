use maud::{html, DOCTYPE};
use warp::{Filter, Reply, Rejection};

//use crate::templates;
//
//macro_rules! make_filter {
//    ($name:ident $block:block ) => {
//        pub fn $name() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone 
//            $block
//    };
//}
//
//make_filter!{index {
//    //yeah
//    warp::get().and(warp::path!().map(|| {
//        warp::reply::html(html! {
//            (DOCTYPE)
//        })
//    }))
//}}

//Warp REALLY confuses me...
pub fn get_all_routes() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    //This COULD be configurable but whatever. this is the filesystem static serving filter
    let static_route = warp::path("static").and(warp::fs::dir("static"));

    //warp::body::content_length_limit(1024 * 32)

    let index_route = warp::get().and(warp::path!().map(|| {
        warp::reply::html(html! {
            (DOCTYPE)
            body {
                h1 { "wow this is index" }
            }
        }.into_string())
    }));
    //.with(warp::cors().allow_any_origin());

    static_route.or(index_route)
    //    //
    //    warp::get().and(
    //        //get routes
    //    )
    //    .or(warp::post().and(
    //        //post routes?
    //    )

    //    )
    //)
}
