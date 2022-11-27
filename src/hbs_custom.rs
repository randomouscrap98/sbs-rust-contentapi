use rocket_dyn_templates::handlebars::{self, Handlebars, JsonRender};

static FILERAWKEY: &'static str = "api_fileraw";

//The helper signature is just TOO DAMN COMPLICATED (I know you need those
//params sometimes) so I'm just... simplifying it
macro_rules! generate_helper {
    ($name:ident, $h:ident, $out:ident, $ctx:ident, $code:block) => {
        fn $name(
            $h: &handlebars::Helper<'_, '_>,
            _: &handlebars::Handlebars,
            $ctx : &handlebars::Context,
            _: &mut handlebars::RenderContext<'_, '_>,
            $out: &mut dyn handlebars::Output
        ) -> handlebars::HelperResult {
            $code
            Ok(())
        }
    };
}

generate_helper!{avatar_helper, h, out, ctx, {
    if let Some(param) = h.param(0) {
        if let Some(fileroot) = ctx.data().get(FILERAWKEY).and_then(|v| v.as_str())
        {
            out.write("<b><i>")?;
            out.write(&param.value().render())?;
            out.write("</b></i>")?;
        }
    }
}}

pub fn customize(hbs: &mut Handlebars) {
    hbs.register_helper("avatar", Box::new(avatar_helper));
    //hbs.register_template_string("hbs/about.html", r#"
    //    {{#*inline "page"}}

    //    <section id="about">
    //      <h1>About - Here's another page!</h1>
    //    </section>

    //    {{/inline}}
    //    {{> hbs/layout}}
    //"#).expect("valid HBS template");
}
