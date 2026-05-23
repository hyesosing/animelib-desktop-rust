import urllib.request, re, json

js_url = 'https://kodikplayer.com/assets/js/app.player_single.9abf69e8fcd08ec00343f70a42b529a07f728312fe4d74d23d8932a7b288d67f.js'
js = urllib.request.urlopen(urllib.request.Request(js_url, headers={'User-Agent': 'Mozilla/5.0'})).read().decode('utf-8')

# Find all ajax calls
ajax_calls = re.findall(r'\$\.ajax\(\{(.*?)\}\)', js, flags=re.DOTALL)
for call in ajax_calls:
    if 'type:"POST"' in call:
        print('--- AJAX CALL ---')
        # Extract url
        m_url = re.search(r'url:"(.*?)"', call)
        if m_url: print('URL:', m_url.group(1))
        # Extract data
        m_data = re.search(r'data:(.*?)(?:,success|$)', call)
        if m_data: print('DATA:', m_data.group(1))
