use crypto::digest::Digest;
use crypto::md5::Md5;
use rand::Rng;
use reqwest::header::USER_AGENT;
use select::{document::Document, predicate::Attr};
use serde_json::Value;
use std::convert::Infallible;
use warp::Filter;

#[tokio::main]
async fn main() {
    // Match `/:Seconds`...
    let routes = warp::path("api").and(
        warp::path("transp").and(
            warp::path::param()
                // and_then create a `Future` that will simply wait N seconds...
                .and_then(transp),
        ),
    );

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}

async fn transp(id: u64) -> Result<impl warp::Reply, Infallible> {
    println!("new request!{}", id);
    let mut nr = String::new();
    match reqwest::Client::new()
        .get(format!("https://www.pixiv.net/novel/show.php?id={}", id))
        .header(
            USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:97.0) Gecko/20100101 Firefox/97.0",
        )
        .send()
        .await
    {
        Ok(res) => {
            match res.text().await {
                Ok(html) => {
                    let doc = Document::from(html.as_str());
                    let v = doc.find(Attr("name", "preload-data")).nth(0);
                    if let Some(v) = v {
                        match v.attr("content") {
                            Some(con) => {
                                let con: Result<Value, serde_json::Error> =
                                    serde_json::from_str(con);
                                match con {
                                    Ok(con) => {
                                        if let Some(novel) = con["novel"].as_object() {
                                            if let Some(v) = novel.iter().nth(0) {
                                                nr.push_str(
                                                    v.1["title"].as_str().unwrap_or_default(),
                                                );
                                                nr.push_str("\n");
                                                nr.push_str(
                                                    v.1["content"].as_str().unwrap_or_default(),
                                                );
                                            } else {
                                                return Ok(format!("???????????????????????????????????????"));
                                            }
                                        } else {
                                            return Ok(format!("????????????????????????????????????????????????"));
                                        }
                                    }
                                    Err(e) => {
                                        return Ok(format!("?????????????????????{:?}", e));
                                    }
                                };
                            }
                            None => {
                                return Ok(format!("?????????????????????"));
                            }
                        };
                    } else {
                        return Ok(format!("??????????????????????????????????????????"));
                    }
                }
                Err(e) => {
                    return Ok(format!("?????????????????????{:?}", e));
                }
            };
        }
        Err(e) => {
            return Ok(format!("?????????????????????{:?}", e));
        }
    };
    let mut cs: Vec<char> = nr.chars().collect();
    let mut ret = String::new();
    while cs.is_empty() == false {
        let mut i = if cs.len() < 2000 { cs.len() - 1 } else { 1999 };
        while cs[i] != '\n' {
            if i > 0 {
                i -= 1;
            } else {
                break;
            }
        }
        let para;
        if cs[i] != '\n' {
            let raw = cs.iter().take(2000).fold(String::new(), |mut s, v| {
                s.push(*v);
                s
            });
            ret.push_str(raw.as_str());
            ret.push_str("\n---\n");
            para = baidu_trans(raw.as_str()).await;
            let mut j = 0;
            while j < 2000 && cs.len() > 0 {
                cs.remove(0);
                j += 1;
            }
        } else {
            let raw = cs.iter().take(i + 1).fold(String::new(), |mut s, v| {
                s.push(*v);
                s
            });
            ret.push_str(raw.as_str());
            ret.push_str("\n---\n");
            para = baidu_trans(raw.as_str()).await;
            let mut j = 0;
            while j < i + 1 {
                cs.remove(0);
                j += 1;
            }
        }
        ret.push_str(para.as_str());
        ret.push_str("\n---\n");
    }
    return Ok(ret);
}

async fn baidu_trans(nr: &str) -> String {
    println!("asking baidu!");
    let app_id = "12345678";//??????
    let key = "12345678";//??????
    let salt: i32 = rand::thread_rng().gen();
    let salt = format!("{}", salt);
    let url = "https://fanyi-api.baidu.com/api/trans/vip/translate";
    let mut md5 = Md5::new();
    let s1 = format!("{}{}{}{}", app_id, nr, salt, key);
    md5.input_str(s1.as_str());
    let sign = md5.result_str();
    let params = [
        ("q", nr),
        ("from", "auto"),
        ("to", "zh"),
        ("appid", app_id),
        ("salt", salt.as_str()),
        ("sign", sign.as_str()),
    ];
    match reqwest::Client::new().post(url).form(&params).send().await {
        Ok(res) => match res.text().await {
            Ok(txt) => {
                let res: Result<Value, serde_json::Error> = serde_json::from_str(txt.as_str());
                match res {
                    Ok(res) => {
                        let mut ret = String::new();
                        let dft = Vec::<Value>::new();
                        if let Some(u) = res["error_code"].as_str() {
                            return format!("Error!{}:{}", u, res["error_msg"].as_str().unwrap_or_default())
                        }
                        let paras = res["trans_result"].as_array().unwrap_or(&dft);
                        for v in paras.iter() {
                            ret.push_str("\n");
                            ret.push_str(v["dst"].as_str().unwrap_or_default());
                        }
                        ret
                    }
                    Err(e) => {
                        format!("?????????{:?}", e)
                    }
                }
            }
            Err(e) => {
                format!("?????????{:?}", e)
            }
        },
        Err(e) => {
            format!("?????????{:?}", e)
        }
    }
}
