use super::*;

//This will render the entire index! It's a handler WITH the template in it! Maybe that's kinda weird? who knows...
pub fn index(data: MainLayoutData) -> Result<impl warp::reply::Reply, warp::reply::Response>{
    Ok(layout(data, html!{
        //This is the body of index
        section {
            h1 { "Hello, this is the index!"}
        }
    }).into_string())
}