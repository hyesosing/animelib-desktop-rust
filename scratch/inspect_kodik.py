import urllib.request, re

req = urllib.request.Request('https://kodikplayer.com/seria/1256432/f23226b44e3fddbea37b65f66898cfb2/720p', headers={'User-Agent': 'Mozilla/5.0'})
html = urllib.request.urlopen(req).read().decode('utf-8')
for match in re.finditer(r'src="(/assets/js/app.*?\.js)"', html):
    js_url = 'https://kodikplayer.com' + match.group(1)
    print('Found JS:', js_url)
    js = urllib.request.urlopen(urllib.request.Request(js_url, headers={'User-Agent': 'Mozilla/5.0'})).read().decode('utf-8')
    post_urls = re.findall(r'type:"POST",url:"(.*?)"', js)
    print('POST endpoints:', post_urls)
    print('Atob usages:', len(re.findall(r'atob', js)))
