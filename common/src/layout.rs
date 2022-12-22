use super::*;

//Render basic navigation link with only text as the body
pub fn main_nav_link(data: &MainLayoutData, text: &str, href: &str, id: Option<&str>) -> Markup {
    main_nav_link_raw(data, PreEscaped(String::from(text)), href, id)
}

//Produce a link for site navigation which supports highlighting if on current page. Body can be "anything"
pub fn main_nav_link_raw(data: &MainLayoutData, body: Markup, href: &str, id: Option<&str>) -> Markup {
    let mut class = String::from("plainlink headertab");
    let compare_path = match &data.override_nav_path {
        Some(path) => path,
        None => data.current_path.as_str()
    };
    if compare_path.starts_with(href) { class.push_str(" current"); }
    html! {
        a.(class) href={(data.config.http_root) (href)} id=[id] { (body) }
    }
}

pub fn header(data: &MainLayoutData) -> Markup {
    html! {
        header."controlbar" {
            nav {
                a."plainlink" #"homelink" href={(data.config.http_root)"/"}{
                    img src={(data.config.resource_root)"/favicon.ico"};
                    (main_nav_link(data,"Activity","/activity",None))
                    (main_nav_link(data,"Browse","/search",None))
                    (main_nav_link(data,"Forums","/forum",None))
                }
            }
            div #"header-user" {
                @if let Some(user) = &data.user {
                    (main_nav_link_raw(data,html! {
                        span { (user.username) }
                        img src=(image_link(&data.config, &user.avatar, 100, true));
                    },"/userhome",None))
                }
                @else {
                    (main_nav_link(data,"Login","/login",None))
                }
            }
        }
    }
}

//Produce the footer for the main selection of pages
pub fn footer(data: &MainLayoutData) -> Markup {
    html! {
        footer class="controlbar smallseparate" {
            span #"api_about" { (data.about_api.environment) " - " (data.about_api.version) }
            div #"footer-spacer" {}
            (main_nav_link(data,"Settings","/sessionsettings",Some("footer-settings")))
            (main_nav_link(data,"About","/about",Some("footer-about")))
            //<!--<span id="debug">{{client_ip}}</span>-->
            //<!--<span id="debug">{{route_path}}</span>-->
        }
    }
}

/// Basic skeleton to output a blank page with some pre-baked stuff from user settings and required
/// css/js. NOTE: YOU'LL BE USING THIS FOR ALL WIDGETS!
pub fn basic_skeleton(data: &MainLayoutData, head_inner: Markup, body_inner: Markup) -> Markup 
{
    //If available, this is MILLISECONDS
    #[allow(unused_assignments, dead_code, unused_mut)]
    let mut profile_data: Option<HashMap<String,f64>> = None;

    #[cfg(feature = "profiling")]
    {
        profile_data = Some(data.profiler.list_copy().into_iter()
            .map(|pd| (pd.name, pd.duration.as_secs_f64() * 1000f64)).collect());
    }

    html! {
        (DOCTYPE)
        html lang=(data.user_config.language) {
            head {
                (basic_meta(&data.config))
                (style(&data.config, "/base.css"))
                (style(&data.config, "/themes.css"))
                (script(&data.config, "/base.js"))
                (head_inner)
            }
        }
        body data-compact[data.user_config.compact]
             data-theme=(data.user_config.theme) 
        { 
            (body_inner) 
            //Gotta do it HERE so everything has already run!
            @if let Some(profile_data) = profile_data {
                script {
                    "var profiler_data = "(PreEscaped(serde_json::to_string(&profile_data).unwrap_or(String::from("{} /* COULD NOT SERIALIZE */"))))";"
                }
            }
        }
    }
}


pub fn layout(main_data: &MainLayoutData, page: Markup) -> Markup {
    basic_skeleton(main_data, html!{
        title { "SmileBASIC Source" }
        meta name="description" content="A community for sharing programs and getting advice on SmileBASIC applications on the Nintendo DSi, 3DS, and Switch";
        (style(&main_data.config, "/layout.css"))
        (script(&main_data.config, "/sb-highlight.js"))
        //MUST come after, it uses sb-highlight!
        (script(&main_data.config, "/layout.js"))
        style { (PreEscaped(r#"
            body {
                background-repeat: repeat;
                background-image: url(""#))(main_data.config.resource_root)(PreEscaped(r#"/sb-tile.png")
            }
            "#))
        }
    }, html! {
        (header(&main_data))
        main { 
            section {
                p { 
                    span."error" { "This is a preview website! Changes will not carry over or be saved in the end! " }
                    "Original website still up at " a href="https://old.smilebasicsource.com" { "https://old.smilebasicsource.com" }
                }
            }
            (page) 
        }
        (footer(&main_data))
    })
}