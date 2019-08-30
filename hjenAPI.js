async function word(word) {
    const request = require('request');
    let jsonData = {
        word: '',
        word_katakana: '',
        word_audio_en: '',
        word_audio_us: '',
        simple_explain: [],
        more_details: {},
        inflections: [],
        synonym: [],
        antonym: [],
        English_explains: {},
        phrase: []
    };

    let header = {
        headers: {
            'User-Agent': 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/69.0.3497.81 Safari/537.36',
            'Cookie': 'HJ_UID=0f406091-be97-6b64-f1fc-f7b2470883e9; HJ_CST=1; HJ_CSST_3=1;TRACKSITEMAP=3%2C; HJ_SID=393c85c7-abac-f408-6a32-a1f125d7e8c6; _REF=; HJ_SSID_3=4a460f19-c0ae-12a7-8e86-6e360f69ec9b; _SREF_3=; HJ_CMATCH=1'
        }
    };
    return await new Promise(function (resolve, reject) {
        request(encodeURI('https://dict.hjenglish.com/w/' + word), header, (err, res, body) => {
            if (err) {
                return console.log(err);
            }
            const cheerio = require('cheerio'), $ = cheerio.load(body);
            $(`.word-details-pane`).map(function (index, html) {
                // if (index !== 0){
                //     console.log();
                // }
                let sub$ = cheerio.load(html);
                let word_text = sub$('.word-info .word-text h2').text();
                if (sub$('.word-info .pronounces .pronounce-value-en').text() === "") {
                    let word_katakana = sub$('.word-info .pronounces span').text();
                    let word_audio = sub$('.word-info .pronounces .word-audio').attr('data-src');
                    // console.log(word_text,word_katakana,word_audio,'\nSimple explanation:');
                    jsonData.word = word_text;
                    jsonData.word_katakana = word_katakana;
                    jsonData.word_audio_en = word_audio;
                } else {
                    let word_audio_en = "英 " + sub$('.word-info .pronounces .pronounce-value-en').text() + ' ' + sub$('.word-info .pronounces .word-audio-en').attr('data-src');
                    let word_audio_us = "美 " + sub$('.word-info .pronounces .pronounce-value-us').text() + ' ' + sub$('.word-info .pronounces .word-audio').last().attr('data-src');
                    // console.log(word_text);
                    // console.log(word_audio_en);
                    // console.log(word_audio_us,'\nSimple explanation:');
                    jsonData.word = word_text;
                    jsonData.word_audio_en = word_audio_en;
                    jsonData.word_audio_us = word_audio_us;
                }

                let word_simple = sub$('.simple p .simple-definition a');
                let word_simple_jsonData = [];
                if (word_simple.text() === "") {
                    sub$('.simple p').map(function (index, html) {
                        let word_simple_p$ = cheerio.load(html);
                        // console.log('   '+(index+1) +')'+ word_simple_p$.text().replace(/[\r\n | \n | \r]/g, " ").replace(/ +/g, " "))
                        jsonData.simple_explain.push((index + 1) + ')' + word_simple_p$.text().replace(/[\r\n | \n | \r]/g, " ").replace(/ +/g, " "));
                    })
                } else {
                    word_simple.map(function (index, html) {
                        let word_simple_simple_definition_a$ = cheerio.load(html);
                        // console.log('   '+(index+1)+'.'+word_simple_simple_definition_a$.text());
                        jsonData.simple_explain.push((index + 1) + '.' + word_simple_simple_definition_a$.text());
                    })
                }

                sub$('.word-details-pane-content .word-details-item').map(function (index, html) {
                    // if (index ===0){
                    //     console.log();
                    //     console.log('more details:');
                    // }
                    let word_detail$ = cheerio.load(html);
                    word_detail$('.word-details-item-content .detail-groups dl').map(function (index, html) {
                        let word_detail_dl$ = cheerio.load(html);
                        let word_attribute = word_detail_dl$('dt').text().replace(/[\r\n | \n | \r]/g, " ").replace(/ +/g, " ");
                        // console.log(" word attribute:",word_attribute);
                        jsonData.more_details[word_attribute] = {};
                        word_detail_dl$("dd").map(function (index, html) {
                            let word_detail_dl_dd$ = cheerio.load(html);
                            let word_detail_imi = (index + 1) + '.' + word_detail_dl_dd$("h3").text().replace(/[\r\n | \n | \r]/g, " ").replace(/ +/g, " ");
                            // console.log('     '+word_detail_imi);
                            jsonData.more_details[word_attribute][word_detail_imi] = [];
                            word_detail_dl_dd$("ul li").map(function (index, html) {
                                let word_detail_dl_dd_ul_li$ = cheerio.load(html);
                                let eg = word_detail_dl_dd_ul_li$('.def-sentence-from').text().replace(/[\r\n | \n | \r]/g, " ").replace(/ +/g, " ");
                                // console.log('       '+eg);
                                let eg2 = word_detail_dl_dd_ul_li$('.def-sentence-to').text().replace(/[\r\n | \n | \r]/g, " ").replace(/ +/g, " ");
                                // console.log('       '+eg2);
                                jsonData.more_details[word_attribute][word_detail_imi].push({eg, eg2});
                            })
                        })
                    });

                    word_detail$('.word-details-item-content .phrase-items li').map(function (index, html) {
                        // if (index === 0){
                        //     console.log();
                        //     console.log('phrase:');
                        // }
                        let word_detail_li$ = cheerio.load(html);
                        // console.log('   '+(index+1)+'.'+word_detail_li$.text().replace(/[\r\n | \n | \r]/g, " ").replace(/ +/g, " "));
                        jsonData.phrase.push(word_detail_li$.text().replace(/[\r\n | \n | \r]/g, " ").replace(/ +/g, " "));
                    });

                    word_detail$('.word-details-item-content .enen-groups dl').map(function (index, html) {
                        // if (index === 0){
                        //     console.log();
                        //     console.log('use English to explains:');
                        // }
                        let word_detail_dl$ = cheerio.load(html);
                        let word_attribute = word_detail_dl$('dt').text().replace(/[\r\n | \n | \r]/g, " ").replace(/ +/g, " ");
                        // console.log(" word attribute:",word_attribute);
                        jsonData.English_explains[word_attribute] = [];
                        word_detail_dl$("dd").map(function (index, html) {
                            let word_detail_dl_dd$ = cheerio.load(html);
                            // console.log('    '+(index+1)+'.'+word_detail_dl_dd$.text());
                            jsonData.English_explains[word_attribute].push(word_detail_dl_dd$.text());
                        })
                    });

                    word_detail$('.word-details-item-content .inflections-items li').map(function (index, html) {
                        // if (index === 0){
                        //     console.log();
                        //     console.log('inflections:');
                        // }
                        let word_detail_li$ = cheerio.load(html);
                        // console.log('   '+(index+1)+'.'+word_detail_li$.text().replace(/[\r\n | \n | \r]/g, " ").replace(/ +/g, " "));
                        jsonData.inflections.push(word_detail_li$.text().replace(/[\r\n | \n | \r]/g, " ").replace(/ +/g, " "));
                    });

                    word_detail$('.word-details-item-content .syn table tbody td').map(function (index, html) {
                        // if (index === 0){
                        //     console.log();
                        //     console.log('synonym(同义词):');
                        // }
                        let word_detail_li$ = cheerio.load(html);
                        // console.log('   '+(index+1)+'.'+word_detail_li$.text().replace(/[\r\n | \n | \r]/g, " ").replace(/ +/g, " "));
                        jsonData.synonym.push(word_detail_li$.text().replace(/[\r\n | \n | \r]/g, " ").replace(/ +/g, " "));
                    });

                    word_detail$('.word-details-item-content .ant table tbody td').map(function (index, html) {
                        // if (index === 0){
                        // console.log();
                        // console.log('antonym(反义词):');
                        // }
                        let word_detail_li$ = cheerio.load(html);
                        // console.log('   '+(index+1)+'.'+word_detail_li$.text().replace(/[\r\n | \n | \r]/g, " ").replace(/ +/g, " "));
                        jsonData.antonym.push(word_detail_li$.text().replace(/[\r\n | \n | \r]/g, " ").replace(/ +/g, " "));
                    })
                });
            });
            resolve(JSON.stringify(jsonData,null,1))
        });
    });
}
exports.word = word;
