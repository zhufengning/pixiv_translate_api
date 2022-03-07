# 一个翻译Pixiv上的小说到中文的api  
自动按段落拆分过长而无法全文翻译的文章  
请求格式`/api/transp/:id`  
返回的是一段原文接一段译文，如此循环。  
**使用前请在<http://api.fanyi.baidu.com/manage/developer>复制app_id和key填入main.rs中的baidu_trans函数**
