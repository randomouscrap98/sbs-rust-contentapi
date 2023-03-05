use std::collections::HashMap;

use common::constants::DOCSGROUPUSERNAME;
use common::constants::MARKUPBBCODE;
use common::constants::PTCSYSTEM;
use common::constants::SBSMARKUPS;
use common::constants::SBSPageType;
use common::constants::SBSSYSTEMS;
use common::constants::SBSValue;
use common::constants::ANYSYSTEM;
use common::view::*;
use common::prefab::*;
use contentapi::*;

use common::*;
use common::forms::*;
use common::render::*;
use common::render::forum::*;
use common::render::layout::*;
use contentapi::endpoints::ApiContext;
use maud::*;


//Rendering ALWAYS requires the form, even if it's just an empty one
pub fn render(data: MainLayoutData, form: PageForm, mode: Option<String>, all_categories: Vec<Category>, 
    all_docpaths: Vec<String>, errors: Option<Vec<String>>) -> String 
{
    let title : String;
    let mut submit_value = format!("Submit {}", form.subtype);
    let raw_categories : Vec<(String, String)> = all_categories.iter().map(|c| (c.id.to_string(), c.name.clone())).collect();
    let raw_systems : Vec<&(&str, &str)> = SBSSYSTEMS.iter().filter(|(id,_)| *id != ANYSYSTEM && *id != PTCSYSTEM).collect(); 

    //Assume it's new or not based on the values in the form. The form drives this render
    if form.id == 0 {
        title = format!("Create new {}", form.subtype);
    }
    else {
        title = format!("Edit page: '{}'", form.title);
        submit_value = format!("Update {}", form.subtype);
    }

    let real_mode = if let Some(m) = mode { m } else { get_mode_from_form(&form) };

    layout(&data, html!{
        (data.links.style("/forpage/pageeditor.css"))
        (data.links.script("/forpage/pageeditor.js"))
        section {
            @if real_mode != SBSPageType::PROGRAM && real_mode != SBSPageType::RESOURCE && real_mode != PTCSYSTEM && 
                real_mode != SBSPageType::DOCUMENTATION { 
                h1."error" { "Unknown editor type: " (form.subtype) }
            }
            @else {
                h1 { (title) }
                //NOTE: NO ACTION! These kinds of pages always post to themselves
                form."editor" #"pageedit_form" data-mode=(real_mode) data-noupgrade method="POST" {
                    (errorlist(errors))
                    input #"pageedit_id" type="hidden" name="id" value=(form.id);
                    input #"pageedit_subtype" type="hidden" name="subtype" value=(form.subtype);
                    label for="pageedit_title" { "Title:" }
                    input #"pageedit_title" type="text" name="title" value=(form.title) required placeholder="Careful: first title is used for permanent link text!";
                    label for="pageedit_tagline" { "Tagline:" }
                    input #"pageedit_tagline" type="text" name="description" value=(form.description) required[real_mode != SBSPageType::DOCUMENTATION] placeholder="Short and sweet!";
                    label for="pageedit_text" { "Main Page:" }
                    (post_textbox(Some("pageedit_text"), Some("text"), Some(&form.text)))
                    @if real_mode == SBSPageType::DOCUMENTATION {
                        //Show the markup selector for documentation (may change in the future)
                        label for="pageedit_markup"  { "Markup:" }
                        select #"pageedit_markup" name ="markup" required {
                            @for (key, value) in SBSMARKUPS {
                                option value=(key) selected[Some(*key) == form.markup.as_deref()] { (value) }
                            }
                        }
                        label for="pageedit_docpath" { "Documentation Path:" }
                        input #"pageedit_docpath" type="text" name="docpath" value=(opt_s!(form.docpath)) required 
                            list="all_docpaths" placeholder="Select existing or enter new";
                        datalist #"all_docpaths" {
                            @for ref docpath in all_docpaths {
                                option value=(docpath);
                            }
                        }
                        label for="pageedit_hash" { "Documentation URL Hash:" }
                        input #"pageedit_hash" type="text" name="hash" value=(opt_s!(form.hash)) required placeholder="Example: docs-sb4-while";
                    }
                    @if real_mode == SBSPageType::PROGRAM || real_mode == PTCSYSTEM { 
                        @if real_mode == PTCSYSTEM {
                            noscript { h2."error" { "The PTC editor requires javascript, I'm very sorry!" }}
                            input #"pageedit_systems" type="hidden" name="systems" value=(PTCSYSTEM) required;
                            label for="pageedit_newfile" { "Add .PTC file:" }
                            input #"pageedit_newfile" type="file" accept=".ptc";
                            p."aside" {
                                "While in Petit Computer, go to the file manager and export the file or files you want onto your sd card. "
                                "The files are exported to folders named after the file, and inside is your .ptc file. Every time you export "
                                "a file with the same name, it creates a new file in that folder, so you generally want the last one. When you "
                                "upload the file here, we'll parse the name from it and let you add a description. When people visit your page, "
                                "they'll be able to get the QR codes for each file you added."
                            }
                            label { "Manage PTC files:" }
                            div #"ptc_file_list" { }
                            details."editorinstructions" {
                                summary."aside" { "Inspect raw PTC form data (readonly, auto-generated)" }
                                textarea #"pageedit_ptc_files" name="ptc_files" readonly { (opt_s!(form.ptc_files)) }
                                button type="button" #"ptc_files_refresh" { "Refresh" }
                            }
                        }
                        @else {
                            label for="pageedit_key" { "Key:" }
                            input #"pageedit_key" type="text" name="key" value=(opt_s!(form.key)) required placeholder="The key for people to download your program!";
                            label for="pageedit_systems" { "Systems:" }
                            input #"pageedit_systems" type="text" name="systems" value=(opt_s!(form.systems)) required placeholder="What console does this go on?";
                            details."editorinstructions" #"systems_instructions"{
                                summary."aside" { "About systems" }
                                p { "SmileBASIC is available for several systems, so people have to know what system your program is for! "
                                    "Certain systems are interoperable and share keys, so you can add multiple systems if multiple apply. "
                                    "Please use the IDs below for the system, not the name."
                                }
                                table #"systems_table" data-raw=(serde_json::ser::to_string(&raw_systems).unwrap_or_default()) {
                                    tr { th { "Name" } th { "Id" } }
                                    @for (id, name) in raw_systems {
                                        tr { td{ (name) } td{ (id) }}
                                    }
                                }
                                p."aside" #"ptc_editor_aside" { 
                                    "Looking for Petit Computer (DSi)? That requires a different editor: "
                                    a href=(data.links.page_editor_new(PTCSYSTEM)) { "PTC Page Editor" }
                                    " (you will lose any data entered here!)"
                                }
                            }
                        }
                        label for="pageedit_version" { "Version:" }
                        input #"pageedit_version" type="text" name="version" value=(opt_s!(form.version)) placeholder="A version to keep track of updates (not required)";
                        label for="pageedit_size" { "Size (include units):" }
                        input #"pageedit_size" type="text" name="size" value=(opt_s!(form.size)) placeholder="Rough estimate for total size of download (not required)";
                    }
                    @if real_mode != SBSPageType::DOCUMENTATION {
                        label for="pageedit_images" { "Images:" }
                        input #"pageedit_images" type="text" name="images" value=(opt_s!(form.images)) placeholder="Space separated";
                        details."editorinstructions" {
                            summary."aside" { "About images" }
                            p { "Images are uploaded to your account, not to the page. So, you first upload your images using the form "
                                "below, then you can copy the unique image id and paste it into the field above. The first image listed "
                                "becomes the main image for your page"
                            }
                            iframe."imagebrowser" src={(data.links.imagebrowser())} {}
                        }
                        label for="pageedit_categories" { "Categories:" }
                        input #"pageedit_categories" type="text" name="categories" value=(opt_s!(form.categories)) placeholder="Space separated";
                        details."editorinstructions" #"categories_instructions" {
                            summary."aside" { "About categories" }
                            p { "You can categorize your page for organization and searching. The category table is below: for each category " 
                                "you want, add the ID to the field above"
                            }
                            table #"categories_table" data-raw=(serde_json::ser::to_string(&raw_categories).unwrap_or_default()) {
                                tr { th { "Name" } th { "Id" } }
                                @for category in all_categories {
                                    tr { td{ (category.name) } td{ (category.id) }}
                                }
                            }
                        }
                    }
                    label for="pageedit_keywords" { "Keywords:" }
                    input #"pageedit_keywords" type="text" name="keywords" value=(form.keywords) placeholder="Space separated";
                    input type="submit" value=(submit_value);
                }
            }
        }
    }).into_string()
}

pub async fn get_render_categories(api_context: &mut ApiContext, subtype: &str) -> Result<Vec<Category>, Error> {
    let all_categories = map_categories(get_all_categories(api_context, None).await?);
    let cloned_subtype = subtype.clone();
    Ok(all_categories.into_iter().filter(move |c| &c.forcontent == &cloned_subtype).collect())
}

pub async fn get_render_docpaths(api_context: &mut ApiContext) -> Result<Vec<String>, Error> {
    let all_documentation = get_all_documentation(api_context).await?;
    let docpath_map = get_all_docpaths(&all_documentation);
    let mut docpaths : Vec<String> = docpath_map.keys().map(|k| k.clone()).collect();
    docpaths.sort_by(|a, b| a.len().cmp(&b.len()));
    Ok(docpaths)
}

pub fn get_mode_from_form(form: &PageForm) -> String {
    if let Some(ref system) = form.systems {
        if system.contains(PTCSYSTEM) {
            return "ptc".to_string()
        }
    }
    return form.subtype.clone()
}

pub fn get_subtype_from_mode(mode: &str) -> String {
    if mode == "ptc" { SBSPageType::PROGRAM.to_string() }
    else { mode.to_string() }
}

pub async fn get_render(mut context: PageContext, mode: Option<String>, page_hash: Option<String>) -> 
    Result<Response, Error> 
{
    let mut form = PageForm::default();

    if let Some(hash) = page_hash 
    {
        let fullpage = get_fullpage_by_hash(&mut context.api_context, &hash).await?; 
        let page = fullpage.main; 
        let page_type = page.literalType.as_deref();

        //Remember to do all the ref stuff before we move values out of page
        if page_type == Some(SBSPageType::PROGRAM) || page_type == Some(SBSPageType::RESOURCE) {
            form.categories = Some(get_tagged_categories(&page).into_iter().map(|x| x.to_string()).collect::<Vec<String>>().join(" "));
        }
        else {
            form.categories = None;
        }
        //Don't need to check the page type for these, we just pass them through based on if they are Some or not
        form.key = page.get_value_string(SBSValue::DOWNLOADKEY);
        form.size = page.get_value_string(SBSValue::SIZE); 
        form.version = page.get_value_string(SBSValue::VERSION); 
        form.markup = page.get_value_string(SBSValue::MARKUP);
        form.docpath = page.get_value_string(SBSValue::DOCPATH);
        form.hash = page.hash.clone();
        if let Some(images) = page.get_value_array(SBSValue::IMAGES) {
            form.images = Some(images.into_iter().map(|i| i.as_str().unwrap_or("")).collect::<Vec<&str>>().join(" "));
        }
        if let Some(systems) = page.get_value_array(SBSValue::SYSTEMS) {
            form.systems = Some(systems.into_iter().map(|i| i.as_str().unwrap_or("")).collect::<Vec<&str>>().join(" "));
        }
        if let Some(ptcpage) = fullpage.ptc {
            form.ptc_files = ptcpage.text;
        }
        form.id = page.id.unwrap();
        //WARN : APPARENTLY DESCRIPTION CAN BE "NONE" FROM THE DATABASE!!
        form.description = page.description.unwrap_or_default();
        form.keywords = page.keywords.unwrap().join(" ");
        form.subtype = page.literalType.unwrap();
        form.text = page.text.unwrap();
        form.title = page.name.unwrap();
    }
    else if let Some(ref mode) = mode {
        form.subtype = get_subtype_from_mode(mode);
    }
    else {
        return Err(Error::Other(String::from("Invalid operating mode: must have hash or mode!")));
    }

    let render_categories = get_render_categories(&mut context.api_context, &form.subtype).await?;
    let render_docpaths = get_render_docpaths(&mut context.api_context).await?;
    Ok(Response::Render(render(context.layout_data, form, mode, render_categories, render_docpaths, None)))
}

/// Craft the MAIN content to be written to the api for the given post form
pub async fn construct_post_content_full(context: &mut ApiContext, form: &PageForm) 
    -> Result<FullPage, Error>
{
    let mut fullpage; 
    
    if form.id > 0 {
        //Go pull all the original values. Note that most pages are legacy pages with important 
        //information, make sure that information is NOT overwritten or lost! Also, note that
        //pages cannot change their form, so 'literalType' is not set on edit
        fullpage = get_fullpage_by_id(context, form.id).await?;
    }
    else {
        //Setup some defaults. These need to be defaults for ALL editor types!! Anything that's not
        //may go somewhere else (but may not!). If the form supplies it, it does NOT GO HERE!!!
        fullpage = FullPage::default(); 
        fullpage.main.contentType = Some(ContentType::PAGE);
        fullpage.main.literalType = Some(form.subtype.clone());
        fullpage.main.values = Some(HashMap::new());
        fullpage.main.permissions = Some(make_permissions! {
            "0": "CR" 
        });

        //We HAVE to get the parent of content!
        let mut request = FullRequest::new();
        add_value!(request, "systemtype", ContentType::SYSTEM);

        //The actual parent type changes based on the form's subtype (we know it must be set appropriately
        //if we're posting)
        if form.subtype == SBSPageType::DOCUMENTATION {
            add_value!(request, "parent_type", SBSPageType::DOCPARENT);
        }
        else {
            add_value!(request, "parent_type", SBSPageType::SUBMISSIONS);
        }

        request.requests.push(build_request!(
            RequestType::content, 
            String::from("id,literalType,contentType"), 
            String::from("literalType = @parent_type and contentType = @systemtype")
        )); 

        let result = context.post_request(&request).await?;
        let mut submission_parents = conversion::cast_result_required::<Content>(&result, &RequestType::content.to_string())?;
        let submission_parent = submission_parents.pop().ok_or_else(|| Error::NotFound(String::from("Couldn't find submissions parent!")))?;

        fullpage.main.parentId = submission_parent.id;
    }

    //Note that at this point, we MAY OR MAY NOT have a filled out ptc data. We ensure everything is set appropriately later.
    //If we pulled an existing page from the database, it MAY have the ptc field filled out. In all other cases, it is None.

    fullpage.main.text = Some(form.text.clone()); 
    fullpage.main.name = Some(form.title.clone());
    fullpage.main.description = Some(form.description.clone());
    fullpage.main.keywords = Some(parse_compound_value(&form.keywords));
    fullpage.main.hash = form.hash.clone();
    if let Some(ref categories) = form.categories {
        add_category_taglist(parse_compound_value(categories), &mut fullpage.main);
    }

    if let Some(ref ptc_files) = form.ptc_files 
    {
        //There is ptc data from the form, so set it. The next check determines if there was already data 
        //from the database or not, and creates a new pending content if not.
        if let Some(ref mut ptc_page) = fullpage.ptc {
            ptc_page.text = Some(ptc_files.clone());
        }
        else {
            //We set VERY LITTLE data on the ptc page because we dont' have much to say about it. The data is pre-formatted
            //by the javascript, and it goes as-is into the text field. We don't even set the parent id, because it can't be
            //known at this point (if we're in this 'else' branch, we have no data anyway)
            let mut ptc_page = Content::default();
            ptc_page.contentType = Some(ContentType::PAGE);
            ptc_page.literalType = Some(PTCSYSTEM.to_string());
            ptc_page.name = Some(format!("PTC files container for {}", form.title));
            ptc_page.permissions = Some(make_permissions! {
                "0": "CR" 
            });
            ptc_page.text = Some(ptc_files.clone());

            fullpage.ptc = Some(ptc_page);
        }
    }
    else {
        //Regardless, throw away the data. Note that we DON'T ACTUALLY DELETE the pending ptc data if the page
        //no longer has it, as that's an invalid state anyway. Although... ugh someone will want to do it. "why can't
        //i remove my key" and all that from sb3, but for ptc. No, I'm just going to say that ptc pages without ptc
        //data is fully invalid and not support that. 
        fullpage.ptc = None;
    }

    //We KNOW there will be values, but might as well do the thing...
    if let Some(ref mut values) = fullpage.main.values 
    {
        //ALWAYS have a markup, it just might come from different places
        values.insert(String::from(SBSValue::MARKUP), 
            if let Some(ref markup) = form.markup { markup }
            else { MARKUPBBCODE }.into());

        if let Some(ref images) = form.images {
            values.insert(SBSValue::IMAGES.to_string(), parse_compound_value(images).into());
        }
        if let Some(ref key) = form.key {
            values.insert(SBSValue::DOWNLOADKEY.to_string(), key.clone().into());
        }
        if let Some(ref size) = form.size {
            values.insert(SBSValue::SIZE.to_string(), size.clone().into());
        }
        if let Some(ref version) = form.version {
            values.insert(SBSValue::VERSION.to_string(), version.clone().into());
        }
        if let Some(ref systems) = form.systems {
            values.insert(SBSValue::SYSTEMS.to_string(), parse_compound_value(systems).into());
        }
        if let Some(ref docpath) = form.docpath {
            values.insert(SBSValue::DOCPATH.to_string(), docpath.clone().into());
        }
    }
    else {
        return Err(Error::Other(String::from("INTERNAL ERROR: Somehow while constructing content, there wasn't a values dictionary!")))
    }

    //So, regardless of new/edit/etc, if we're documentation, we NEED to set the docgroup perms!
    if form.subtype == SBSPageType::DOCUMENTATION {
        if let Some(ref mut perms) = fullpage.main.permissions {
            match context.get_user_by_username(DOCSGROUPUSERNAME, "id,username").await {
                Ok(docsuser) => { perms.insert(docsuser.id.to_string(), "CRUD".to_string()); },
                Err(error) => { println!("Couldn't find docsgroup user!! This is bad: {}", error.to_verbose_string()); }
            }
        }
    }

    Ok(fullpage)
}

pub async fn post_render(mut context: PageContext, form: PageForm) ->
    Result<Response, Error>
{
    if let Some(ref _user) = context.layout_data.user 
    {
        //This one, we throw all the way, since we can't re-render the page without the parent anyway
        let mut written_page : Option<Content> = None;
        let mut errors = Vec::new();

        //Get all the content that will be stored in the database for this form. There may be more than
        //one content to store, but we'll start with main (see next match)
        match construct_post_content_full(&mut context.api_context, &form).await {
            Ok(mut fullpage) =>
            {
                //Store the main content. This is most of the time all that is required, however there are some
                //page types that have more data, which we'll check for within
                match context.api_context.post_content(&fullpage.main).await { 
                    Ok(posted_page) => {
                        //Still have to write the subpages if they exist
                        if let Some(ref mut ptc_page) = fullpage.ptc {
                            ptc_page.parentId = posted_page.id; //Make sure it's pointing to the right place
                            match context.api_context.post_content(&ptc_page).await { 
                                Ok(p) => { println!("Wrote PTC page: {}", i(&p.id)); }, //might do something more later idk
                                Err(e) => { errors.push(e.to_user_string()); }
                            }
                        }
                        written_page = Some(posted_page);
                    },
                    Err(e) => { errors.push(e.to_user_string()); }
                }
            },
            Err(e) => { errors.push(e.to_user_string()); }
        }

        if errors.is_empty() {
            //If there are no errors, we go to the new page
            Ok(Response::Redirect(
                if let Some(ref page) = written_page {
                    context.layout_data.links.forum_thread(page)
                }
                else {
                    Err(Error::Other(String::from("Some internal error occurred, preventing the new page from being shown! No errors produced, but no page data found!")))?
                })
            )
        }
        else {
            //Otherwise, we stay here and show all the terrifying errors
            let render_categories = get_render_categories(&mut context.api_context, &form.subtype).await?;
            let render_docpaths = get_render_docpaths(&mut context.api_context).await?;
            Ok(Response::Render(render(context.layout_data, form, None, render_categories, render_docpaths, Some(errors))))
        }
    }
    else {
        Err(Error::Other(String::from("Not logged in!")))
    }
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

    //Note that this leaves the leftover ptc files, never able to be retrieved because they have no parent. I'm 
    //going to leave it like this because there are benefits to this approach (being able to easily restore the page)

    Ok(Response::Redirect(context.layout_data.links.activity()))
}
