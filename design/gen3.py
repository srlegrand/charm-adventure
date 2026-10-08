# Design pass 3: after the user's reference. Needle limbs, mask faces, bell cloaks, plum line, cel shading.
P = '#3b1436'; DK = '#2a0f2c'
def S(w=5): return f'stroke="{P}" stroke-width="{w}" stroke-linejoin="round" stroke-linecap="round"'
def g(x, y, body, s=1.0, flip=False): return f'<g transform="translate({x},{y}) scale({-s if flip else s},{s})">{body}</g>'
def needle(x0, x1, y, xt, yt=0, fill=DK): return f'<path d="M{x0},{y} L{x1},{y} L{xt},{yt}Z" fill="{fill}" {S(4)}/>'
def almond(x, y, w, h, rot, fill=P, hi=True):
    o = f'<g transform="translate({x},{y}) rotate({rot})"><path d="M{-w},0 C{-w*.4},{-h} {w*.6},{-h*.9} {w},{-h*.15} C{w*.5},{h*.7} {-w*.5},{h*.6} {-w},0Z" fill="{fill}"/>'
    if hi: o += f'<circle cx="{-w*.3}" cy="{-h*.2}" r="{h*.22}" fill="#fff" opacity=".9"/>'
    return o + '</g>'
def spark(x, y, s=1, c='#f2b24a'): return f'<path transform="translate({x},{y}) scale({s})" d="M0,-14 L7,0 L0,14 L-7,0Z" fill="{c}" opacity=".9"/>'
def arm(d, w=9): return f'<path d="{d}" fill="none" stroke="{P}" stroke-width="{w+6}" stroke-linecap="round" stroke-linejoin="round"/><path d="{d}" fill="none" stroke="{DK}" stroke-width="{w}" stroke-linecap="round" stroke-linejoin="round"/>'

DEFS = f'''<defs>
<pattern id="br" width="10" height="26" patternUnits="userSpaceOnUse"><rect width="10" height="26" fill="#f6efe0"/><rect width="10" height="11" fill="#27305c"/></pattern>
<pattern id="ch" width="22" height="22" patternUnits="userSpaceOnUse" patternTransform="rotate(12)"><rect width="22" height="22" fill="#a8262a"/><rect x="5" y="5" width="8" height="8" fill="none" stroke="#f1d6a8" stroke-width="2.2"/><circle cx="17" cy="17" r="1.8" fill="#f1d6a8"/></pattern>
<linearGradient id="leg" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{DK}"/><stop offset="1" stop-color="#8a3a3a"/></linearGradient>
</defs>'''

def hero():
    sk = '#f3d2b3'
    cloak = "M-18,-344 L-66,-372 L-52,-322 C-72,-262 -88,-214 -104,-166 L-58,-190 L-34,-150 L-4,-188 L26,-152 L52,-190 L96,-164 C82,-220 64,-272 52,-322 L66,-372 L18,-344Z"
    return f'''
    <path d="M-20,-200 L0,-200 L-30,0Z M6,-200 L26,-200 L40,0Z" fill="url(#leg)" {S(4)}/>
    <path d="M-10,-360 L10,-360 L8,-330 L-8,-330Z" fill="{DK}" {S(4)}/>
    <path d="{cloak}" fill="url(#br)" {S()}/>
    <path d="M52,-322 C64,-272 82,-220 96,-164 L52,-190 L26,-152 L18,-200 C30,-250 40,-290 52,-322Z" fill="{P}" opacity=".28"/>
    <path d="M-18,-344 L-66,-372 L-52,-322 L-20,-318Z M18,-344 L66,-372 L52,-322 L20,-318Z" fill="{DK}" {S(4)}/>
    {arm("M50,-318 L96,-268 L46,-232")}
    <path d="M-66,-262 L-176,-34 q-10,24 12,32 l8,-12 q-10,-6 -4,-16 L-52,-256Z" fill="#e2452e" {S(4)}/>
    <path d="M-62,-250 L-166,-36" stroke="#ff9a7a" stroke-width="3" stroke-linecap="round"/>
    {arm("M-50,-318 L-92,-262 L-78,-222")}
    <circle cx="-76" cy="-224" r="11" fill="{sk}" {S(4)}/><circle cx="46" cy="-232" r="11" fill="{sk}" {S(4)}/>
    <ellipse cx="-50" cy="-424" rx="9" ry="13" fill="#e2b894" {S(4)}/><circle cx="-54" cy="-408" r="7" fill="none" stroke="#f2b24a" stroke-width="4"/>
    <path d="M-46,-428 C-52,-506 52,-506 46,-428 C44,-398 22,-380 0,-352 C-22,-380 -44,-398 -46,-428Z" fill="{sk}" {S()}/>
    <path d="M-27,-388 C-10,-396 10,-396 27,-388 C16,-374 8,-364 0,-352 C-8,-364 -16,-374 -27,-388Z" fill="#4a3a52"/><path d="M-5,-368 L5,-368 L0,-354Z" fill="#cfc6d2"/>
    <path d="M-46,-428 C-52,-506 52,-506 46,-428 C44,-398 22,-380 0,-352 C-22,-380 -44,-398 -46,-428Z" fill="none" {S()}/>
    <path d="M22,-470 C34,-462 40,-446 40,-430 C28,-440 20,-454 22,-470Z" fill="#fff" opacity=".75"/>
    {almond(-20,-424,15,10,24)}{almond(20,-424,15,10,156)}
    {spark(84,-440,1.1)}{spark(100,-410,.6)}{spark(-120,-300,.8)}'''

def charm():
    sk = '#a9683f'; hair = '#2a1024'
    return f'''
    <path d="M-20,-200 L0,-200 L-30,0Z M6,-200 L26,-200 L40,0Z" fill="url(#leg)" {S(4)}/>
    <path d="M-10,-360 L10,-360 L8,-330 L-8,-330Z" fill="{DK}" {S(4)}/>
    <path d="M-30,-486 C-64,-470 -66,-410 -56,-372 C-62,-330 -58,-290 -78,-262 C-50,-268 -34,-300 -34,-340 L34,-340 C34,-300 44,-262 70,-240 C76,-290 60,-340 56,-372 C66,-410 64,-470 30,-486Z" fill="{hair}" {S()}/>
    <path d="M-14,-346 C-34,-300 -70,-226 -92,-176 C-70,-160 -48,-176 -30,-158 C-10,-176 12,-176 30,-158 C48,-176 70,-160 92,-176 C70,-226 34,-300 14,-346Z" fill="url(#ch)" {S()}/>
    <path d="M14,-346 C34,-300 70,-226 92,-176 C70,-160 48,-176 30,-158 C30,-220 26,-290 14,-346Z" fill="{P}" opacity=".3"/>
    <path d="M-30,-158 C-28,-220 -18,-290 -6,-342" stroke="{P}" stroke-width="3" fill="none" opacity=".6"/>
    <path d="M62,-262 L70,-274 L170,4Z" fill="#eaf6ff" {S(4)}/><path d="M70,-262 L164,-4" stroke="#9fd0ea" stroke-width="2.5"/>
    {arm("M22,-322 L60,-296 L62,-272", 8)}
    <path d="M44,-258 q22,-30 42,-8" fill="none" stroke="{P}" stroke-width="12" stroke-linecap="round"/><path d="M44,-258 q22,-30 42,-8" fill="none" stroke="#f2b24a" stroke-width="6" stroke-linecap="round"/>
    <path d="M60,-278 L48,-312" stroke="{P}" stroke-width="10" stroke-linecap="round"/><circle cx="46" cy="-320" r="9" fill="none" stroke="{P}" stroke-width="5"/><circle cx="46" cy="-320" r="9" fill="none" stroke="#f2b24a" stroke-width="2"/>
    {arm("M-22,-322 L-56,-280 L-40,-246", 8)}<circle cx="-40" cy="-246" r="10" fill="{sk}" {S(4)}/>
    <path d="M-46,-428 C-52,-506 52,-506 46,-428 C44,-398 22,-380 0,-352 C-22,-380 -44,-398 -46,-428Z" fill="{sk}" {S()}/>
    <path d="M-50,-420 C-58,-514 58,-514 50,-420 C34,-462 -4,-472 -50,-420Z" fill="{hair}" {S(4)}/>
    <path d="M22,-494 C46,-476 50,-440 58,-390" stroke="#d9c08a" stroke-width="7" fill="none" stroke-linecap="round"/>
    {almond(-20,-420,15,10,24,'#fff',False)}{almond(20,-420,15,10,156,'#fff',False)}
    {spark(-90,-470,1)}{spark(-106,-436,.6)}{spark(96,-400,.7,'#e58a8a')}'''

def louis():
    f = '#e9c896'; d = '#c79c63'
    hind = "M-110,-196 C-124,-156 -104,-122 -86,-106 L-108,-58 L-98,0 L-88,0 L-90,-58 L-60,-106 C-58,-140 -70,-168 -86,-180Z"
    tail = "M-108,-206 C-156,-186 -116,-124 -146,-84 C-162,-60 -192,-62 -194,-90"
    return f'''
    <path d="{tail}" fill="none" stroke="{P}" stroke-width="11" stroke-linecap="round"/><path d="{tail}" fill="none" stroke="{f}" stroke-width="4" stroke-linecap="round"/>
    <path transform="translate(24,0)" d="{hind}" fill="{d}" {S(4)}/>{needle(74,90,-140,92,0,d)}
    <path d="M52,-234 C30,-222 4,-228 -22,-244 C-52,-262 -86,-250 -100,-224 C-118,-208 -122,-176 -104,-150 C-90,-172 -60,-186 -24,-178 C0,-152 30,-110 58,-110 C84,-114 96,-150 88,-198Z" fill="{f}" {S()}/>
    <path d="M-24,-178 C0,-152 30,-110 58,-110 C84,-114 96,-150 88,-198 C80,-150 60,-130 40,-134 C16,-150 0,-170 -24,-178Z" fill="{d}"/>
    <path d="{hind}" fill="{f}" {S(4)}/>{needle(50,70,-128,58,0,f)}
    <path d="M48,-230 C70,-262 84,-292 90,-320 L126,-308 C118,-270 106,-230 88,-192Z" fill="{f}" {S()}/>
    <path d="M62,-250 L112,-232 L106,-212 L84,-216 L44,-186 L60,-226Z" fill="#c8322c" {S(4)}/>
    <path d="M86,-324 C94,-346 122,-346 138,-332 L206,-308 C214,-302 210,-294 200,-294 L132,-296 C108,-292 88,-304 86,-324Z" fill="{f}" {S()}/>
    <path d="M92,-330 C84,-352 80,-374 84,-394 C102,-380 116,-360 122,-340Z" fill="{d}" {S(4)}/><path d="M100,-352 C98,-364 96,-374 94,-380" stroke="{P}" stroke-width="3" fill="none" stroke-linecap="round"/>
    <circle cx="205" cy="-301" r="6" fill="{P}"/>
    {almond(132,-318,12,7,16)}
    {spark(160,-370,.8)}{spark(-150,-270,.6)}'''

def tomato(r, col='#d8342c', sh='#9c1c26', hi='#f58a6a', sx=1.0, sy=1.0, legs=(1, 1), leglen=None, horn=1.0, leaf='#3f7a3a', extra='', back='', bent=False):
    w, h = r * sx, r * sy; L = leglen if leglen is not None else r * .9; cy = -L - h * .85
    o = back.replace('CY', str(cy))
    if L > 0:
        if bent:
            o += f'<path d="M{-w*.35},{cy+h*.7} L{-w*1.0},{-L*.55} L{-w*.5},0 M{w*.35},{cy+h*.7} L{w*1.0},{-L*.55} L{w*.6},0" fill="none" {S(6)}/>'
        else:
            o += needle(-w*.5, -w*.2, cy+h*.6, -w*.55*legs[0]) + needle(w*.2, w*.5, cy+h*.6, w*.6*legs[1])
    o += f'<g transform="translate(0,{cy})">'
    o += f'<path d="M-6,{-h*.9} C{-r*.5*horn},{-h*1.3} {-r*.9*horn},{-h*1.5} {-r*.7*horn},{-h*2.0*horn} C{-r*.3*horn},{-h*1.6} {-r*.1},{-h*1.3} 4,{-h*.95} C{r*.2},{-h*1.4} {r*.5*horn},{-h*1.7} {r*.9*horn},{-h*2.2*horn} C{r*.8*horn},{-h*1.6} {r*.5},{-h*1.2} 10,{-h*.9}Z" fill="{leaf}" {S(4)}/>'
    o += f'<ellipse cx="0" cy="0" rx="{w}" ry="{h}" fill="{col}" {S()}/>'
    o += f'<path d="M{w*.2},{h*.96} C{w*.9},{h*.7} {w*1.02},{-h*.1} {w*.7},{-h*.66} C{w*.7},{h*.1} {w*.5},{h*.6} {w*.2},{h*.96}Z" fill="{sh}"/>'
    o += f'<path d="M{-w*.72},{-h*.3} C{-w*.66},{-h*.66} {-w*.3},{-h*.82} {-w*.12},{-h*.78} C{-w*.4},{-h*.6} {-w*.6},{-h*.4} {-w*.72},{-h*.3}Z" fill="{hi}"/>'
    o += f'<ellipse cx="0" cy="0" rx="{w}" ry="{h}" fill="none" {S()}/>'
    e = r * .3
    o += almond(-w*.3, h*.05, e, e*.62, 26) + almond(w*.36, h*.05, e, e*.62, 154)
    return o + extra + '</g>'

def cape(w, c, d):
    k = d / 140
    return f'<path transform="translate(0,CY)" d="M{-w*.8},-10 C{-w*1.5},{40*k} {-w*1.6},{90*k} {-w*1.9},{130*k} L{-w*1.1},{110*k} L{-w*.6},{140*k} L0,{112*k} L{w*.6},{140*k} L{w*1.1},{110*k} L{w*1.9},{130*k} C{w*1.6},{90*k} {w*1.5},{40*k} {w*.8},-10Z" fill="{c}" {S()}/>'
wingp = lambda s: f'<path transform="translate(0,CY) scale({s},1)" d="M10,0 C40,-70 100,-90 150,-70 C120,-50 130,-30 104,-22 C116,-4 90,6 60,10Z" fill="#5c9a4a" {S(4)}/>'
TOMS = [
 ('CHERRY', 'swarm walker', tomato(24, horn=.9)),
 ('BEEFSTEAK', 'heavy brute', tomato(58, '#b8242a', '#7a1424', '#e0705a', 1.25, .9, leglen=44, horn=.6,
    extra=f'<path d="M-72,-10 L-112,60 L-80,40Z M72,-10 L112,60 L80,40Z" fill="{DK}" {S(4)}/>')),
 ('HOPPER', 'leaps at you', tomato(30, '#f08a24', '#b84a14', '#ffc880', leglen=44, bent=True)),
 ('PLUM', 'charging lancer', tomato(28, sx=.8, sy=1.5, leglen=50, horn=.7,
    extra=f'<path d="M30,10 L150,70 L28,26Z" fill="#eaf6ff" {S(4)}/>')),
 ('GREENIE', 'seed spitter', tomato(30, '#9ac83e', '#4f8a2a', '#d8f08a', leaf='#2f5a2c',
    extra=f'<circle cx="60" cy="18" r="8" fill="#d8f08a" {S(3)}/><circle cx="86" cy="12" r="5" fill="#d8f08a" {S(3)}/>')),
 ('VINE FLYER', 'dive bomber', tomato(26, leglen=0, back=wingp(1) + wingp(-1)).replace('translate(0,-22', 'translate(0,-130')),
 ('SUN-DRIED', 'armoured', tomato(34, '#8a2a24', '#4a1020', '#b85a44', 1.1, .8, leglen=20, leaf='#7a6a34', horn=.7,
    back=cape(34, '#5a2a3a', 41), extra=f'<path d="M-24,-16 q10,8 2,18 M12,-20 q-8,10 2,18" stroke="{P}" stroke-width="3.5" fill="none"/>')),
 ('BIG TOM', 'boss', tomato(80, sx=1.1, sy=.95, leglen=70, horn=.55, back=cape(80, DK, 132),
    extra=f'<path d="M-50,-70 L-42,-128 L-20,-90 L0,-140 L20,-90 L42,-128 L50,-70Z" fill="#f2b24a" {S()}/>')),
]
def label(x, y, t, s, c='#8a2a3a'):
    return (f'<text x="{x}" y="{y}" text-anchor="middle" font-size="30" font-weight="800" letter-spacing="3" fill="{c}">{t}</text>'
            f'<text x="{x}" y="{y+24}" text-anchor="middle" font-size="15" fill="#8a6a70">{s}</text>')
p = [DEFS, '<rect width="1800" height="1300" fill="#faf5ee"/>',
     g(330, 610, hero(), .89), label(330, 670, 'SIMON', 'player 1, the crowbar', '#b23a2a'),
     g(900, 610, charm(), .89), label(900, 670, 'CHARM', 'player 2, the sword', P),
     g(1420, 610, louis(), 1.05), label(1440, 670, 'LOUIS', 'called with Y', '#a8683f')]
xs = [100, 300, 490, 740, 910, 1120, 1370, 1620]
for (n, s, body), x in zip(TOMS, xs):
    p += [g(x, 1200, body, 1.0, flip=True), label(x, 1250, n.title() if False else n, s)]
open('sheet3.svg', 'w').write(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1800 1300" width="1800" height="1300" font-family="DejaVu Sans, Arial, sans-serif">{"".join(p)}</svg>')
