function word(word) {
    const request = require('request');
    let header = {
        headers: {
            'User-Agent': 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/69.0.3497.81 Safari/537.36',
            'Cookie': 'HJ_UID=0f406091-be97-6b64-f1fc-f7b2470883e9; HJ_CST=1; HJ_CSST_3=1;TRACKSITEMAP=3%2C; HJ_SID=393c85c7-abac-f408-6a32-a1f125d7e8c6; _REF=; HJ_SSID_3=4a460f19-c0ae-12a7-8e86-6e360f69ec9b; _SREF_3=; HJ_CMATCH=1'
        }
    };
    request(encodeURI('https://dict.hjenglish.com/jp/jc/'+word), header, (err, res, body) => {
        // console.log('----------------------------');
        if (err) {
            return console.log(err);
        }
        const cheerio = require('cheerio'), $ = cheerio.load(body);
        $(`.word-details-pane`).map(function (index, html) {
            if (index !== 0){
                console.log();
            }
            let sub$ = cheerio.load(html);
            let word_text = sub$('.word-info .word-text h2').text();
            let word_katakana = sub$('.word-info .pronounces span').text();
            let word_audio = sub$('.word-info .pronounces .word-audio').attr('data-src');
            let word_simple = sub$('.simple').text().replace(/[\r\n | \n | \r]/g, " ").
            replace(/ +/g, "\n").replace(/\n。/g, "。").split('\n');

            console.log(word_text, word_katakana, word_audio,'\nSimple explanation:');
            for (let value of word_simple) {
                if (value !== "") {
                    console.log('   '+value)
                }
            }
            sub$('.word-details-pane-content .word-details-item').map(function (index,html) {
                console.log('more details:');
                let word_detail$ = cheerio.load(html);
                word_detail$('.word-details-item-content .detail-groups dl').map(function (index,html) {
                    let word_detail_dl$ = cheerio.load(html);
                    let word_attribute = word_detail_dl$('dt').text().replace(/[\r\n | \n | \r]/g, "").replace(/ +/g, "");
                    console.log(" word attribute:",word_attribute);
                    word_detail_dl$("dd").map(function (index,html) {
                        let word_detail_dl_dd$ = cheerio.load(html);
                        let word_detail_imi = '  '+(index+1) +'.'+word_detail_dl_dd$("h3").text().replace(/[\r\n | \n | \r]/g, "").replace(/ +/g, "");
                        console.log(word_detail_imi);
                        let eg = word_detail_dl_dd$("ul").text().replace(/[\r\n | \n | \r]/g, " ").replace(/ +/g, "\n").replace(/\n。/g, "。").split('\n');
                        for (let value of eg) {
                            if (value !== "") {
                                console.log('    '+value)
                            }
                        }
                    })
                });
            });
        })
    });
}

word(process.argv.splice(2)[0]);
