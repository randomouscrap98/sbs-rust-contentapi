
pub fn render(data: MainLayoutData, page: Content, comments: Vec<Message>, users: HashMap<i64, User>, ) -> String 
{
    //Need to split category search into parts 
    //let search_system = match &search.system { Some(system) => system, None => };
    layout(&data, html!{
        section { }
    }).into_string()
}