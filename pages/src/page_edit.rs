use common::constants::SBSPageType;
use common::submissions::*;
use contentapi::*;

use common::*;
use common::forms::*;
use common::render::*;
use common::render::forum::*;
use common::render::layout::*;
use contentapi::endpoints::ApiContext;
use maud::*;

//Rendering ALWAYS requires the form, even if it's just an empty one
pub fn render(data: MainLayoutData, form: PageForm, all_categories: Vec<Category>, errors: Option<Vec<String>>) -> String 
{
    let title : Option<String>;
    let mut submit_value = format!("Submit {}", form.subtype);

    //Assume it's new or not based on the values in the form. The form drives this render
    if form.id == 0 {
        title = Some(format!("Create new {}", form.subtype));
    }
    else {
        title = Some(format!("Edit page: '{}'", form.title));
        submit_value = format!("Update {}", form.subtype);
    }
/* 
    pub id: i64, //Should default to 0
    pub subtype: String, //Has to be SOMETHING, and the post endpoint will reject invalid values
    pub title: String,
    pub text: String,
    pub description: String,    //Making this required now
    pub keywords: String,       //List of keywords separated by space, gets split afterwards
    pub images: String,         //Same as keywords
    pub categories: String,     //Same as keywords

    //These are optional fields, for programs
    pub key: Option<String>,
    pub version: Option<String>,
    pub size: Option<String>,
    pub systems: Option<String>     //Same as keywords
    */

    layout(&data, html!{
        (data.links.style("/forpage/pageeditor.css"))
        section {
            @if form.subtype != SBSPageType::PROGRAM && form.subtype != SBSPageType::RESOURCE {
                h1."error" { "Unknown editor type: " (form.subtype) }
            }
            @else if let Some(title) = title {
                h1 { (title) }
                //NOTE: NO ACTION! These kinds of pages always post to themselves
                form."editor" #"pageedit_form" method="POST" {
                    (errorlist(errors))
                    input #"pageedit_id" type="hidden" name="id" value=(form.id);
                    input #"pageedit_subtype" type="hidden" name="subtype" value=(form.subtype);
                    label for="pageedit_title" { "Title:" }
                    input #"pageedit_title" type="text" name="title" value=(form.title) required placeholder="Careful: first title is used for permanent link text!";
                    label for="pageedit_tagline" { "Tagline:" }
                    input #"pageedit_tagline" type="text" name="description" value=(form.description) required placeholder="Short and sweet!";
                    label for="pageedit_text" { "Main Page:" }
                    (post_textbox(Some("pageedit_text"), Some("text"), Some(&form.text)))
                    @if form.subtype == SBSPageType::PROGRAM {
                        label for="pageedit_key" { "Key:" }
                        input #"pageedit_key" type="text" name="key" value=(opt_s!(form.key)) required placeholder="The key for people to download your program!";
                        label for="pageedit_version" { "Version:" }
                        input #"pageedit_version" type="text" name="version" value=(opt_s!(form.version)) placeholder="A version to keep track of updates (not required)";
                        label for="pageedit_size" { "Size in bytes:" }
                        input #"pageedit_size" type="text" name="size" value=(opt_s!(form.size)) placeholder="Rough estimate for total size of download (not required)";
                    }
                    label for="pageedit_images" { "Images:" }
                    input #"pageedit_images" type="text" name="images" value=(form.images) placeholder="Space separated";
                    details."editorinstructions" {
                        summary { "About images" }
                        p { "Images are uploaded to your account, not to the page. So, you first upload your images using the form "
                            "below, then you can copy the unique image id and paste it into the field above. The first image listed "
                            "becomes the main image for your page"
                        }
                        iframe."imagebrowser" src={(data.links.imagebrowser())} {}
                    }
                    label for="pageedit_categories" { "Categories:" }
                    input #"pageedit_categories" type="text" name="categories" value=(form.categories) placeholder="Space separated";
                    details."editorinstructions" {
                        summary { "About categories" }
                        p { "You can categorize your page for organization and searching. The category table is below: for each category " 
                            "you want, add the ID to the field above"
                        }
                        table {
                            tr { th { "Name" } th { "Id" } }
                            @for category in all_categories {
                                tr { td{ (category.name) } td{ (category.id) }}
                            }
                        }
                    }
                    label for="pageedit_keywords" { "Keywords:" }
                    input #"pageedit_keywords" type="text" name="keywords" value=(form.keywords) placeholder="Space separated";
                    input type="submit" value=(submit_value);
                }
            }
            @else {
                h1."error" { "PAGE EDITOR CANNOT LOAD" }
            }
        }
    }).into_string()
}

//You can optimize this later I guess (if it really needs it...)
const THISCONTENTFIELDS : &str = "*";
const THISMESSAGEFIELDS : &str = "*";

pub async fn get_render(mut context: PageContext, subtype: Option<String>, page_hash: Option<String>) -> 
    Result<Response, Error> 
{
    //let mut page: Option<Content> = None;
    let mut form = PageForm::default();

    if let Some(subtype) = subtype {
        form.subtype = subtype;
    }

    //if let Some(hash) = page_hash {
    //    let c = context.api_context.get_content_by_hash(&hash, THISCONTENTFIELDS).await?;
    //    form.content_id = c.id.unwrap(); 
    //    thread = Some(c);
    //}
    let all_categories = map_categories(get_all_categories(&mut context.api_context, None).await?);
    let cloned_subtype = form.subtype.clone();
    //println!("Subtype: {}, All categories: {:#?}", cloned_subtype, all_categories);

    Ok(Response::Render(render(context.layout_data, form, all_categories.into_iter().filter(move |c| &c.forcontent == &cloned_subtype).collect(), None)))
}

/* 
/// Craft the message to be written to the api for the given post form
pub async fn construct_post_message(context: &ApiContext, form: &PostForm) 
    -> Result<Message, Error>
{
    let mut message;
    
    if form.id > 0 {
        //Use all the values from the original message. You can't "move" messages fyi...
        message = context.get_message_by_id(form.id, THISMESSAGEFIELDS).await?;
    }
    else {
        message = Message::default();
        message.contentId = Some(form.content_id);
        let mut values = make_values! {
            "markup": "bbcode"
        };
        //If we have a reply, add the appropriate reply-replated values
        if let Some(reply_id) = form.reply_id {
            let reply_to = context.get_message_by_id(reply_id, THISMESSAGEFIELDS).await?;
            let reply_data = get_new_replydata(&reply_to);
            reply_data.write_to_values(&mut values);
        }
        message.values = Some(values);
    }
    message.text = Some(form.post.clone()); 

    Ok(message)
} */

pub async fn post_render(context: PageContext, form: PageForm) ->
    Result<Response, Error>
{
        Err(Error::Other(String::from("Not logged in!")))
    /*if let Some(ref _user) = context.layout_data.user 
    {
        //This one, we throw all the way, since we can't re-render the page without the parent anyway
        let thread = context.api_context.get_content_by_id(form.content_id, THISCONTENTFIELDS).await?;
        let mut written_post : Option<Message> = None;
        let mut errors = Vec::new();

        match construct_post_message(&context.api_context, &form).await {
            Ok(message) =>
            {
                match context.api_context.post_message(&message).await { 
                    Ok(posted_post) => {
                        written_post = Some(posted_post);
                    },
                    Err(e) => { errors.push(e.to_user_string()); }
                }
            },
            Err(e) => { errors.push(e.to_user_string()); }
        }

        if errors.is_empty() {
            //If there are no errors, we go to the new page
            Ok(Response::Redirect(
                if let Some(ref post) = written_post {
                    context.layout_data.links.forum_post(post, &thread)
                }
                else {
                    Err(Error::Other(String::from("Some internal error occurred, preventing the new thread from being shown! No errors produced, but no thread data found!")))?
                })
            )
        }
        else {
            //Otherwise, we stay here and show all the terrifying errors
            Ok(Response::Render(render(context.layout_data, form, Some(thread), Some(errors))))
        }
    }
    else {
        Err(Error::Other(String::from("Not logged in!")))
    }*/
}

pub async fn delete_render(context: PageContext, page_id: i64) ->
    Result<Response, Error>
{
    //This is a VERY DUMB delete endpoint, because it just passes it through to the backend. Then the backend will
    //produce errors and the resulting screen will just be one of the ugly 400 screens (which we might spruce up).
    //I don't care much about it because the delete thread isn't a form, its just a button, so putting the usual
    //error box doesn't really work.
    //let result = 
    context.api_context.post_delete_content(page_id).await?;

    Ok(Response::Redirect(context.layout_data.links.activity()))
}
