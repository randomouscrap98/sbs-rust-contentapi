use std::collections::HashMap;

use common::*;
use common::constants::SBSPageType;
use common::forms::AdminSearchParams;
use common::forms::BasicPage;
use common::render::*;
use common::prefab::*;
use common::render::layout::*;
use common::view::map_users;
use contentapi::conversion::cast_result_required;
use contentapi::forms::*;
use contentapi::*;

use maud::*;

pub struct AdminRenderData 
{
    pub data: MainLayoutData,
    pub frontpage: Option<Content>,
    pub banner: Option<Content>,
    pub docpage: Option<Content>,
    pub registration_config: RegistrationConfig,
    pub registrationconfig_errors: Option<Vec<String>>,
    pub frontpage_errors: Option<Vec<String>>,
    pub banner_errors: Option<Vec<String>>,
    pub docpage_errors: Option<Vec<String>>,
    pub bans: Vec<UserBan>,
    pub logs: Vec<AdminLog>,
    pub list_users: HashMap<i64, User>
}

impl AdminRenderData
{
    pub fn new_empty(data: MainLayoutData, registration_config: RegistrationConfig) -> Self {
        Self {
            data,
            frontpage: None,
            banner: None,
            docpage: None,
            registration_config,
            registrationconfig_errors: None,
            frontpage_errors: None,
            banner_errors: None,
            docpage_errors: None,
            bans: Vec::new(),
            logs: Vec::new(),
            list_users: HashMap::new()
        }
    }

    pub fn new(data: MainLayoutData, registration_config: RegistrationConfig, frontpage: Option<Content>,
        banner: Option<Content>, docpage: Option<Content>, bans: Vec<UserBan>, logs: Vec<AdminLog>, 
        users: HashMap<i64, User>) -> Self 
    {
        let mut base = Self::new_empty(data, registration_config);
        base.frontpage = frontpage;
        base.banner = banner;
        base.docpage = docpage;
        base.bans = bans;
        base.logs = logs;
        base.list_users = users;
        base
    }
}

pub fn render(render_data: AdminRenderData, search_params: AdminSearchParams) -> String
{
    let mut frontpage_id: i64 = 0;
    let mut frontpage_text: String = String::from("");
    if let Some(frontpage) = render_data.frontpage {
        if let Some(id) = frontpage.id { frontpage_id = id }
        if let Some(text) = frontpage.text { frontpage_text = text.clone() }
    }
    let mut banner_id: i64 = 0;
    let mut banner_text: String = String::from("");
    if let Some(banner) = render_data.banner {
        if let Some(id) = banner.id { banner_id = id }
        if let Some(text) = banner.text { banner_text = text.clone() }
    }
    let mut docpage_id: i64 = 0;
    let mut docpage_text: String = String::from("");
    if let Some(docpage) = render_data.docpage{
        if let Some(id) = docpage.id { docpage_id = id }
        if let Some(text) = docpage.text { docpage_text = text.clone() }
    }
    let data = render_data.data;
    layout(&data, html!{
        (data.links.style("/forpage/admin.css"))
        section {
            @if let Some(user) = &data.user {
                @if user.admin {
                    h3 { "Banning:" }
                    p { "Go to the individual user's page to ban them" }
                    hr;
                    h3 { "Registration config:" }
                    form method="POST" action={(data.links.http_root)"/admin?registrationconfig=1"} {
                        (errorlist(render_data.registrationconfig_errors))
                        label."inline" for="registrationconfig_enabled"{
                            span{"Allow registration:"} 
                            input #"registrationconfig_enabled" type="checkbox" name="enabled" value="true" checked[render_data.registration_config.enabled];
                        }
                        p."error aside" { 
                            "WARN: these settings are temporary and are reset when the server is reset! To make permanent "
                            "changes, talk to the SBS system admin!"
                        }
                        input type="submit" value="Set (NO WARNING, BE CAREFUL!)";
                    }
                    hr;
                    h3 #"update-frontpage" {"Set frontpage (HTML!):"}
                    form."editor" method="POST" action={(data.links.http_root)"/admin?frontpage=1#update-frontpage"} {
                        (errorlist(render_data.frontpage_errors))
                        input type="hidden" name="id" value=(frontpage_id);
                        textarea type="text" name="text"{(frontpage_text)}
                        input type="submit" value="Update";
                    }
                    h3 #"update-alert" {"Set alert banner (HTML!):"}
                    form."editor" method="POST" action={(data.links.http_root)"/admin?alert=1#update-alert"} {
                        (errorlist(render_data.banner_errors))
                        input type="hidden" name="id" value=(banner_id);
                        textarea type="text" name="text"{(banner_text)}
                        input type="submit" value="Update";
                    }
                    h3 #"update-docpage" {"Set Documentation preamble (HTML!):"}
                    form."editor" method="POST" action={(data.links.http_root)"/admin?docscustom=1#update-docpage"} {
                        (errorlist(render_data.docpage_errors))
                        input type="hidden" name="id" value=(docpage_id);
                        textarea type="text" name="text"{(docpage_text)}
                        input type="submit" value="Update";
                    }
                    hr;
                    h3 #"adminlogs" { "Admin log:" }
                    form."smallseparate compactform" action={(data.current())"#adminlogs"} {
                        div."inline smallseparate" {
                            label for="adminlogs_logpage" {"Page:"}
                            input."smallinput" #"adminlogs_logpage" name="logpage" value=(search_params.logpage);
                        }
                        div."inline" {
                            label for="adminlogs_bans_only" {"Bans only:"}
                            input #"adminlogs_bans_only" name="bans_only" type="checkbox" value="true" checked[search_params.bans_only];
                        }
                        input type="submit" value="Update log search";
                    }
                    div."adminlogs" {
                        @for log in render_data.logs {
                            div."resultitem smallseparate" {
                                time."aside" { (d(&log.createDate)) } //Let the javascript take care of the format maybe...
                                span."logid" { "[" (i(&log.id)) "]" } 
                                span."logmessage" { (opt_s!(log.text)) }
                                @if let Some(initiator) = log.initiator {
                                    @if let Some(user) = render_data.list_users.get(&initiator) {
                                        a href=(data.links.user(user)) { "(" (user.username) ")" }
                                    }
                                }
                            }
                        }
                    }
                    hr;
                    h3 #"activebans" { "Active bans:" }
                    form."smallseparate compactform" action={(data.current())"#activebans"} {
                        div."inline smallseparate" {
                            label for="activebans_banpage" {"Page:"}
                            input."smallinput" #"activebans_banpage" name="banpage" value=(search_params.banpage);
                        }
                        input type="submit" value="Update ban search";
                    }
                    div."banlogs" {
                        @for ban in render_data.bans {
                            div."resultitem smallseparate" {
                                time."aside" { (dd(&ban.createDate)) } //Let the javascript take care of the format maybe...
                                span."banid" title={"Id: " (ban.id) ", Type: " (ban.r#type)} { "[" (ban.id) "-" (ban.r#type) "]" } 
                                span."banabout" {
                                    (get_result_user(&data, &render_data.list_users, ban.createUserId))
                                    span { " banned " }
                                    (get_result_user(&data, &render_data.list_users, ban.bannedUserId))
                                    span { " until " (dd(&ban.expireDate)) }
                                }
                                span."aside logmessage" { (opt_s!(ban.message)) }
                            }
                        }
                    }
                }
                @else {
                    p."error" { "You must be an admin to use this page!" }
                }
            }
            @else {
                p."error" { "You must be logged in to use this page!" }
            }
        }
    }).into_string()
}

//So, usually we pass this value in from the config, but I'm rushing and this is just the admin page so it doesn't matter too much
const PERPAGE: i64 = 100;

fn get_result_user(data: &MainLayoutData, all_users: &HashMap<i64, User>, user_id: i64) -> Markup
{
    html!{
        @if let Some(user) = all_users.get(&user_id) {
            a href=(data.links.user(user)) { (user.username) }
        }
        @else {
            span { "???(" (user_id) ")"}
        }
    }
}

/// Generate a basic admin render data, since there's so much required to render the admin page now. 
/// Note that this is the absolute baseline, no errors etc
async fn get_render_data(mut context: PageContext, search: &AdminSearchParams) -> Result<AdminRenderData, Error>
{
    //Need to go lookup some data, use the page to skip. We ask for "all" all the time, because we want
    //them to be LOGS, and admins can get to the user page to see if they're banned maybe...
    let mut request = FullRequest::new();
    //TODO: add all the admin log types to the contentapi crate
    add_value!(request, "bantypes", vec![12, 13]); //12 = ban_create, 13 = ban_edit

    let query = if search.bans_only {
        format!("type in @bantypes")
    }
    else {
        String::from("")
    };

    let logs_request = build_request!(
        RequestType::adminlog,
        String::from("*"),
        query,
        String::from("id_desc"),
        PERPAGE,
        (PERPAGE * search.logpage as i64)
    );
    request.requests.push(logs_request);

    //Only want ACTIVE bans!!
    let bans_request = build_request!(
        RequestType::ban,
        String::from("*"),
        String::from("!activebans()"),
        String::from("id_desc"),
        PERPAGE,
        (PERPAGE * search.banpage as i64)
    );
    request.requests.push(bans_request);

    let users_request = build_request!(
        RequestType::user,
        String::from("*"),
        format!("id in @ban.createUserId or id in @ban.bannedUserId or id in @adminlog.initiator")
    );
    request.requests.push(users_request);

    let result = context.api_context.post_request_profiled_opt(&request, "all_admin_logs").await?;
    let bans = cast_result_required::<UserBan>(&result, "ban")?;
    let logs = cast_result_required::<AdminLog>(&result, "adminlog")?;
    let users = cast_result_required::<User>(&result, "user")?;

    //TODO: link users to bans and then actually find a way to display them!

    Ok(AdminRenderData::new(
        context.layout_data,
        context.api_context.get_registrationconfig().await?,
        get_system_frontpage(&mut context.api_context).await?,
        get_system_alert(&mut context.api_context).await?,
        get_system_docscustom(&mut context.api_context).await?,
        bans, logs, map_users(users)
    ))
}

async fn get_base_render_data(context: PageContext) -> Result<AdminRenderData, Error>
{
    get_render_data(context, &AdminSearchParams::default()).await
}

pub fn render_nosearch(render_data: AdminRenderData) -> Response
{
    Response::Render(render(render_data, AdminSearchParams::default()))
}

pub async fn get_render(context: PageContext, search_params: AdminSearchParams) -> Result<Response, Error> 
{
    Ok(Response::Render(render(get_render_data(context, &search_params).await?, search_params)))
}

pub async fn post_registrationconfig(context: PageContext, form: RegistrationConfig) -> Result<Response, Error>
{
    let mut errors = Vec::new();
    match context.api_context.post_registrationconfig(&form).await {
        Ok(_token) => {} //Don't need the token
        Err(error) => { errors.push(error.to_user_string()) }
    };
    let mut render_data = get_base_render_data(context).await?;
    render_data.registrationconfig_errors = Some(errors);
    Ok(render_nosearch(render_data))
}

async fn to_system_content(form: BasicPage, name: String, literal_type: String) -> Result<Content, Error> {
    let mut content = Content::default();
    //note: the hash it autogenerated from the name (hopefully)
    content.text = Some(form.text);
    content.id = Some(form.id);
    content.contentType = Some(ContentType::SYSTEM);
    content.name = Some(name);
    content.literalType = Some(literal_type);
    content.permissions = Some(make_permissions! { "0": "R" });
    content.values = Some(make_values! { "markup": "html" });
    Ok(content)
}

pub async fn post_frontpage(context: PageContext, form: BasicPage) -> Result<Response, Error>
{
    let mut errors = Vec::new();

    let content = to_system_content(form, String::from("frontpage"), SBSPageType::FRONTPAGE.to_string()).await?;

    match context.api_context.post_content(&content, None).await {
        Ok(_) => {} 
        Err(error) => { errors.push(error.to_user_string()) }
    };
    let mut render_data = get_base_render_data(context).await?;
    render_data.frontpage_errors = Some(errors);
    Ok(render_nosearch(render_data))
}

pub async fn post_alert(mut context: PageContext, form: BasicPage) -> Result<Response, Error>
{
    let mut errors = Vec::new();

    let content = to_system_content(form, String::from("alert"), SBSPageType::ALERT.to_string()).await?;

    match context.api_context.post_content(&content, None).await {
        Ok(new_alert) => { context.layout_data.raw_alert = new_alert.text; } 
        Err(error) => { errors.push(error.to_user_string()) }
    };
    let mut render_data = get_base_render_data(context).await?;
    render_data.banner_errors = Some(errors);
    Ok(render_nosearch(render_data))
}

pub async fn post_docscustom(context: PageContext, form: BasicPage) -> Result<Response, Error>
{
    let mut errors = Vec::new();

    let content = to_system_content(form, String::from("docscustom"), SBSPageType::DOCSCUSTOM.to_string()).await?;

    match context.api_context.post_content(&content, None).await {
        Ok(_new_docspage) => { },
        Err(error) => { errors.push(error.to_user_string()) }
    };
    let mut render_data = get_base_render_data(context).await?;
    render_data.docpage_errors = Some(errors);
    Ok(render_nosearch(render_data))
}
