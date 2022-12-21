use super::*;

//Render basic navigation link with only text as the body
pub fn main_nav_link(config: &LinkConfig, text: &str, href: &str, current_path: &str, id: Option<&str>) -> Markup {
    main_nav_link_raw(config, PreEscaped(String::from(text)), href, current_path, id)
}

//Produce a link for site navigation which supports highlighting if on current page. Body can be "anything"
pub fn main_nav_link_raw(config: &LinkConfig, body: Markup, href: &str, current_path: &str, id: Option<&str>) -> Markup {
    let mut class = String::from("plainlink headertab");
    if current_path.starts_with(href) { class.push_str(" current"); }
    html! {
        a.(class) href={(config.http_root) (href)} id=[id] { (body) }
    }
}

//Produce just the inner user element (not the link itself) for a logged-in user
pub fn header_user_inner(config: &LinkConfig, user: &contentapi::User) -> Markup {
    html! {
        span { (user.username) }
        img src=(image_link(config, &user.avatar, 100, true));
    }
}

pub fn header(config: &LinkConfig, current_path: &str, user: &Option<contentapi::User>) -> Markup {
    html! {
        header."controlbar" {
            nav {
                a."plainlink" #"homelink" href={(config.http_root)"/"}{
                    img src={(config.resource_root)"/favicon.ico"};
                    (main_nav_link(config,"Activity","/activity",current_path,None))
                    (main_nav_link(config,"Browse","/search",current_path,None))
                    (main_nav_link(config,"Forums","/forum",current_path,None))
                }
            }
            div #"header-user" {
                @if let Some(user) = user {
                    (main_nav_link_raw(config,header_user_inner(config,user),"/userhome",current_path,None))
                }
                @else {
                    (main_nav_link(config,"Login","/login",current_path,None))
                }
            }
        }
    }
}

//Produce the footer for the main selection of pages
pub fn footer(config: &LinkConfig, about_api: &contentapi::About, current_path: &str) -> Markup {
    html! {
        footer class="controlbar smallseparate" {
            span #"api_about" { (about_api.environment) " - " (about_api.version) }
            div #"footer-spacer" {}
            (main_nav_link(config,"Settings","/sessionsettings",current_path,Some("footer-settings")))
            (main_nav_link(config,"About","/about",current_path,Some("footer-about")))
            //<!--<span id="debug">{{client_ip}}</span>-->
            //<!--<span id="debug">{{route_path}}</span>-->
        }
    }
}

/// Basic skeleton to output a blank page with some pre-baked stuff from user settings and required
/// css/js. NOTE: YOU'LL BE USING THIS FOR ALL WIDGETS!
pub fn basic_skeleton(data: &MainLayoutData, head_inner: Markup, body_inner: Markup) -> Markup {
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
        }
    }
}


pub fn layout(main_data: &MainLayoutData, page: Markup) -> Markup {
    //If available, this is MILLISECONDS
    #[allow(unused_assignments, dead_code, unused_mut)]
    let mut profile_data: Option<HashMap<String,f64>> = None;

    #[cfg(feature = "profiling")]
    {
        profile_data = Some(main_data.profiler.list_copy().into_iter()
            .map(|pd| (pd.name, pd.duration.as_secs_f64() * 1000f64)).collect());
    }

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
        (header(&main_data.config, &main_data.current_path, &main_data.user))
        main { 
            section {
                p { 
                    span."error" { "This is a preview website! Changes will not carry over or be saved in the end! " }
                    "Original website still up at " a href="https://old.smilebasicsource.com" { "https://old.smilebasicsource.com" }
                }
            }
            (page) 
        }
        (footer(&main_data.config, &main_data.about_api, &main_data.current_path ))
        //Gotta do it HERE so everything has already run!
        @if let Some(profile_data) = profile_data {
            script {
                "var profiler_data = "(PreEscaped(serde_json::to_string(&profile_data).unwrap_or(String::from("{} /* COULD NOT SERIALIZE */"))))";"
                //(PreEscaped(r#"console.log("Profiling data:", profile_data);"#))
            }
        }
    })
}