//Lots of boilerplate to turn maude into something organized. Oh well.
macro_rules! html { () => {

maud::html! {
    (maud::DOCTYPE) 
    h1 {
        "what"
    }
}

}; }
pub(crate) use html;