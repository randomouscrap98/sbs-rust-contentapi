use regex::Regex;

//So:
//- dupes auto close previous scope.
//- ignore unmatched closing tags

//The user provides this
#[derive(Debug, Clone)]
pub struct TagInfo {
    //The tag identity, such as "b", "youtube", etc
    pub tag : &'static str,
    pub argout : &'static str,
    pub verbatim : bool //whether tags are allowed inside, basically
}

//Bbcode tags come in... different flavors maybe (or this is bad?)
#[derive(Debug, Clone)]
pub enum MatchType { 
    Open(TagInfo), //this is so small, it's fine to dupe in open/close
    Close(TagInfo),
    DirectReplace(&'static str)
}

//The system uses this 
#[derive(Debug, Clone)]
pub struct TagDo {
    pub regex : Regex,
    pub match_type: MatchType,
    //pub info : TagInfo 
}

pub struct BBCode {
    //These are ALWAYS processed
    //pub directs : Vec<(&'static str, &'static str)>,
    //These are SOMETIMES processed (based on context)
    pub tags : Vec<TagDo>
}

impl BBCode {
    //Maybe get rid of anyhow if you want to separate this, kind of a big thing to include
    pub fn build(taginfos: Vec<TagInfo>) -> Result<Self, anyhow::Error> {
        //First, get the default direct replacements
        let mut tags = Self::html_escapes().iter().map(|e| {
            //Unfortunately, have to allocate string
            let regstring = format!(r"^{}", e.1);
            Ok(TagDo { 
                regex: Regex::new(&regstring)?,
                match_type : MatchType::DirectReplace(e.1)
            })
        }).collect::<Result<Vec<TagDo>,anyhow::Error>>()?;

        //Next, convert the taginfos to even more "do"
        for tag in taginfos.iter() {
            //The existing system on SBS doesn't allow spaces in tags at ALL. I don't know if this 
            //much leniency on the = value is present in the old system though...
            let open_tag = format!(r#"^\[{}(=[^\<>']"]*)?\]"#, tag.tag);
            tags.push(TagDo {
                regex: Regex::new(&open_tag)?,
                match_type : MatchType::Open(tag.clone())
            });
            let close_tag = format!(r#"^\[/{}\]"#, tag.tag);
            tags.push(TagDo {
                regex: Regex::new(&close_tag)?,
                match_type : MatchType::Open(tag.clone())
            });
        }

        Ok(BBCode { tags })
    }

    pub fn html_escapes() -> Vec<(&'static str, &'static str)> {
        vec![
            ("<", "&lt;"),
            (">", "&gt;"),
            ("&", "&amp;"),
            ("\"", "&quot;"),
            ("'", "&#39;")
        ]
    }

    pub fn parse(&self, input: String) -> String {
        //We know it will be at LEAST as big, and that strings usually double in size
        //when they grow anyway, so just start at 2X by default
        let result = String::with_capacity(input.len() * 2);

        //Because of utf-8, it's better to just use regex directly all the time?
        let slice = &input[0..]; //Not necessary to be this explicit ofc

        let mut scopes : Vec<&str> = Vec::new();

        //While there is string
        while slice.len() > 0
        {

        }

        result
    }
}