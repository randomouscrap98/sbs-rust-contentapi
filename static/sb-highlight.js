var highlight_smilebasic=(function(){
var keywords=["BREAK","COMMON","CONTINUE","ELSE","END","ENDIF","REM","REPEAT","THEN","WEND",];
var keywords_sb3=["STOP"];
var keywords_sb4=["OTHERWISE","ENDCASE","LOOP","ENDLOOP"];
var argKeywords=["CALL","DATA","DEC","DIM","ELSEIF","EXEC","FOR","GOSUB","GOTO","IF","INC","INPUT","LINPUT","NEXT","ON","OUT","PRINT","READ","RESTORE","RETURN","SWAP","UNTIL","USE","VAR","WHILE",];
var argKeywords_sb4=["CASE","WHEN","DEFOUT","TPRINT","CONST","ENUM",];
var builtinFunctions=["ABS","ACCEL","ACLS","ACOS","ARYOP","ASC","ASIN","ATAN","ATTR","BACKCOLOR","BEEP","BGMCHK","BGMCLEAR","BGMCONT","BGMPAUSE","BGMPLAY","BGMSET","BGMSETD","BGMSTOP","BGMVAR","BGMVOL","BIN$","BIQUAD","BQPARAM","BREPEAT","BUTTON","CEIL","CHKCALL","CHKCHR","CHKFILE","CHKLABEL","CHKMML","CHKVAR","CHR$","CLASSIFY","CLIPBOARD","CLS","COLOR","CONTROLLER","COPY","COS","COSH","DEG","DELETE","DIALOG","DTREAD","EFCSET","EFCWET","EXP","FADE","FADECHK","FFT","FFTWFN","FILES","FILL","FLOOR","FORMAT$","GBOX","GCIRCLE","GCLIP","GCLS","GCOLOR","GCOPY","GFILL","GLINE","GLOAD","GPAINT","GPSET","GPUTCHR","GSAVE","GTRI","GYROA","GYROSYNC","GYROV","HEX$","IFFT","INKEY$","INSTR","KEY","LEFT$","LEN","LOAD","LOCATE","LOG","MAX","MID$","MIN","OPTION","PCMCONT","PCMSTOP","PCMSTREAM","PCMVOL","POP","POW","PRGDEL","PRGEDIT","PRGGET$","PRGINS","PRGNAME$","PRGSET","PRGSIZE","PROJECT","PUSH","RAD","RANDOMIZE","RENAME","RGB","RIGHT$","RINGCOPY","RND","RNDF","ROUND","RSORT","SAVE","SCROLL","SGN","SHIFT","SIN","SINH","SNDSTOP","SORT","SPANIM","SPCHK","SPCHR","SPCLR","SPCOL","SPCOLOR","SPCOLVEC","SPDEF","SPFUNC","SPHIDE","SPHITINFO","SPHITRC","SPHITSP","SPHOME","SPLINK","SPOFS","SPPAGE","SPROT","SPSCALE","SPSET","SPSHOW","SPSTART","SPSTOP","SPUNLINK","SPUSED","SPVAR","SQR","STICK","STR$","SUBST$","TALK","TALKCHK","TALKSTOP","TAN","TANH","TMREAD","TOUCH","UNSHIFT","VAL","VSYNC","WAIT","WAVSET","WAVSETA","XSCREEN","VIBRATE","PI",
//ptc
"CLEAR","BGMGETV","BGMSETV","BGREAD","BTRIG","CHRINIT","CHRREAD","CHRSET","COLINIT","COLREAD","COLSET","GDRAWMD","ICONCHK","ICONCLR","ICONSET","PNLSTR","PNLTYPE","SENDFILE","RECVFILE","SPANGLE","SPGETV","SPSETV","SPREAD","VISIBLE"];
var builtinFunctions_sb3=["BACKTRACE","BGANIM","BGCHK","BGCLIP","BGCLR","BGCOLOR","BGCOORD","BGCOPY","BGFILL","BGFUNC","BGGET","BGHIDE","BGHOME","BGLOAD","BGOFS","BGPAGE","BGPUT","BGROT","BGSAVE","BGSCALE","BGSCREEN","BGSHOW","BGSTART","BGSTOP","BGVAR","BGMPRG","BGMPRGA","DISPLAY","DLCOPEN","EFCOFF","EFCON","FONTDEF","GOFS","GPAGE","GPRIO","GSPOIT","MICDATA","MICSAVE","MICSTART","MICSTOP","MPEND","MPGET","MPNAME$","MPRECV","MPSEND","MPSET","MPSTART","MPSTAT","STICKEX","RGBREAD","SPCLIP","VISIBLE","WIDTH","XOFF","XON","GPUTCHR16",];
var builtinFunctions_sb4=["PCMPOS","TYPEOF","ARRAY#","ARRAY%","ARRAY$","RESIZE","INSERT","REMOVE","FIND","INSPECT","DEFARGC","DEFARG","DEFOUTC","INT","FLOAT","LAST","FONTINFO","PERFBEGIN","PERFEND","SYSPARAM","METAEDIT","METALOAD","METASAVE","XCTRLSTYLE","MOUSE","MBUTTON","IRSTART","IRSTOP","IRSTATE","IRREAD","IRSPRITE","KEYBOARD","TCPIANO","TCHOUSE","TCROBOT","TCFISHING","TCBIKE","TCVISOR","TCCAR","TCPLANE","TCSUBM","TCVEHICLE","LOADG","LOADV","SAVEG","SAVEV","ANIMDEF","TSCREEN","TPAGE","TCOLOR","TLAYER","TPUT","TFILL","THOME","TOFS","TROT","TSCALE","TSHOW","THIDE","TBLEND","TANIM","TSTOP","TSTART","TCHK","TVAR","TCOPY","TSAVE","TLOAD","TARRAY","TUPDATE","TFUNC","TCOORD","GTARGET","RGBF","HSV","HSVF","GPGET","GARRAY","GUPDATE","GSAMPLE","GPUTCHRP","SPLAYER","STOP","LAYER","LMATRIX","LFILTER","LCLIP","BEEPPIT","BEEPPAN","BEEPVOL","BEEPSTOP","BGMPITCH","BGMWET","EFCEN","SNDMSBAL","SNDMVOL","PRGSEEK","XSUBSCREEN","ENVSTAT","ENVTYPE","ENVLOAD","ENVSAVE","ENVINPUT$","ENVFOCUS","ENVPROJECT","ENVLOCATE","PUSHKEY","HELPGET","HELPINFO","UISTATE","UIMASK","UIPUSHCMPL","DATE$","TIME$","RESULT","CALLIDX","FREEMEM","MILLISEC","MAINCNT"];
var systemVariables=["CALLIDX","CSRX","CSRY","CSRZ","DATE$","TIME$","ERRLINE","ERRNUM","ERRPRG","EXTFEATURE","FREEMEM","HARDWARE","MAINCNT","MICPOS","MICSIZE","MILLISEC","MPCOUNT","MPHOST","MPLOCAL","PCMPOS","PRGSLOT","RESULT","SYSBEEP","TABSTEP","VERSION",
//ptc
"ERR","ERL","MAINCNTH","MAINCNTL","TCHST","TCHTIME","TCHX","TCHY","ICONPUSE","ICONPAGE","ICONPMAX","FUNCNO","FREEVAR","KEYBOARD","SPHITNO","SPHITX","SPHITY","MEM$","PRGNAME$","PACKAGE$"];
var wordConst=["TRUE","FALSE",
//ptc
"CANCEL"];
var wordOperators=["AND","OR","XOR","NOT"];
var wordOperators_sb3Plus=["DIV","MOD"];
function isAlpha(c){return c>='A'&&c<='Z'||c>='a'&&c<='z';}
function isDigit(c){return c>='0'&&c<='9';}
function isInExpr(type){return type=="argkeyword"||type=="function"||type=="operator"||type=="name"||type=="equals"||type=="expr";}
return function(code,callback,sb4){var i=-1,c;function next(){i++;c=code.charAt(i);}
function jump(pos){i=pos-1;next();}
var prev=0;var prevType="start";function push(type,cssType){var word=code.substring(prev,i);prev=i;if(type=="word"){var upper=word.toUpperCase();if(wordConst.indexOf(upper)>=0){type="number";cssType="true-false number";}else if(wordOperators.indexOf(upper)>=0||wordOperators_sb3Plus.indexOf(upper)>=0){type="operator";cssType="word-operator operator";}else if(upper=="DEF"){type="def";cssType="def keyword";}else if(upper=="T"&&c=='?'){word+=c;next();prev=i;type="keyword";cssType="keyword";}else if(keywords.indexOf(upper)>=0||sb4==false&&keywords_sb3.indexOf(upper)>=0||sb4!=false&&keywords_sb4.indexOf(upper)>=0){type="keyword";cssType="keyword";}else if(argKeywords.indexOf(upper)>=0||sb4!=false&&argKeywords_sb4.indexOf(upper)>=0){type="argkeyword";cssType="keyword";}else if(prevType=="def"){type="name";cssType="name";}else{var fPos=i;while(c==' '||c=='\t')
next();var isFunc=false;if(isInExpr(prevType)){if(c=="(")
isFunc=true;}else{isFunc=true;if(c=="["){isFunc=false;}else if(c=="="){next();if(c!="=")
isFunc=false;}}
if(isFunc){type="function";if(builtinFunctions.indexOf(upper)!=-1||sb4!=true&&builtinFunctions_sb3.indexOf(upper)!=-1||sb4!=false&&builtinFunctions_sb4.indexOf(upper)!=-1)
cssType="statement function";else if(upper=="TO"||upper=="STEP")
cssType="to-step keyword";else
cssType="statement";}else{type="variable"
if(sb4!=true&&systemVariables.indexOf(upper)!=-1)
cssType="variable function";else
cssType="variable";}
jump(fPos);}}else if(type=="label"){if(isInExpr(prevType)){type="string";cssType="label-string string";}else{cssType="label";}}else{if(cssType==undefined)
cssType=type;}
callback(word,cssType);if(type!="whitespace")
prevType=type;}
next();while(c){if(isAlpha(c)||c=='_'){next();while(isAlpha(c)||isDigit(c)||c=='_')
next();if(c=='#'||c=='%'||c=='$')
next();push("word");}else if(isDigit(c)||c=='.'){while(isDigit(c))
next();if(c=='.'){next();if(isDigit(c)){next();while(isDigit(c))
next();}else{if(c=='#')
next();push("number");continue;}}
if(c=='E'||c=='e'){var ePos=i;next();if(c=='+'||c=='-')
next();if(isDigit(c)){next();while(isDigit(c))
next();}else{jump(ePos);push();continue;}}
if(c=='#')
next();push("number");}else switch(c){case '"':next();while(c&&c!='"'&&c!='\n'&&c!='\r')
next();if(c=='"')
next();push("string");break;case '\'':next();while(c&&c!='\n'&&c!='\r')
next();push("comment");break;case '&':next();switch(c){case '&':next();push("operator");break;case 'H':case 'h':var hPos=i;next();if(isDigit(c)||c>='A'&&c<='F'||c>='a'&&c<='f'||(c=='_'&&sb4!=false)){next();while(isDigit(c)||c>='A'&&c<='F'||c>='a'&&c<='f'||(c=='_'&&sb4!=false))
next();push("number");}else{jump(hPos);push();}
break;case 'B':case 'b':var bPos=i;next();if(c=='0'||c=='1'||(c=='_'&&sb4!=false)){next();while(c=='0'||c=='1'||(c=='_'&&sb4!=false))
next();push("number");}else{jump(bPos);push();}
break;default:push();}
break;case '@':next();while(isDigit(c)||isAlpha(c)||c=='_')
next();push("label");break;case '#':next();if(isDigit(c)||isAlpha(c)||c=='_'){next();while(isDigit(c)||isAlpha(c)||c=='_')
next();if(c=='#'||c=='%'||c=='$')
next();push("number","constant number");}else{if(c=='#'||c=='%'||c=='$'){next();push("number","constant number");}else{push();}}
break;case '|':next();if(c=='|'){next();push("operator");}else{push();}
break;case '<':next();if(c=='='||c=='<')
next();push("operator");break;case '>':next();if(c=='='||c=='>')
next();push("operator");break;case '=':next();if(c=='='){next();push("operator");}else{push("equals");}
break;case '!':next();if(c=='=')
next();push("operator");break;case '+':case '-':case '*':case '/':next();push("operator");break;case '\\':next();if(sb4==false){push(undefined,false);}else{while(c&&c!='\n'&&c!='\r')
next();next();push("whitespace")}
break;case ';':case ',':case '[':case '(':next();push("expr",false);break;case '\n':next();push("linebreak",false);break;case ":":case ")":case "]":next();push("noexpr",false);break;case " ":case "\t":next();push("whitespace",false);break;case '?':next();push("argkeyword","question keyword");break;default:next();push(undefined,false);}}
push("eof");}})();function applySyntaxHighlighting(element){function escapeHTML(text){return text.replace(/&/g,"&amp;").replace(/</g,"&lt;");}
var lang=element.dataset.code;if(lang)
lang=lang.toLowerCase();var html="";var text=element.textContent;if(!lang||lang=="sb3"||lang=="sb4"||lang=="ptc"||lang=="sb2"){if(lang=="sb4")
lang=true;else if(lang=="sb3")
lang=false;else if(lang=="sb2"||lang=="ptc")
lang=false;else
lang=undefined;var prevType=false;function callback(word,type){if(word){if(type!=prevType){if(prevType)
html+="</span>";if(type)
html+="<span class=\""+type+"\">";}
html+=escapeHTML(word);prevType=type;}}
highlight_smilebasic(text,callback,lang);if(prevType)
html+="</span>";}else{html=escapeHTML(text)}
element.innerHTML=html;}