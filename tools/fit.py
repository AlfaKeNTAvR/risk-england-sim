import re, json, sys, glob, os
P = sys.argv[1]; OUT = sys.argv[2]; os.makedirs(OUT, exist_ok=True)
def get(ctx, path):
    v = ctx
    for k in path.split('.'):
        v = v[k]
    return v
def render(tpl, ctx):
    # expand innermost sc-for first via recursive descent
    out, i = [], 0
    pat = re.compile(r'<sc-for list="\{\{([\w.]+)\}\}" as="(\w+)"[^>]*>')
    while True:
        m = pat.search(tpl, i)
        if not m:
            out.append(subst(tpl[i:], ctx)); break
        out.append(subst(tpl[i:m.start()], ctx))
        depth, j = 1, m.end()
        while depth:
            o = tpl.find('<sc-for', j); c = tpl.find('</sc-for>', j)
            if o != -1 and o < c: depth += 1; j = o + 7
            else: depth -= 1; j = c + 9
        body = tpl[m.end():j - 9]
        for item in get(ctx, m.group(1)):
            out.append(render(body, {**ctx, m.group(2): item}))
        i = j
    return ''.join(out)
def subst(s, ctx):
    return re.sub(r'\{\{([\w.]+)\}\}', lambda m: str(get(ctx, m.group(1))), s)
for f in sorted(glob.glob(f"{P}/*.dc.html")):
    t = open(f).read()
    body = t.split('</helmet>')[1].split('</x-dc>')[0]
    js = t.split('renderVals() {')[1].split('\n  }\n}')[0]
    m = re.search(r'return (\{.*\});\s*$', js, re.S)
    try:
        ctx = json.loads(m.group(1))
    except Exception:
        continue  # only check JSON-data files and static ones
    html = '<!doctype html><meta charset=utf-8><style>body{margin:0}</style>' + render(body, ctx)
    open(f"{OUT}/{os.path.basename(f)}.html", 'w').write(html)
    print("rendered", os.path.basename(f))
