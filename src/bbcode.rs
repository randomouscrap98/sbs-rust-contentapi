use regex::Regex;

//So:
//- dupes auto close previous scope.
//- ignore unmatched closing tags

//The user provides this
#[derive(Debug, Clone)]
pub struct TagInfo {
    //The tag identity, such as "b", "youtube", etc
    pub tag : &'static str,
    pub argout : Option<&'static str>,
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
    //Maybe get rid of anyhow if you want to separate this, kind of a big thing to include.
    //Anyway, this build function precompiles all the regex for you. Try to reuse this item
    //as much as possible, it doesn't require mutation and so can be part of the global rocket state
    pub fn build(taginfos: Vec<TagInfo>) -> Result<Self, anyhow::Error> {
        //First, get the default direct replacements
        let mut tags = Self::html_escapes().iter().map(|e| {
            //Unfortunately, have to allocate string
            let regstring = format!(r"^{}", e.0);
            Ok(TagDo { 
                regex: Regex::new(&regstring)?,
                match_type : MatchType::DirectReplace(e.1)
            })
        }).collect::<Result<Vec<TagDo>,anyhow::Error>>()?;

        //Next, convert the taginfos to even more "do".
        for tag in taginfos.iter() {
            //The existing system on SBS doesn't allow spaces in tags at ALL. I don't know if this 
            //much leniency on the = value is present in the old system though...
            let open_tag = format!(r#"^\[{}(=[^\]]*)?\]"#, tag.tag);
            tags.push(TagDo {
                regex: Regex::new(&open_tag)?,
                match_type : MatchType::Open(tag.clone())
            });
            let close_tag = format!(r#"^\[/{}\]"#, tag.tag);
            tags.push(TagDo {
                regex: Regex::new(&close_tag)?,
                match_type : MatchType::Close(tag.clone())
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

    //Get a vector of the basic taginfos of bbcode
    pub fn basics() -> Vec<TagInfo> {
        vec![
            TagInfo { tag: "b", argout: None, verbatim: false },
            TagInfo { tag: "i", argout: None, verbatim: false },
            TagInfo { tag: "sup", argout: None, verbatim: false },
            TagInfo { tag: "sub", argout: None, verbatim: false },
            TagInfo { tag: "url", argout: None, verbatim: true },
            TagInfo { tag: "img", argout: None, verbatim: true },
            TagInfo { tag: "s", argout: None, verbatim: false },
            TagInfo { tag: "u", argout: None, verbatim: false }
        ]
    }

    fn push_open_tag(mut result: String, info: &TagInfo, capture: &regex::Captures) -> String {
        result.push_str("<");
        result.push_str(info.tag);
        if let Some(argout) = info.argout {
            if capture.len() > 1 {
                result.push_str(" ");
                result.push_str(argout);
                result.push_str("=\"");
                //Now we need an html escaper
                result.push_str(&html_escape::encode_quoted_attribute(&capture[1]));
                //html_escape::encode_quoted_attribute_to_string(capture[1].into(), mut result);
                result.push_str("\"");
            }
        }
        result.push_str(">");
        result
    }

    fn push_close_tag(mut result: String, info: &TagInfo) -> String {
        result.push_str("</");
        result.push_str(info.tag);
        result.push_str(">");
        result
    }

    pub fn parse(&self, input: &str) -> String 
    {
        //We know it will be at LEAST as big, and that strings usually double in size
        //when they grow anyway, so just start at 2X by default
        let mut result = String::with_capacity(input.len() * 2);

        //Because of utf-8, it's better to just use regex directly all the time?
        let mut slice = &input[0..]; //Not necessary to be this explicit ofc

        //Only 'Taginfo' can create scope, so don't worry about "DirectReplace" types
        let mut scopes : Vec<&TagInfo> = Vec::new();

        //While there is string left, keep checking against all the regex. Remove some regex
        //if the current scope is a meanie
        while slice.len() > 0
        {
            //Slow? IDK
            let verbatim_scope = scopes.last().and_then(|i| Some(i.verbatim)).unwrap_or(false);

            let mut matched_do : Option<&TagDo> = None;

            for tagdo in &self.tags {
                if verbatim_scope && !matches!(tagdo.match_type, MatchType::DirectReplace(_)) {
                    continue;
                }
                if tagdo.regex.is_match(slice) {
                    matched_do = Some(tagdo);
                    break;
                }
            }

            if let Some(tagdo) = matched_do {
                //There should only be one but whatever
                for capture in tagdo.regex.captures_iter(slice) {
                    match &tagdo.match_type {
                        MatchType::DirectReplace(new_text) => {
                            result.push_str(new_text);
                        },
                        MatchType::Open(info) => {
                            result = Self::push_open_tag(result, info, &capture);
                        },
                        MatchType::Close(info) => {
                            result = Self::push_close_tag(result, info);
                        }
                    }
                    slice = &slice[capture[0].len()..];
                }
            }
            else {
                //just move forward and emit the char. Note that the slice is in bytes, but the char
                //is a unicode scalar that could be up to 4 bytes, so we need to know how many 'bytes'
                //we just popped off
                if let Some(ch) = slice.chars().next() {
                    result.push(ch);
                    slice = &slice[ch.len_utf8()..];
                }
                else {
                    println!("In BBCode::parse, there were no more characters but there were leftover bytes!");
                    break;
                }
            }

            //each regex until one matches. skip regexes that aren't direct replacement if 
            //current scope is "verbatim". if direct replace, just dump direct replacement to
            //result. if open tag, dump it to stream too, add to scope. if closing tag, scan backwards
            //until a matching scope is found, pop all scopes and write them in pop order to the result,
            //and finally the last scope too. if none are found, fully ignore close and continue.
        }

        result
    }
}



// ----------------------------
// *         TESTS           
// ----------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_init() {
        //This shouldn't fail?
        let _bbcode = BBCode::build(BBCode::basics()).unwrap();
    }

    #[test]
    fn build_add_lt() {
        //This shouldn't fail?
        let bbcode = BBCode::build(BBCode::basics()).unwrap();
        let found = bbcode.tags.iter().find(|x| matches!(x.match_type, MatchType::DirectReplace(_))).unwrap();
        assert_eq!(found.regex.as_str(), "^<");
        if let MatchType::DirectReplace(repl) = found.match_type {
            assert_eq!(repl, "&lt;")
        }
        else {
            panic!("TEST LOGIC ERROR, NOT DIRECTREPLACE TYPE");
        }
    }

    #[test]
    fn no_alter() {
        let bbcode = BBCode::build(BBCode::basics()).unwrap();
        assert_eq!(bbcode.parse("hello"), "hello");
    }

    #[test]
    fn lt_single() {
        let bbcode = BBCode::build(BBCode::basics()).unwrap();
        assert_eq!(bbcode.parse("h<ello"), "h&lt;ello");
    }

    #[test]
    fn gt_single() {
        let bbcode = BBCode::build(BBCode::basics()).unwrap();
        assert_eq!(bbcode.parse("h>ello"), "h&gt;ello");
    }

    #[test]
    fn amp_single() {
        let bbcode = BBCode::build(BBCode::basics()).unwrap();
        assert_eq!(bbcode.parse("h&ello"), "h&amp;ello");
    }

    #[test]
    fn quote_single() {
        let bbcode = BBCode::build(BBCode::basics()).unwrap();
        assert_eq!(bbcode.parse("h'ello"), "h&#39;ello");
    }

    #[test]
    fn doublequote_single() {
        let bbcode = BBCode::build(BBCode::basics()).unwrap();
        assert_eq!(bbcode.parse("h\"ello"), "h&quot;ello");
    }

    #[test]
    fn complex_escape() {
        let bbcode = BBCode::build(BBCode::basics()).unwrap();
        assert_eq!(bbcode.parse(
            //this has both 4 byte single scalar unicde and a big 25 byte combo one
            "it's a %CRAZY% <world> 💙=\"yeah\" 👨‍👨‍👧‍👦>>done"), 
            "it&#39;s a %CRAZY% &lt;world&gt; 💙=&quot;yeah&quot; 👨‍👨‍👧‍👦&gt;&gt;done"
        );
    }

    #[test]
    fn simple_bold() {
        let bbcode = BBCode::build(BBCode::basics()).unwrap();
        assert_eq!(bbcode.parse("[b]hello[/b]"), "<b>hello</b>");
    }

}