use scraper::{ElementRef, Selector};

#[derive(Debug)]
pub struct Simple {
    pub attribute: String,
    pub explains: Vec<String>,
}

#[derive(Debug)]
pub struct Example {
    pub original: String,
    pub translate: String,
}

#[derive(Debug)]
pub struct ExplainsAndExample {
    pub explain: String,
    pub examples: Vec<Example>,
}

#[derive(Debug)]
pub struct Detail {
    pub source: String,
    pub attribute: String,
    pub explains: Vec<ExplainsAndExample>,
}

#[derive(Debug)]
pub struct Word {
    pub word: String,
    pub katakana: String,
    pub audio_url: String,
    pub simple: Vec<Simple>,
    pub detail: Vec<Detail>,
}

static USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/69.0.3497.81 Safari/537.36";
static COOKIE: &str = "HJ_UID=0f406091-be97-6b64-f1fc-f7b2470883e9; HJ_CST=1; HJ_CSST_3=1;TRACKSITEMAP=3%2C; HJ_SID=393c85c7-abac-f408-6a32-a1f125d7e8c6; _REF=; HJ_SSID_3=4a460f19-c0ae-12a7-8e86-6e360f69ec9b; _SREF_3=; HJ_CMATCH=1";

pub async fn get(word: &str, t: &str) -> Result<Vec<Word>, reqwest::Error> {
    let r = reqwest::Client::builder()
        .build()?
        .get(format!("https://dict.hjenglish.com/jp/{}/{}", t, word))
        .header("User-Agent", USER_AGENT)
        .header("Cookie", COOKIE)
        .send()
        .await?;

    let text = r.text().await?;

    // println!("{}", text);
    Ok(parse(&text))
}

fn parse_detail(element: ElementRef) -> Vec<Detail> {
    let mut eps: Vec<Detail> = vec![];

    let detail_selector = Selector::parse(".word-details-pane-content .word-details-item").unwrap();
    let details = element.select(&detail_selector);
    for detail in details {
        let source_selector = Selector::parse(".detail-source").unwrap();
        let source = match detail.select(&source_selector).next() {
            None => "unknown".to_string(),
            Some(v) => v.text().collect::<Vec<_>>().join(" "),
        };

        let dl_selector = Selector::parse(".word-details-item-content .detail-groups dl").unwrap();
        let dls = detail.select(&dl_selector);

        for dl in dls {
            let attr_selector = Selector::parse("dt").unwrap();
            let attr = match dl.select(&attr_selector).next() {
                None => "unknown".to_string(),
                Some(v) => v.text().collect::<Vec<_>>().join(" ").trim().to_string(),
            };

            let mut d = Detail {
                attribute: attr,
                explains: vec![],
                source: source.clone(),
            };

            let dd_selector = Selector::parse("dd").unwrap();
            let dds = dl.select(&dd_selector);

            for dd in dds {
                let explain_selector = Selector::parse("h3 p").unwrap();
                let mut explain_str = "".to_string();
                for explain in dd.select(&explain_selector) {
                    explain_str += explain.text().collect::<Vec<_>>().join(" ").as_str();
                }

                let mut ep = ExplainsAndExample {
                    explain: explain_str.split_whitespace().collect::<Vec<_>>().join(" "),
                    examples: vec![],
                };
                let example_selector = Selector::parse("ul li").unwrap();
                for example in dd.select(&example_selector) {
                    let from_selector = Selector::parse(".def-sentence-from").unwrap();
                    let to_selector = Selector::parse(".def-sentence-to").unwrap();

                    let from = match example.select(&from_selector).next() {
                        None => "".to_string(),
                        Some(v) => v.text().collect::<Vec<_>>().join(" "),
                    };

                    let to = match example.select(&to_selector).next() {
                        None => "".to_string(),
                        Some(v) => v.text().collect::<Vec<_>>().join(" "),
                    };

                    ep.examples.push(Example {
                        original: from.split_whitespace().collect::<Vec<_>>().join(" "),
                        translate: to.split_whitespace().collect::<Vec<_>>().join(" "),
                    });
                }

                d.explains.push(ep);
            }

            eps.push(d);
        }
    }

    return eps;
}

fn parse_simple(element: ElementRef) -> Vec<Simple> {
    let mut sps: Vec<Simple> = vec![];

    let simple_selector = Selector::parse(".simple").unwrap();

    let simples = element.select(&simple_selector);

    for simple in simples {
        let attributes_selector = Selector::parse("h2").unwrap();
        let mut attributes = simple.select(&attributes_selector);

        let attribute = match attributes.next() {
            None => String::from(""),
            Some(x) => x.inner_html(),
        };

        if attribute == "" {
            let definition_selector = Selector::parse("span.simple-definition").unwrap();

            let definition = match simple.select(&definition_selector).next() {
                None => continue,
                Some(v) => v,
            };

            let html = definition
                .text()
                .collect::<Vec<_>>()
                .join(" ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");

            if html != "" {
                sps.push(Simple {
                    attribute: "".to_string(),
                    explains: vec![html],
                });
            }
            continue;
        }

        let list_selector = Selector::parse("ul").unwrap();
        let list = simple.select(&list_selector);

        for li in list {
            let mut sp = Simple {
                attribute: attribute.clone(),
                explains: vec![],
            };

            let li_selector = Selector::parse("li").unwrap();
            let lis = li.select(&li_selector);

            for li in lis {
                let mut li_text = String::new();
                for child in li.children() {
                    if let Some(text_node) = child.value().as_text() {
                        let trimmed = text_node.trim();
                        if !trimmed.is_empty() {
                            li_text.push_str(trimmed);
                        }
                    }
                }

                if li_text.len() == 0 {
                    continue;
                }

                sp.explains.push(li_text);
            }

            sps.push(sp);
        }
    }
    return sps;
}

fn parse(text: &str) -> Vec<Word> {
    let mut ws: Vec<Word> = vec![];

    let q = scraper::Html::parse_document(&text);

    let selector = scraper::Selector::parse(".word-details-pane").unwrap();

    let res = q.select(&selector);

    for element in res {
        let word_selector = Selector::parse(".word-text h2").unwrap();
        let pronounce_selector = Selector::parse(".pronounces").unwrap();
        let pronounce = element.select(&pronounce_selector).next().unwrap();
        let katakana_selector = Selector::parse("span").unwrap();
        let audio_selector = Selector::parse(".word-audio").unwrap();
        let audio = pronounce
            .select(&audio_selector)
            .next()
            .unwrap()
            .value()
            .attr("data-src")
            .unwrap();

        let mut katakana = String::new();

        for p in pronounce.select(&katakana_selector) {
            katakana.push_str(p.text().collect::<Vec<_>>().join("").trim());
        }

        ws.push(Word {
            word: element
                .select(&word_selector)
                .next()
                .unwrap()
                .text()
                .collect::<Vec<_>>()
                .join("")
                .trim()
                .to_string(),
            katakana: katakana,
            audio_url: audio.to_string(),
            simple: parse_simple(element),
            detail: parse_detail(element),
        });
    }

    return ws;
}

#[cfg(test)]
mod tes {
    use crate::jp::parse;

    #[tokio::test]
    async fn run_parse() {
        println!("{:?}", parse(TEST_DATA));
        println!("{:?}", parse(TEST_DATA_2));
        // println!("{:?}", get("你好", "cj").await.unwrap());
    }

    static TEST_DATA: &str = r#"
<!DOCTYPE html>
<html lang="zh-Hans">
<head>
  <meta charset="utf-8">
  <meta name="keywords" content="kodomo,kodomo是什么意思,kodomo的翻译,kodomo的用法" />
  <meta name="description" content="沪江小D日语在线翻译是免费日语在线翻译网站、提供kodomo的翻译、kodomo是什么意思、kodomo的音标与发音、kodomo的含义及用法、以及kodomo的参考例句、kodomo是什么意思、解析kodomo的含义。" />
  <title>kodomo是什么意思_日语在线翻译_kodomo的翻译_含义_读音_用法_参考例句_沪江小D日语词典</title>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="shortcut icon" href="/favicon.ico">
  <script>
    var Gentian = {
      start_time: +new Date,
      project: '5003',
      business: '1000',
      request_id: 'e52e1c2f725a42bab95c516c90ff09ae'
    };
    
    // http://res.hjfile.cn/co/gentian/error.1.2.0.min.js
    !function(e){function r(o){if(n[o])return n[o].exports;var i=n[o]={exports:{},id:o,loaded:!1};return e[o].call(i.exports,i,i.exports,r),i.loaded=!0,i.exports}var n={};return r.m=e,r.c=n,r.p="",r(0)}([function(e,r){!function(e){function r(e){var r,n,o=encodeURIComponent,i=[];for(r in e)n=e[r],Object(n)!==n&&i.push(r+"="+o(void 0==n||null==n?"":n));return i.join("&")}function n(e){var n=s+r(e);setTimeout(function(){var e=new Image(1,1);e.src=n,n=null},0)}var o=e,i=o.document,t=o.Gentian||o._gentian,a="";a=/qa|local|dev/.test(i.domain)?"//qagentian-frd.hjapi.com":/yz/.test(i.domain)?"//yzgentian-frd.hjapi.com":"//gentian-frd.hjapi.com";var s=a+"/gentian_error.gif?";e.onerror=function(){var e=arguments[0],r=arguments[1],o=arguments[2],i=arguments[3],a=e.toLowerCase(),s="script error",u={};u=a.indexOf(s)>-1?{message:"Script Error: See Browser Console for Detail",page_url:location.href||"",project:t.project||"",business:t.business||"",request_id:t.request_id||"",page_id:t.page_id||0}:{message:e||"Script Error: See Browser Console for Detail",url:r||"",line:o||"",column:i||"",page_url:location.href||"",project:t.project||"",business:t.business||"",request_id:t.request_id||"",page_id:t.page_id||0},n(u)}}(window)}]);
  </script>
  
  <link rel="stylesheet" href="//res.hjfile.cn/co/style/base.min.css" />
  
    
      <link rel="stylesheet" href="//res.hjfile.cn/tool/dict.hjenglish.com/common-892b9.css">
    
    <link rel="stylesheet" href="//res.hjfile.cn/tool/dict.hjenglish.com/word-6db50.css">
  
  <link rel="stylesheet" href="//res.hjfile.cn/lib/hui/footer/0.5.2/footer.css">
  
  <script>
    window.__static_public_path__ = '//res.hjfile.cn/tool/dict.hjenglish.com/';
  </script>
  
  
  
</head>
<body class="word">
<div class="wrapper">
  <header class="header">
  <nav class="main-nav">
    <div class="inner clearfix">
        <ul class="org-nav">
          <li>
            <a href="https://www.hujiang.com/" target="_blank">沪江首页</a>
          </li>
          <li>
            <a href="https://www.hujiang.com/xuexi/" target="_blank">学习资讯</a>
          </li>
          <li class="has-menu">
            学习工具
            <ul class="org-nav-apps">
              <li>
                <a class="app-d" href="https://www.hujiang.com/app/#hjdict" target="_blank">沪江小D</a>
              </li>
              <li>
                <a class="app-ci" href="https://www.hujiang.com/app/#cichang" target="_blank">开心词场</a>
              </li>
              <li>
                <a class="app-hs" href="https://www.hujiang.com/app/hujiang/" target="_blank">沪江学习</a>
              </li>
              <li>
                <a class="app-t" href="https://www.hujiang.com/app/#hjtlk" target="_blank">听力酷</a>
              </li>
            </ul>
          </li>
          <li>
            <a href="https://class.hujiang.com/" target="_blank">沪江网校</a>
          </li>
          <li>
            <a href="https://www.cctalk.com/" target="_blank">CC 课堂</a>
          </li>
        </ul>
        
          <ul class="auth-nav">
            <li>
              <a href="https://login.hujiang.com/?url=http%3A%2F%2Fdict.hjenglish.com%2Fjp%2Fjc%2Fkodomo" rel="nofollow">登录</a>
            </li>
            <li>
              <a href="https://pass.hujiang.com/signup?url=http%3A%2F%2Fdict.hjenglish.com%2Fjp%2Fjc%2Fkodomo" rel="nofollow">注册</a>
            </li>
          </ul>
        

        <div class="ad ad-175x35">
          <script type="text/javascript">
            var uzt_545 = "_" + Math.random().toString(36).slice(2);
            document.write('<div id="' + uzt_545+ '"></div>');
            (window._Uzt_Slots = window._Uzt_Slots || []).push({
                id: 545,
                cid: uzt_545,
              size: "175,35"
            });
          </script>
        </div>
    </div>
  </nav>
  
  <nav class="sub-nav">
    <div class="inner clearfix">
        <a href="/" class="logo">
          <h1>沪江小D</h1>
        </a>
        <ul class="site-nav">
          <li><a class="active" href="/">查词</a></li>
          <li><a class="" href="/app/trans">翻译</a></li>
          <li><a class="" href="/scb">生词本</a></li>
          <li><a target="_blank" href="https://www.hujiang.com/app/#hjdict">手机版小D</a></li>
          <li><a class="manual-translate-header" target="_blank" href="//hj.yeecloud.com">人工翻译</a></li>
        </ul>
    </div>
  </nav>
  
</header>
  <div class="container">
  
<div class="search-wrapper">
  <div class="search" data-lang="jp">
  <a class="search-logo" href="/"></a>
  <ul class="search-tabs">
    
    <li class="" data-lang="en">
      <a href="/">英语</a>
    </li>
    
    <li class="search-tabs-active" data-lang="jp">
      <a href="/jp">日语</a>
    </li>
    
    <li class="" data-lang="kr">
      <a href="/kr">韩语</a>
    </li>
    
    <li class="" data-lang="fr">
      <a href="/fr">法语</a>
    </li>
    
    <li class="" data-lang="de">
      <a href="/de">德语</a>
    </li>
    
    <li class="" data-lang="es">
      <a href="/es">西语</a>
    </li>
    
  </ul>
  <form class="search-form search-form-type-jp clearfix">
    <div class="search-input-wrapper">
      <input type="text" 
        class="search-input" 
        name="word"
        placeholder="在此输入要查询的单词" 
        autocomplete="off"
        value="kodomo"
      >

      <a class="search-vkeyboard">虚拟键盘</a>
    </div>
    <div class="search-buttonpane">
      <div class="search-buttonpane-group">
        <button type="button" class="button">查 询</button>
      </div>
      <div class="search-buttonpane-group search-buttonpane-group-jp">
        <button type="button" class="button" data-trans="jc">日 &rarr; 中</button>
        <button type="button" class="button" data-trans="cj">中 &rarr; 日</button>
      </div>
    </div>
  </form>
</div>


  <div class="ad ad-search-right-text">
    
      <script type="text/javascript">
        var uzt_157 = "_" + Math.random().toString(36).slice(2);
        document.write('<div id="' + uzt_157+ '"></div>');
        (window._Uzt_Slots = window._Uzt_Slots || []).push({
            id: 157,
          cid: uzt_157,
          size: "200,29"
        });
      </script>
      
  </div>
</div>
<main class="main">
  <div class="inner main-inner clearfix">
    <section class="content">
      
        <ul class="word-nav">
  
  <li><a data-id="detail">详细释义</a></li>
  
</ul>
<div class="word-details word-details-multi">

  
  
  <header class="word-details-header">
    <p>该词条有 <span>2</span> 种写法，请选择你所需的词条</p>
    <ul>
      
        <li class="word-details-tab  word-details-tab-active" data-categories="[{&quot;value&quot;:&quot;detail&quot;,&quot;text&quot;:&quot;详细释义&quot;}]">
          <h2>小友</h2>
          <div class="pronounces">
            

            
              
                <span class="pronounce-value">[こども]</span>
              
              
            
          </div>
        </li>
      
        <li class="word-details-tab " data-categories="[{&quot;value&quot;:&quot;detail&quot;,&quot;text&quot;:&quot;详细释义&quot;}]">
          <h2>子供</h2>
          <div class="pronounces">
            

            
              
                <span class="pronounce-value">[こども]</span>
              
              
            
          </div>
        </li>
      
    </ul>
  </header>
  

  <section class="word-details-content">
    
    <div class="word-details-pane">
      <header class="word-details-pane-header word-details-pane-header-multi">
        <div class="word-info">

  
  <div class="word-text">
    <h2>小友</h2>

    
    
    <a href="https://login.hujiang.com/?url=http%3A%2F%2Fdict.hjenglish.com%2Fjp%2Fjc%2Fkodomo" class="add-scb"></a>
    

  </div>

  
  

  
  <div class="pronounces">
    

    
    
      <span>[こども]</span>
      
      <span>[kodomo]</span>
      
      <span class="word-audio" data-src="https://tts.hjapi.com/jp/0ECE7C47D9B2B07889498603D1BD0B3B"></span>

    

  </div>

</div>
        <div class="simple">
  
</div>
      </header>
      <div class="word-details-ads-placeholder"></div>
      <div class="word-details-pane-content" 
        data-word="小友" 
        data-ext-key="こども" 
        data-from-lang="jp" 
        data-to-lang="cn"
        data-word-id="3404110"
        data-keywords="[&quot;小友&quot;]",
        data-remove-id="-1"
      >

      

      
      <div class="word-details-item detail" data-id="detail">
  <h2>详细释义</h2>
  <div class="word-details-item-content">
    <header>

      
      

      
      
        
      

      
      

    </header>
    <section class="detail-groups">

      
      
      
      

      
      

      <dl>
        <dt>
          
          
        </dt>
        
        <dd>
          <h3>

            
              <p></p>
            

            <p>
            
            
            【日本地名】
            

            

            
            

            
            

            
            

            
            

            
            
            </p>
          </h3>

          
          

          <ul>
            
          </ul>
          
          
        </dd>
        
      </dl>
      
    </section>
  </div>
</div>
      

      

      

      

      

      

      
      </div>
      <footer class="word-details-pane-footer">
        <button class="button word-details-button-feedback">纠错</button>
      </footer>
    </div>
    
    <div class="word-details-pane">
      <header class="word-details-pane-header word-details-pane-header-multi">
        <div class="word-info">

  
  <div class="word-text">
    <h2>子供</h2>

    
    
    <a href="https://login.hujiang.com/?url=http%3A%2F%2Fdict.hjenglish.com%2Fjp%2Fjc%2Fkodomo" class="add-scb"></a>
    

  </div>

  
  

  
  <div class="pronounces">
    

    
    
      <span>[こども]</span>
      
      <span>[kodomo]</span>
      <span class="pronounce-value-jp">◎</span>
      <span class="word-audio" data-src="http://d1.g.hjfile.cn/voice/jpsound/J26582.mp3"></span>

    

  </div>

</div>
        <div class="simple">
  

    
    
      
        <h2>【惯用句】</h2>
      
      <ul>
        
          <li><span>1.</span>1、子供のけんかに親が出る。/孩子打架大人出面。</li>
        
      </ul>

    
    
  

    
    
      
        <h2>【名词】</h2>
      
      <ul>
        
          <li><span>1.</span>自己的儿女。（むすこ・むすめ。）</li>
        
          <li><span>2.</span>『参考』自分の子を他人に紹介するときの呼び方：</li>
        
          <li><span>3.</span>儿童,小孩儿。（児童。）</li>
        
          <li><span>4.</span>仔zi，崽zai。（動物の子。）</li>
        
          <li><span>5.</span>幼稚。（考えの未熟な人。）</li>
        
      </ul>

    
    
  
</div>
      </header>
      <div class="word-details-ads-placeholder"></div>
      <div class="word-details-pane-content" 
        data-word="子供" 
        data-ext-key="こども" 
        data-from-lang="jp" 
        data-to-lang="cn"
        data-word-id="3463141"
        data-keywords="[&quot;子供&quot;]",
        data-remove-id="-1"
      >

      

      
      <div class="word-details-item detail" data-id="detail">
  <h2>详细释义</h2>
  <div class="word-details-item-content">
    <header>

      
      

      
      
        
          <p class="detail-source">源自:《现代日汉双解词典》<span class="sflep-icon">外教社</span></p>
        
      

      
      

    </header>
    <section class="detail-groups">

      
      
      
      

      
      

      <dl>
        <dt>
          名词
          
        </dt>
        
        <dd>
          <h3>

            
              <p>幼い子。児童。</p>
            

            <p>
            
            
            小孩子。儿童。
            

            

            
            

            
            

            
            

            
            

            
            
            </p>
          </h3>

          
          

          <ul>
            
            <li>
              <p class="def-sentence-from">
                子供むきの本。
                
                <span class="word-audio" data-src="https://tts.hjapi.com/jp/DE688764BF87AB68884C9C3F6879605995B844E804FF029B"></span>
                
              </p>
              <p class="def-sentence-to">
                儿童读物。
                
              </p>
            </li>
            
            <li>
              <p class="def-sentence-from">
                子供扱いにする。
                
                <span class="word-audio" data-src="https://tts.hjapi.com/jp/4BE0224857DABDE5E53780985256C458C9A9A3BBB9C8B5A005ABDB88389ED4FF"></span>
                
              </p>
              <p class="def-sentence-to">
                当作孩子看待。
                
              </p>
            </li>
            
          </ul>
          
          
        </dd>
        
        <dd>
          <h3>

            
              <p>自分のもうけた子。</p>
            

            <p>
            
            
            自己的儿女。
            

            

            
            

            
            

            
            

            
            

            
            
            </p>
          </h3>

          
          

          <ul>
            
            <li>
              <p class="def-sentence-from">
                わたくしの子供は10歳になる。
                
                <span class="word-audio" data-src="https://tts.hjapi.com/jp/8C59EF4A0807373B6DD78793A5209CBA0AA3FF42BF769F8B6A8F78FDEA4ADA1CC3FC3A4FCCDD303CD463FEA15BD9AB68"></span>
                
              </p>
              <p class="def-sentence-to">
                我的孩子已经十岁了。
                
              </p>
            </li>
            
          </ul>
          
          
        </dd>
        
      </dl>
      
    </section>
  </div>
</div>
      

      

      

      

      

      

      
      </div>
      <footer class="word-details-pane-footer">
        <button class="button word-details-button-feedback">纠错</button>
      </footer>
    </div>
    
  </section>
  <div class="word-details-ads">
    <div class="ad ad-details-text">
  
    <script type="text/javascript">
      var uzt_526 = "_" + Math.random().toString(36).slice(2);
      document.write('<div id="' + uzt_526+ '"></div>');
      (window._Uzt_Slots = window._Uzt_Slots || []).push({
          id: 526,
        cid: uzt_526,
        size: "200,29"
      });
    </script>
    
</div>
  </div>
</div>
      

      

    </section>
    <aside class="side">
    
      <section class="side-block">
        <div class="ad ad-230x160">
  
  <!-- 所有语种共用的右侧栏广告位 -->
  <script type="text/javascript">
    var uzt_531 = "_" + Math.random().toString(36).slice(2);
    document.write('<div id="' + uzt_531 + '"></div>');
    (window._Uzt_Slots = window._Uzt_Slots || []).push({
      id: 531,
      cid: uzt_531,
      size: "230,160"
    });
  </script>
</div>


  <div class="ad ad-230x160">
    <script type="text/javascript">
      var uzt_533 = "_" + Math.random().toString(36).slice(2);
      document.write('<div id="' + uzt_533 + '"></div>');
      (window._Uzt_Slots = window._Uzt_Slots || []).push({
          id: 533,
          cid: uzt_533,
        size: "230,160"
      });
    </script>
  </div>
  <div class="ad ad-230x160">
    <script type="text/javascript">
      var uzt_539 = "_" + Math.random().toString(36).slice(2);
      document.write('<div id="' + uzt_539 + '"></div>');
      (window._Uzt_Slots = window._Uzt_Slots || []).push({
          id: 539,
          cid: uzt_539,
        size: "230,160"
      });
    </script>
  </div>


      </section>

      
      
      <section class="side-block">
        <h2>相关搜索</h2>
        <ul class="side-block-content related-words">
        
          <li>
            <a href="/jp/jc/%E5%B0%8F%E5%8F%8B">小友</a>
          </li>
        
          <li>
            <a href="/jp/jc/%E5%AD%90%E4%BE%9B">子供</a>
          </li>
        
        </ul>
      </section>
      

      
      
      <section class="side-block">
        <h2>今日热词</h2>
        <ul class="side-block-content hot-words">
          
          <li>
            <a href="/jp/jc/%E6%96%B0%E5%93%81">
              新品
            </a>
          </li>
          
          <li>
            <a href="/jp/jc/%E3%82%A2%E3%83%A2%E3%82%A4">
              アモイ
            </a>
          </li>
          
          <li>
            <a href="/jp/jc/%E9%81%8E%E5%8A%B4">
              過労
            </a>
          </li>
          
          <li>
            <a href="/jp/jc/%E6%97%85%E5%85%88">
              旅先
            </a>
          </li>
          
          <li>
            <a href="/jp/jc/%E6%9D%A1%E7%B4%84">
              条約
            </a>
          </li>
          
          <li>
            <a href="/jp/jc/%E8%A8%BC%E5%88%B8%E5%8F%96%E5%BC%95%E6%89%80">
              証券取引所
            </a>
          </li>
          
          <li>
            <a href="/jp/jc/%E9%80%80%E9%99%A3">
              退陣
            </a>
          </li>
          
        </ul>
      </section>
      

      
      
      <section class="side-block">
        <h2>内容推荐</h2>
        <ul class="side-block-content recommand-links">
          
          <li>
            <a href="https://jp.hjenglish.com/n/s/15893/" target="_blank">红白歌会</a>
          </li>
          
          <li>
            <a href="http://jp.hjenglish.com/nenglikaon2/n2linianzhenti/" target="_blank">日语二级真题</a>
          </li>
          
          <li>
            <a href="http://jp.hjenglish.com/n/s/39/" target="_blank">日语等级</a>
          </li>
          
          <li>
            <a href="http://jp.hjenglish.com/n/s/24/" target="_blank">日语自学</a>
          </li>
          
          <li>
            <a href="http://jp.hjenglish.com/n/s/36/" target="_blank">新编日语</a>
          </li>
          
          <li>
            <a href="http://www.hujiang.com/c/" target="_blank">课程专题</a>
          </li>
          
          <li>
            <a href="http://jp.hjenglish.com/zt/riyusanjichengjichaxun/" target="_blank">日语三级成绩查询</a>
          </li>
          
          <li>
            <a href="http://jp.hjenglish.com/zt/riyuerjichengjichaxun/" target="_blank">日语二级成绩查询</a>
          </li>
          
          <li>
            <a href="http://www.hjenglish.com/sijidaan/zuixinsijizuowen/" target="_blank">四级作文范文</a>
          </li>
          
          <li>
            <a href="http://jp.hjenglish.com/jlpt/beikao/" target="_blank">日语等级考试备考</a>
          </li>
          
          <li>
            <a href="http://www.hjenglish.com/wangming/" target="_blank">英文网名</a>
          </li>
          
          <li>
            <a href="http://www.hjenglish.com/sijidaan/siliujizuowenmuban/" target="_blank">英语四级作文万能模板</a>
          </li>
          
        </ul>
      </section>
      

    </aside>
  </div>
</main>

<!-- 弹窗广告 -->
<script type="text/javascript">
  var uzt_547 = "_" + Math.random().toString(36).slice(2);
  document.write('<div id="' + uzt_547+ '"></div>');
  (window._Uzt_Slots = window._Uzt_Slots || []).push({
      id: 547,
     cid: uzt_547
  });
</script>

<!-- 底部浮层广告 -->
<script type="text/javascript">
  var uzt_106 = "_" + Math.random().toString(36).slice(2);
  document.write('<div id="' + uzt_106+ '"></div>');
  (window._Uzt_Slots = window._Uzt_Slots || []).push({
      id: 106,
      cid: uzt_106
  });
</script>

<div class="word-feedback" title="纠错">
  <div class="word-feedback-content">
    <form class="word-feedback-form">
      <input type="hidden" name="word" value="">
      <input type="hidden" name="extKey" value="">
      <input type="hidden" name="user" value="">
      <input type="hidden" name="source" value="0">
      <input type="hidden" name="fromLang" value="">
      <input type="hidden" name="toLang" value="">

      <fieldset>
        <legend>请选择错误类型（可多选）：</legend>
        <div class="word-feedback-controls clearfix">
          <label for="radio-type-pronounce">发音有误</label>
          <input type="checkbox" name="type" id="radio-type-pronounce" value="1">
          <label for="radio-type-paraphrase">释义有误</label>
          <input type="checkbox" name="type" id="radio-type-paraphrase" value="2">
          <label for="radio-type-sentence">例句有误</label>
          <input type="checkbox" name="type" id="radio-type-sentence" value="3">
          <label for="radio-type-content">内容有误</label>
          <input type="checkbox" name="type" id="radio-type-content" value="4">
        </div>
      </fieldset>
      <fieldset>
        <legend>其他错误（选填）：</legend>
        <div class="word-feedback-controls word-feedback-controls-comment">
          <span class="word-feedback-input-tips">
            <span class="word-feedback-input-message"></span>
            <span class="word-feedback-input-counter">
              <em>500</em>/500
            </span>
          </span>
          <textarea class="word-feedback-input" name="comment" placeholder="描述错误内容，便于小D更快核实更正。500 字以内。"></textarea>
        </div>
      </fieldset>
      <footer class="word-feedback-buttonspane">
        <button type="button" class="button button-submit" disabled="disabled">提交</button>
        <button type="button" class="button button-state-ghost button-cancel">取消</button>
      </footer>
    </form>
  </div>
</div>
<a class="feedback-link" href="https://ks.wjx.top/jq/20671240.aspx" target="_blank">小D<br />共建窝</a>

  </div>
</div>
<div id="footer-ft" class="footer"></div>

<script src="//res.hjfile.cn/tool/lib/jquery-3.2.1.min.js"></script>
<script src="//res.hjfile.cn/lib/hui/footer/0.5.2/footer.js"></script>
<script src="//trackcommon.hujiang.com/analytics/site/bulo_dict.js"></script>

  <script src="//res.hjfile.cn/tool/dict.hjenglish.com/manifest-bdfba.js"></script>
  
    <script src="//res.hjfile.cn/tool/dict.hjenglish.com/lib-70d99.js"></script>
  
  
    <script src="//res.hjfile.cn/tool/dict.hjenglish.com/common-3e6f1.js"></script>
  

<script src="//res.hjfile.cn/tool/dict.hjenglish.com/word-08dce.js"></script>
<script src="//res.hjfile.cn/co/gentian/gentian.1.2.0.min.js"></script>
<script src="//res.hjfile.cn/lib/uzhi/uzt.load.min.js"></script>

<!-- fingerprint 数据收集 -->
<!-- <script>
  (function() {
    window._hjfpcfg = {
      teamId: 'teamXiaoD',
      source: 'res.hjfile.cn/fp2/fp3.min.js'
    };

    var _hjfp = document.createElement("script");
    _hjfp.src = ('https:' === document.location.protocol ? 'https:' : 'http:') + '//' + _hjfpcfg.source;
    _hjfp.defer = 'defer';
    var __hjfp = document.getElementsByTagName("script")[0];
    __hjfp.parentNode.insertBefore(_hjfp, __hjfp);
  })();
</script> -->



</body>
</html>
    "#;

    static TEST_DATA_2: &str = r#"
<!DOCTYPE html>
<html lang="zh-Hans">
<head>
  <meta charset="utf-8">
  <meta name="keywords" content="你好,你好是什么意思,你好的翻译,你好的用法" />
  <meta name="description" content="沪江小D日语在线翻译是免费日语在线翻译网站、提供你好的翻译、你好是什么意思、你好的音标与发音、你好的含义及用法、以及你好的参考例句、你好是什么意思、解析你好的含义。" />
  <title>你好是什么意思_日语在线翻译_你好的翻译_含义_读音_用法_参考例句_沪江小D日语词典</title>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="shortcut icon" href="/favicon.ico">
  <script>
    var Gentian = {
      start_time: +new Date,
      project: '5003',
      business: '1000',
      request_id: '9ef898ac8af4457ea6205d8c1e3d46a5'
    };
    
    // http://res.hjfile.cn/co/gentian/error.1.2.0.min.js
    !function(e){function r(o){if(n[o])return n[o].exports;var i=n[o]={exports:{},id:o,loaded:!1};return e[o].call(i.exports,i,i.exports,r),i.loaded=!0,i.exports}var n={};return r.m=e,r.c=n,r.p="",r(0)}([function(e,r){!function(e){function r(e){var r,n,o=encodeURIComponent,i=[];for(r in e)n=e[r],Object(n)!==n&&i.push(r+"="+o(void 0==n||null==n?"":n));return i.join("&")}function n(e){var n=s+r(e);setTimeout(function(){var e=new Image(1,1);e.src=n,n=null},0)}var o=e,i=o.document,t=o.Gentian||o._gentian,a="";a=/qa|local|dev/.test(i.domain)?"//qagentian-frd.hjapi.com":/yz/.test(i.domain)?"//yzgentian-frd.hjapi.com":"//gentian-frd.hjapi.com";var s=a+"/gentian_error.gif?";e.onerror=function(){var e=arguments[0],r=arguments[1],o=arguments[2],i=arguments[3],a=e.toLowerCase(),s="script error",u={};u=a.indexOf(s)>-1?{message:"Script Error: See Browser Console for Detail",page_url:location.href||"",project:t.project||"",business:t.business||"",request_id:t.request_id||"",page_id:t.page_id||0}:{message:e||"Script Error: See Browser Console for Detail",url:r||"",line:o||"",column:i||"",page_url:location.href||"",project:t.project||"",business:t.business||"",request_id:t.request_id||"",page_id:t.page_id||0},n(u)}}(window)}]);
  </script>
  
  <link rel="stylesheet" href="//res.hjfile.cn/co/style/base.min.css" />
  
    <link rel="stylesheet" href="//res.hjfile.cn/tool/dict.hjenglish.com/common-892b9.css">
  
  <link rel="stylesheet" href="//res.hjfile.cn/tool/dict.hjenglish.com/word-6db50.css">
  <link rel="stylesheet" href="//res.hjfile.cn/lib/hui/footer/0.5.2/footer.css">
  
  <script>
    window.__static_public_path__ = '//res.hjfile.cn/tool/dict.hjenglish.com/';
  </script>
  
  
  
</head>
<body class="word">
<div class="wrapper">
  <header class="header">
  <nav class="main-nav">
    <div class="inner clearfix">
        <ul class="org-nav">
          <li>
            <a href="https://www.hujiang.com/" target="_blank">沪江首页</a>
          </li>
          <li>
            <a href="https://www.hujiang.com/xuexi/" target="_blank">学习资讯</a>
          </li>
          <li class="has-menu">
            学习工具
            <ul class="org-nav-apps">
              <li>
                <a class="app-d" href="https://www.hujiang.com/app/#hjdict" target="_blank">沪江小D</a>
              </li>
              <li>
                <a class="app-ci" href="https://www.hujiang.com/app/#cichang" target="_blank">开心词场</a>
              </li>
              <li>
                <a class="app-hs" href="https://www.hujiang.com/app/hujiang/" target="_blank">沪江学习</a>
              </li>
              <li>
                <a class="app-t" href="https://www.hujiang.com/app/#hjtlk" target="_blank">听力酷</a>
              </li>
            </ul>
          </li>
          <li>
            <a href="https://class.hujiang.com/" target="_blank">沪江网校</a>
          </li>
          <li>
            <a href="https://www.cctalk.com/" target="_blank">CC 课堂</a>
          </li>
        </ul>
        
          <ul class="auth-nav">
            <li>
              <a href="https://login.hujiang.com/?url=http%3A%2F%2Fdict.hjenglish.com%2Fjp%2Fcj%2F%25E4%25BD%25A0%25E5%25A5%25BD" rel="nofollow">登录</a>
            </li>
            <li>
              <a href="https://pass.hujiang.com/signup?url=http%3A%2F%2Fdict.hjenglish.com%2Fjp%2Fcj%2F%25E4%25BD%25A0%25E5%25A5%25BD" rel="nofollow">注册</a>
            </li>
          </ul>
        

        <div class="ad ad-175x35">
          <script type="text/javascript">
            var uzt_545 = "_" + Math.random().toString(36).slice(2);
            document.write('<div id="' + uzt_545+ '"></div>');
            (window._Uzt_Slots = window._Uzt_Slots || []).push({
                id: 545,
                cid: uzt_545,
              size: "175,35"
            });
          </script>
        </div>
    </div>
  </nav>
  
  <nav class="sub-nav">
    <div class="inner clearfix">
        <a href="/" class="logo">
          <h1>沪江小D</h1>
        </a>
        <ul class="site-nav">
          <li><a class="active" href="/">查词</a></li>
          <li><a class="" href="/app/trans">翻译</a></li>
          <li><a class="" href="/scb">生词本</a></li>
          <li><a target="_blank" href="https://www.hujiang.com/app/#hjdict">手机版小D</a></li>
          <li><a class="manual-translate-header" target="_blank" href="//hj.yeecloud.com">人工翻译</a></li>
        </ul>
    </div>
  </nav>
  
</header>
  <div class="container">
  
<div class="search-wrapper">
  <div class="search" data-lang="jp">
  <a class="search-logo" href="/"></a>
  <ul class="search-tabs">
    
    <li class="" data-lang="en">
      <a href="/">英语</a>
    </li>
    
    <li class="search-tabs-active" data-lang="jp">
      <a href="/jp">日语</a>
    </li>
    
    <li class="" data-lang="kr">
      <a href="/kr">韩语</a>
    </li>
    
    <li class="" data-lang="fr">
      <a href="/fr">法语</a>
    </li>
    
    <li class="" data-lang="de">
      <a href="/de">德语</a>
    </li>
    
    <li class="" data-lang="es">
      <a href="/es">西语</a>
    </li>
    
  </ul>
  <form class="search-form search-form-type-jp clearfix">
    <div class="search-input-wrapper">
      <input type="text" 
        class="search-input" 
        name="word"
        placeholder="在此输入要查询的单词" 
        autocomplete="off"
        value="你好"
      >

      <a class="search-vkeyboard">虚拟键盘</a>
    </div>
    <div class="search-buttonpane">
      <div class="search-buttonpane-group">
        <button type="button" class="button">查 询</button>
      </div>
      <div class="search-buttonpane-group search-buttonpane-group-jp">
        <button type="button" class="button" data-trans="jc">日 &rarr; 中</button>
        <button type="button" class="button" data-trans="cj">中 &rarr; 日</button>
      </div>
    </div>
  </form>
</div>


  <div class="ad ad-search-right-text">
    
      <script type="text/javascript">
        var uzt_157 = "_" + Math.random().toString(36).slice(2);
        document.write('<div id="' + uzt_157+ '"></div>');
        (window._Uzt_Slots = window._Uzt_Slots || []).push({
            id: 157,
          cid: uzt_157,
          size: "200,29"
        });
      </script>
      
  </div>
</div>
<main class="main">
  <div class="inner main-inner clearfix">
    <section class="content">
      
        <ul class="word-nav">
  
  <li><a data-id="detail">详细释义</a></li>
  
</ul>
<div class="word-details">

  
  

  <section class="word-details-content">
    
    <div class="word-details-pane">
      <header class="word-details-pane-header">
        <div class="word-info">

  
  <div class="word-text">
    <h2>你好</h2>

    
    
    <a href="https://login.hujiang.com/?url=http%3A%2F%2Fdict.hjenglish.com%2Fjp%2Fcj%2F%25E4%25BD%25A0%25E5%25A5%25BD" class="add-scb"></a>
    

  </div>

  
  

  
  <div class="pronounces">
    

    
    
      
      <span class="word-audio" data-src="https://tts.hjapi.com/py/2B91DF85176B5512?phoneticsymbol=%E4%BD%A0%E5%A5%BD"></span>
    

  </div>

</div>
        <div class="simple">
  

    
    
    <p>
    
      
      

      
      <span class="simple-definition">
      

      
      

      こんにちは；
      
      

      
      

      おはよう；
      
      

      
      

      こんばんは
      
      

      </span>
    </p>
    
  
</div>
      </header>
      <div class="word-details-ads-placeholder"></div>
      <div class="word-details-pane-content" 
        data-word="你好" 
        data-ext-key="" 
        data-from-lang="cn" 
        data-to-lang="jp"
        data-word-id="1555304"
        data-keywords="[&quot;你好&quot;]",
        data-remove-id="-1"
      >

      

      
      <div class="word-details-item detail" data-id="detail">
  <h2>详细释义</h2>
  <div class="word-details-item-content">
    <header>

      
      

      
      
        
      

      
      

    </header>
    <section class="detail-groups">

      
      
      
      

      
      

      <dl>
        <dt>
          感叹词
          
        </dt>
        
        <dd>
          <h3>

            

            <p>
            
            

            

            
            

            
            

            
            

            
            

            
            
            [こんにちは]你好。
            
            </p>
          </h3>

          
          

          <ul>
            
            <li>
              <p class="def-sentence-from">
                你好，天气真好啊！
                
                <span class="word-audio" data-src="https://tts.hjapi.com/jp/30FF898B4E1CBC4B9D48A11B04F8C4C18805A27FDF514D8E5EC67981F1ED7B72CDEB6A35CBAF740D812B67166DEEDBC0"></span>
                
              </p>
              <p class="def-sentence-to">
                こんにちは、いい天気ですね。
                
              </p>
            </li>
            
          </ul>
          
          
        </dd>
        
        <dd>
          <h3>

            

            <p>
            
            

            

            
            

            
            

            
            

            
            

            
            
            [やあ]喂，啊。
            
            </p>
          </h3>

          
          

          <ul>
            
            <li>
              <p class="def-sentence-from">
                噢，你好！
                
              </p>
              <p class="def-sentence-to">
                やあ、こんにちは。
                
                <span class="word-audio" data-src="https://tts.hjapi.com/jp/484A42BA004C34CEADA2EC8DA0F120256BBD5577BF5DA36C678AE15C4FB88907"></span>
                
              </p>
            </li>
            
            <li>
              <p class="def-sentence-from">
                啊，田中先生，您上哪儿去呀？
                
                <span class="word-audio" data-src="https://tts.hjapi.com/jp/484A42BA004C34CEB8F67390591EC223EAF2467EB20F651BE72A8903832080D1E93E80F3158B5AFD8F7E7423AC23A06A5A6EE58DC6495B69202E019762AFADA3"></span>
                
              </p>
              <p class="def-sentence-to">
                やあ、田中さん、どこへお出かけですか。
                
              </p>
            </li>
            
          </ul>
          
          
        </dd>
        
        <dd>
          <h3>

            

            <p>
            
            

            

            
            

            
            

            
            

            
            

            
            
            [おはよう]早啊！您早！早安！
            
            </p>
          </h3>

          
          

          <ul>
            
            <li>
              <p class="def-sentence-from">
                早上好！天气真好啊！
                
                <span class="word-audio" data-src="https://tts.hjapi.com/jp/51E16BC737FCA3A3275F2768E880222C761B02D20976B185E51267A895D662E57297B00AC5ABCCBE"></span>
                
              </p>
              <p class="def-sentence-to">
                おはよう！いい天気ですね。
                
              </p>
            </li>
            
            <li>
              <p class="def-sentence-from">
                老师早安！
                
                <span class="word-audio" data-src="https://tts.hjapi.com/jp/DE58112467F705F908AC4F16C7FAC6F08718B01680CA700128A8AEBD0860312CB1B6B4D9955FC83E"></span>
                
              </p>
              <p class="def-sentence-to">
                先生おはようございます。
                
              </p>
            </li>
            
          </ul>
          
          
        </dd>
        
        <dd>
          <h3>

            

            <p>
            
            

            

            
            

            
            

            
            

            
            

            
            
            [こんばんは]晚上好。
            
            </p>
          </h3>

          
          

          <ul>
            
            <li>
              <p class="def-sentence-from">
                “晚上好。”“晚上好，请进。”
                
                <span class="word-audio" data-src="https://tts.hjapi.com/jp/76BDF7EC3A35AD862CC46BE71EDC4739B18AA874801CC45CC81A6F96F7BB2E50744A6BFD3EC696F18E2015C13E22520774EC3B91B77F0A7388400A8A07771873CC6FD3526F48B1A7A787EE2C3B830767"></span>
                
              </p>
              <p class="def-sentence-to">
                －こんばんは。－こんばんは。どうぞお入りください。
                
              </p>
            </li>
            
          </ul>
          
          
        </dd>
        
      </dl>
      
    </section>
  </div>
</div>
      

      

      

      

      

      

      
      </div>
      <footer class="word-details-pane-footer">
        <button class="button word-details-button-feedback">纠错</button>
      </footer>
    </div>
    
  </section>
  <div class="word-details-ads">
    <div class="ad ad-details-text">
  
    <script type="text/javascript">
      var uzt_526 = "_" + Math.random().toString(36).slice(2);
      document.write('<div id="' + uzt_526+ '"></div>');
      (window._Uzt_Slots = window._Uzt_Slots || []).push({
          id: 526,
        cid: uzt_526,
        size: "200,29"
      });
    </script>
    
</div>
  </div>
</div>
      

      

    </section>
    <aside class="side">
    
      <section class="side-block">
        <div class="ad ad-230x160">
  
  <!-- 所有语种共用的右侧栏广告位 -->
  <script type="text/javascript">
    var uzt_531 = "_" + Math.random().toString(36).slice(2);
    document.write('<div id="' + uzt_531 + '"></div>');
    (window._Uzt_Slots = window._Uzt_Slots || []).push({
      id: 531,
      cid: uzt_531,
      size: "230,160"
    });
  </script>
</div>


  <div class="ad ad-230x160">
    <script type="text/javascript">
      var uzt_533 = "_" + Math.random().toString(36).slice(2);
      document.write('<div id="' + uzt_533 + '"></div>');
      (window._Uzt_Slots = window._Uzt_Slots || []).push({
          id: 533,
          cid: uzt_533,
        size: "230,160"
      });
    </script>
  </div>
  <div class="ad ad-230x160">
    <script type="text/javascript">
      var uzt_539 = "_" + Math.random().toString(36).slice(2);
      document.write('<div id="' + uzt_539 + '"></div>');
      (window._Uzt_Slots = window._Uzt_Slots || []).push({
          id: 539,
          cid: uzt_539,
        size: "230,160"
      });
    </script>
  </div>


      </section>

      
      

      
      
      <section class="side-block">
        <h2>今日热词</h2>
        <ul class="side-block-content hot-words">
          
          <li>
            <a href="/jp/jc/%E3%82%A8%E3%82%A2%E3%83%94%E3%82%B9%E3%83%88%E3%83%AB">
              エアピストル
            </a>
          </li>
          
          <li>
            <a href="/jp/jc/%E6%A0%BC%E5%B7%AE">
              格差
            </a>
          </li>
          
          <li>
            <a href="/jp/jc/%E6%BC%8F%E3%82%8C">
              漏れ
            </a>
          </li>
          
          <li>
            <a href="/jp/jc/%E5%AE%9F%E8%B3%AA">
              実質
            </a>
          </li>
          
          <li>
            <a href="/jp/jc/%E6%B1%82%E5%88%91">
              求刑
            </a>
          </li>
          
          <li>
            <a href="/jp/jc/%E9%AB%98%E7%82%89">
              高炉
            </a>
          </li>
          
          <li>
            <a href="/jp/jc/%E9%96%8B%E4%BC%9A%E5%BC%8F">
              開会式
            </a>
          </li>
          
        </ul>
      </section>
      

      
      
      <section class="side-block">
        <h2>内容推荐</h2>
        <ul class="side-block-content recommand-links">
          
          <li>
            <a href="https://jp.hjenglish.com/n/s/15893/" target="_blank">红白歌会</a>
          </li>
          
          <li>
            <a href="http://jp.hjenglish.com/nenglikaon2/n2linianzhenti/" target="_blank">日语二级真题</a>
          </li>
          
          <li>
            <a href="http://jp.hjenglish.com/n/s/39/" target="_blank">日语等级</a>
          </li>
          
          <li>
            <a href="http://jp.hjenglish.com/n/s/24/" target="_blank">日语自学</a>
          </li>
          
          <li>
            <a href="http://jp.hjenglish.com/n/s/36/" target="_blank">新编日语</a>
          </li>
          
          <li>
            <a href="http://www.hujiang.com/c/" target="_blank">课程专题</a>
          </li>
          
          <li>
            <a href="http://jp.hjenglish.com/zt/riyusanjichengjichaxun/" target="_blank">日语三级成绩查询</a>
          </li>
          
          <li>
            <a href="http://jp.hjenglish.com/zt/riyuerjichengjichaxun/" target="_blank">日语二级成绩查询</a>
          </li>
          
          <li>
            <a href="http://www.hjenglish.com/sijidaan/zuixinsijizuowen/" target="_blank">四级作文范文</a>
          </li>
          
          <li>
            <a href="http://jp.hjenglish.com/jlpt/beikao/" target="_blank">日语等级考试备考</a>
          </li>
          
          <li>
            <a href="http://www.hjenglish.com/wangming/" target="_blank">英文网名</a>
          </li>
          
          <li>
            <a href="http://www.hjenglish.com/sijidaan/siliujizuowenmuban/" target="_blank">英语四级作文万能模板</a>
          </li>
          
        </ul>
      </section>
      

    </aside>
  </div>
</main>

<!-- 弹窗广告 -->
<script type="text/javascript">
  var uzt_547 = "_" + Math.random().toString(36).slice(2);
  document.write('<div id="' + uzt_547+ '"></div>');
  (window._Uzt_Slots = window._Uzt_Slots || []).push({
      id: 547,
     cid: uzt_547
  });
</script>

<!-- 底部浮层广告 -->
<script type="text/javascript">
  var uzt_106 = "_" + Math.random().toString(36).slice(2);
  document.write('<div id="' + uzt_106+ '"></div>');
  (window._Uzt_Slots = window._Uzt_Slots || []).push({
      id: 106,
      cid: uzt_106
  });
</script>

<div class="word-feedback" title="纠错">
  <div class="word-feedback-content">
    <form class="word-feedback-form">
      <input type="hidden" name="word" value="">
      <input type="hidden" name="extKey" value="">
      <input type="hidden" name="user" value="">
      <input type="hidden" name="source" value="0">
      <input type="hidden" name="fromLang" value="">
      <input type="hidden" name="toLang" value="">

      <fieldset>
        <legend>请选择错误类型（可多选）：</legend>
        <div class="word-feedback-controls clearfix">
          <label for="radio-type-pronounce">发音有误</label>
          <input type="checkbox" name="type" id="radio-type-pronounce" value="1">
          <label for="radio-type-paraphrase">释义有误</label>
          <input type="checkbox" name="type" id="radio-type-paraphrase" value="2">
          <label for="radio-type-sentence">例句有误</label>
          <input type="checkbox" name="type" id="radio-type-sentence" value="3">
          <label for="radio-type-content">内容有误</label>
          <input type="checkbox" name="type" id="radio-type-content" value="4">
        </div>
      </fieldset>
      <fieldset>
        <legend>其他错误（选填）：</legend>
        <div class="word-feedback-controls word-feedback-controls-comment">
          <span class="word-feedback-input-tips">
            <span class="word-feedback-input-message"></span>
            <span class="word-feedback-input-counter">
              <em>500</em>/500
            </span>
          </span>
          <textarea class="word-feedback-input" name="comment" placeholder="描述错误内容，便于小D更快核实更正。500 字以内。"></textarea>
        </div>
      </fieldset>
      <footer class="word-feedback-buttonspane">
        <button type="button" class="button button-submit" disabled="disabled">提交</button>
        <button type="button" class="button button-state-ghost button-cancel">取消</button>
      </footer>
    </form>
  </div>
</div>
<a class="feedback-link" href="https://ks.wjx.top/jq/20671240.aspx" target="_blank">小D<br />共建窝</a>

  </div>
</div>
<div id="footer-ft" class="footer"></div>

<script src="//res.hjfile.cn/tool/lib/jquery-3.2.1.min.js"></script>
<script src="//res.hjfile.cn/lib/hui/footer/0.5.2/footer.js"></script>
<script src="//trackcommon.hujiang.com/analytics/site/bulo_dict.js"></script>
<script src="//res.hjfile.cn/tool/dict.hjenglish.com/manifest-bdfba.js"></script>

  <script src="//res.hjfile.cn/tool/dict.hjenglish.com/lib-70d99.js"></script>


  <script src="//res.hjfile.cn/tool/dict.hjenglish.com/common-3e6f1.js"></script>

<script src="//res.hjfile.cn/tool/dict.hjenglish.com/word-08dce.js"></script>
<script src="//res.hjfile.cn/co/gentian/gentian.1.2.0.min.js"></script>
<script src="//res.hjfile.cn/lib/uzhi/uzt.load.min.js"></script>

<!-- fingerprint 数据收集 -->
<!-- <script>
  (function() {
    window._hjfpcfg = {
      teamId: 'teamXiaoD',
      source: 'res.hjfile.cn/fp2/fp3.min.js'
    };

    var _hjfp = document.createElement("script");
    _hjfp.src = ('https:' === document.location.protocol ? 'https:' : 'http:') + '//' + _hjfpcfg.source;
    _hjfp.defer = 'defer';
    var __hjfp = document.getElementsByTagName("script")[0];
    __hjfp.parentNode.insertBefore(_hjfp, __hjfp);
  })();
</script> -->



</body>
</html>
    "#;
}
