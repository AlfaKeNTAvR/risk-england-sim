import re, subprocess, json, sys, os
P, OUT = sys.argv[1], sys.argv[2]
os.makedirs(OUT, exist_ok=True)
src = open(os.path.join(os.path.dirname(__file__), 'fit.py')).read().split('for f in sorted')[0]
exec(src)
def frag(name):
    t = open(f'{P}/{name}.dc.html').read()
    body = t.split('</helmet>')[1].split('</x-dc>')[0]
    js = t.split('class Component extends DCLogic {')[1].rsplit('}', 1)[0]
    code = "class DCLogic{}; class Component extends DCLogic {" + js + "}; console.log(JSON.stringify(new Component().renderVals()))"
    ctx = json.loads(subprocess.run(['node', '-e', code], capture_output=True, text=True, check=True).stdout)
    return render(body, ctx)
FONT = '<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=IM+Fell+English+SC&amp;family=IM+Fell+English:ital@0;1&amp;family=EB+Garamond:wght@400;500;600&amp;display=swap">'
jobs = {
    '1-century-plaques (both sides, flip on long edge)': ['PlaqueSheet1', 'PlaqueSheet2', 'PlaqueSheet3', 'PlaqueSheet4'],
    '2-era-board (both sides, flip on short edge)': ['EraBoard1', 'EraBoard2'],
    '3-event-cards (one side)': ['EventsSheet1', 'EventsSheet2', 'EventsSheet3', 'EventsSheet4'],
    '4-treaty-cards (one side)': ['TreatyCards'],
    '5-house-rules-letter (both sides, flip on long edge)': ['RulesFront', 'RulesBack'],
    '6-house-rules-A5 (both sides, flip on long edge)': ['RulesA5p1', 'RulesA5p2', 'RulesA5p3', 'RulesA5p4'],
}
for title, names in jobs.items():
    land = names[0].startswith('Era')
    w, h = ('11in', '8.5in') if land else ('8.5in', '11in')
    pw, ph = (1056, 816) if land else (816, 1056)
    pages = '\n'.join(f'<section class="sheet">{frag(n)}</section>' for n in names)
    html = f'''<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>{title}</title>
{FONT}
<style>
@page {{ size: {w} {h}; margin: 0; }}
html, body {{ margin: 0; padding: 0; background: #888; }}
* {{ -webkit-print-color-adjust: exact; print-color-adjust: exact; }}
.sheet {{ width: {pw}px; height: {ph}px; overflow: hidden; margin: 24px auto; background: #fff; break-after: page; }}
.sheet:last-child {{ break-after: auto; }}
@media print {{ html, body {{ background: #fff; }} .sheet {{ margin: 0; }} }}
</style></head><body>
{pages}
</body></html>
'''
    fn = re.sub(r' \(.*', '', title) + '.html'
    open(os.path.join(OUT, fn), 'w').write(html)
    print(fn, len(names))
